use fenix_storage::{
    CompletionCheck, CompletionReport, EvidenceInput, EvidenceKind, EvidenceState, Plan,
    PlanNodeSpec, PlanRevision, Store, SubagentSpec, TaskBinding,
};
use serde_json::json;

fn node(id: &str, deps: &[&str]) -> PlanNodeSpec {
    PlanNodeSpec {
        profile: "general".into(),
        id: id.into(),
        objective: format!("Produce {id}"),
        expected_output: format!("Correct {id}.txt"),
        depends_on: deps.iter().map(|s| s.to_string()).collect(),
        skills: vec![],
    }
}
fn setup(nodes: Vec<PlanNodeSpec>) -> (tempfile::TempDir, Store, Plan) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("db")).unwrap();
    let owner = store
        .enqueue_bound(
            "Produce the requested report without invented requirements",
            "web",
            None,
            &TaskBinding::default(),
        )
        .unwrap();
    let plan = store
        .create_plan(&owner.id, "Produce report", "", nodes)
        .unwrap();
    (dir, store, plan)
}
fn child(store: &Store, plan: &Plan, label: &str) -> String {
    store
        .create_subagent(
            &plan.task_id,
            &SubagentSpec {
                profile: "general".into(),
                objective: label.into(),
                expected_output: "Evidence".into(),
                context: String::new(),
                constraints: String::new(),
            },
        )
        .unwrap()
        .id
}
fn execute(store: &Store, plan: &Plan, node: &str) -> String {
    let id = child(store, plan, node);
    store
        .attach_plan_worker(&plan.id, plan.version, node, &id)
        .unwrap();
    store
        .finish_subagent(
            &id,
            "completed",
            &json!({"summary":"Executor says complete"}),
        )
        .unwrap();
    store.reconcile_plan_worker(&id).unwrap();
    id
}
fn auditor(store: &Store, plan: &Plan, node: &str) -> String {
    let audit = store
        .completion_audits(&plan.id)
        .unwrap()
        .into_iter()
        .find(|a| a.node_id == node && a.version == plan.version)
        .unwrap();
    let worker = child(
        store,
        plan,
        &format!("Audit {} v{}", audit.id, plan.version),
    );
    store.attach_completion_auditor(&audit.id, &worker).unwrap();
    worker
}
fn observation(store: &Store, worker: &str) {
    store.event(worker,"tool_result",&json!({"call_id":"read","tool":"file_read","is_error":false,"result":{"content":"Actual report content","path":"report.txt","sha256":"fixture"}})).unwrap();
}
fn report(store: &Store, plan: &Plan, node: &str) -> CompletionReport {
    CompletionReport {
        verdict: "verified".into(),
        summary: "Deliverables independently observed".into(),
        checks: store
            .plan_evidence(&plan.id, plan.version)
            .unwrap()
            .iter()
            .filter(|e| node.is_empty() || e.input.node_id.as_deref() == Some(node))
            .map(|e| CompletionCheck {
                entry_id: e.input.id.clone(),
                state: EvidenceState::Verified,
                detail: "Read and compared against requirement".into(),
                call_ids: vec!["read".into()],
            })
            .collect(),
    }
}

#[test]
fn small_clear_work_adds_no_auditor() {
    let (_dir, store, plan) = setup(vec![node("small", &[])]);
    execute(&store, &plan, "small");
    store.settle_plan(&plan.id).unwrap();
    assert_eq!(store.plan(&plan.id).unwrap().status, "completed");
    assert!(store.completion_audits(&plan.id).unwrap().is_empty());
}

#[test]
fn dependencies_wait_for_independent_evidence_and_final_gate_covers_original_intent() {
    let (_dir, store, plan) = setup(vec![node("source", &[]), node("report", &["source"])]);
    execute(&store, &plan, "source");
    assert_eq!(store.plan(&plan.id).unwrap().nodes[0].status, "verifying");
    let dependent = child(&store, &plan, "dependent");
    assert!(
        store
            .attach_plan_worker(&plan.id, 1, "report", &dependent)
            .is_err()
    );
    store.settle_plan(&plan.id).unwrap();
    assert_eq!(store.plan(&plan.id).unwrap().status, "running");
    let audit = auditor(&store, &plan, "source");
    let pass = report(&store, &plan, "source");
    let error = store
        .validate_completion_report(&audit, &pass)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("Observation call read"),
        "invented evidence must identify the missing reference"
    );
    assert!(error.contains(&pass.checks[0].entry_id));
    assert!(error.contains("task_inspect recent=true"));
    observation(&store, &audit);
    store.finish_completion_audit(&audit, pass).unwrap();
    assert!(
        store
            .finish_completion_audit(&audit, report(&store, &plan, "source"))
            .is_err(),
        "audit results are applied once"
    );
    store
        .attach_plan_worker(&plan.id, 1, "report", &dependent)
        .unwrap();
    store
        .finish_subagent(&dependent, "completed", &json!({"summary":"Report ready"}))
        .unwrap();
    store.reconcile_plan_worker(&dependent).unwrap();
    store.settle_plan(&plan.id).unwrap();
    store.settle_plan(&plan.id).unwrap();
    assert_eq!(store.plan(&plan.id).unwrap().status, "running");
    assert_eq!(
        store.completion_audits(&plan.id).unwrap().len(),
        2,
        "scheduling must be idempotent"
    );
    let final_auditor = auditor(&store, &plan, "");
    observation(&store, &final_auditor);
    let mut incomplete = report(&store, &plan, "");
    incomplete.checks.retain(|c| c.entry_id != "owner_request");
    assert!(
        store
            .validate_completion_report(&final_auditor, &incomplete)
            .is_err()
    );
    store
        .finish_completion_audit(&final_auditor, report(&store, &plan, ""))
        .unwrap();
    store.settle_plan(&plan.id).unwrap();
    assert_eq!(store.plan(&plan.id).unwrap().status, "completed");
}

#[test]
fn audit_failure_blocks_only_dependants_and_does_not_replay_effects() {
    let (_dir, store, plan) = setup(vec![
        node("source", &[]),
        node("report", &["source"]),
        node("independent", &[]),
    ]);
    execute(&store, &plan, "source");
    let audit = auditor(&store, &plan, "source");
    store
        .finish_completion_audit(
            &audit,
            CompletionReport {
                verdict: "incomplete".into(),
                summary: "Required rows missing".into(),
                checks: vec![],
            },
        )
        .unwrap();
    assert_eq!(store.plan(&plan.id).unwrap().nodes[0].status, "blocked");
    let w = child(&store, &plan, "independent child");
    store
        .attach_plan_worker(&plan.id, 1, "independent", &w)
        .unwrap();
    assert!(
        store
            .attach_plan_worker(
                &plan.id,
                1,
                "report",
                &child(&store, &plan, "forbidden child")
            )
            .is_err()
    );
    assert_eq!(store.completion_audits(&plan.id).unwrap().len(), 1);
}

#[test]
fn explicit_single_stage_and_observed_long_work_receive_final_audits() {
    for explicit in [true, false] {
        let (_dir, store, plan) = setup(vec![node("large", &[])]);
        if explicit {
            store
                .put("completion_audit_required", &plan.id, &true)
                .unwrap();
        }
        let worker = child(&store, &plan, "large worker");
        store
            .attach_plan_worker(&plan.id, 1, "large", &worker)
            .unwrap();
        if !explicit {
            for i in 0..12 {
                store
                    .event(
                        &worker,
                        "tool_result",
                        &json!({"call_id":i.to_string(),"tool":"file_read","is_error":false}),
                    )
                    .unwrap();
            }
        }
        store
            .finish_subagent(&worker, "completed", &json!({"summary":"Done"}))
            .unwrap();
        store.reconcile_plan_worker(&worker).unwrap();
        store.settle_plan(&plan.id).unwrap();
        assert!(
            store
                .completion_audits(&plan.id)
                .unwrap()
                .iter()
                .any(|a| a.node_id.is_empty())
        );
        assert_eq!(store.plan(&plan.id).unwrap().status, "running");
    }
}

#[test]
fn plan_revision_and_cancellation_fence_old_auditors_and_preserve_evidence() {
    let (_dir, store, plan) = setup(vec![node("source", &[]), node("report", &["source"])]);
    execute(&store, &plan, "source");
    let a = auditor(&store, &plan, "source");
    observation(&store, &a);
    let pass = report(&store, &plan, "source");
    let (revision, workers) = store
        .revise_plan(
            &plan.id,
            1,
            &plan.task_id,
            PlanRevision {
                objective: &plan.objective,
                constraints: "",
                nodes: plan.nodes.iter().map(|n| n.spec.clone()).collect(),
                reason: "Review the same artifacts again",
            },
        )
        .unwrap();
    assert!(workers.contains(&a));
    assert!(store.finish_completion_audit(&a, pass).is_err());
    store
        .activate_plan_revision(&plan.id, revision.version)
        .unwrap();
    let newer = store.plan(&plan.id).unwrap();
    let b = auditor(&store, &newer, "source");
    assert!(
        store
            .cancel_plan(&plan.id, newer.version, &plan.task_id)
            .unwrap()
            .contains(&b)
    );
    assert!(store.check_completion_auditor(&b).is_err());
    assert!(!store.plan_evidence(&plan.id, 1).unwrap().is_empty());
}

#[test]
fn candidates_cannot_self_verify_or_overwrite_requirements_and_survive_restart() {
    let (dir, store, plan) = setup(vec![node("small", &[])]);
    let input = EvidenceInput {
        id: "note:format".into(),
        kind: EvidenceKind::Requirement,
        state: EvidenceState::Doubtful,
        statement: "Export format unresolved".into(),
        source: "User request did not specify it".into(),
        node_id: None,
        question: Some("CSV or PDF?".into()),
    };
    store
        .record_plan_evidence(&plan.id, 1, &plan.task_id, input.clone())
        .unwrap();
    assert!(
        store
            .record_plan_evidence(&plan.id, 1, &plan.task_id, input.clone())
            .is_err()
    );
    let mut fabricated = input.clone();
    fabricated.id = "note:fake".into();
    fabricated.state = EvidenceState::Verified;
    assert!(
        store
            .record_plan_evidence(&plan.id, 1, &plan.task_id, fabricated)
            .is_err()
    );
    let mut overwrite = input;
    overwrite.id = "owner_request".into();
    assert!(
        store
            .record_plan_evidence(&plan.id, 1, &plan.task_id, overwrite)
            .is_err()
    );
    drop(store);
    let reopened = Store::open(&dir.path().join("db")).unwrap();
    assert!(
        reopened
            .plan_evidence(&plan.id, 1)
            .unwrap()
            .iter()
            .any(|e| e.input.question.as_deref() == Some("CSV or PDF?"))
    );
}

#[test]
fn cross_plan_executors_and_peer_observations_cannot_be_used_as_auditors() {
    let (_dir, store, plan) = setup(vec![node("a", &[]), node("b", &["a"])]);
    let executor = execute(&store, &plan, "a");
    let pending = store.completion_audits(&plan.id).unwrap().remove(0);
    assert!(
        store
            .attach_completion_auditor(&pending.id, &executor)
            .is_err()
    );
    observation(&store, &executor);
    let auditor = auditor(&store, &plan, "a");
    assert!(
        store
            .validate_completion_report(&auditor, &report(&store, &plan, "a"))
            .is_err()
    );
    store.interrupt_unfinished().unwrap();
    assert_eq!(
        store.completion_audits(&plan.id).unwrap()[0].status,
        "interrupted"
    );
    assert!(store.check_completion_auditor(&auditor).is_err());
}

#[test]
fn schema_receipts_failed_reads_and_uncited_observations_cannot_prove_completion() {
    let (_dir, store, plan) = setup(vec![node("a", &[]), node("b", &["a"])]);
    execute(&store, &plan, "a");
    let auditor = auditor(&store, &plan, "a");
    let pass = report(&store, &plan, "a");
    store.event(&auditor, "tool_result", &json!({"call_id":"read","tool":"document_read","is_error":false,"result":{"code":"tool_schema_loaded"}})).unwrap();
    assert!(store.validate_completion_report(&auditor, &pass).is_err());
    observation(&store, &auditor);
    store.event(&auditor, "tool_result", &json!({"call_id":"read","tool":"file_read","is_error":true,"result":{"error":"File disappeared"}})).unwrap();
    assert!(
        store.validate_completion_report(&auditor, &pass).is_err(),
        "The latest receipt for an ID is authoritative"
    );
    observation(&store, &auditor);
    store.event(&auditor, "tool_result", &json!({"call_id":"answer","tool":"ask_user","is_error":false,"result":{"answer":"yes"}})).unwrap();
    let mut only_intent = pass;
    only_intent.checks[0].call_ids = vec!["answer".into()];
    assert!(
        store
            .validate_completion_report(&auditor, &only_intent)
            .is_err(),
        "An unrelated read does not make an owner answer proof of execution"
    );
}

#[test]
fn refuting_an_assumption_preserves_provenance_but_cannot_dismiss_a_requirement() {
    let (_dir, store, plan) = setup(vec![node("a", &[]), node("b", &["a"])]);
    store
        .record_plan_evidence(
            &plan.id,
            1,
            &plan.task_id,
            EvidenceInput {
                id: "note:assumption".into(),
                kind: EvidenceKind::Fact,
                state: EvidenceState::Doubtful,
                statement: "Input includes March rows".into(),
                source: "Unconfirmed inference".into(),
                node_id: Some("a".into()),
                question: None,
            },
        )
        .unwrap();
    execute(&store, &plan, "a");
    let auditor = auditor(&store, &plan, "a");
    observation(&store, &auditor);
    let mut pass = report(&store, &plan, "a");
    pass.checks
        .iter_mut()
        .find(|c| c.entry_id == "milestone:a")
        .unwrap()
        .state = EvidenceState::Refuted;
    assert!(store.validate_completion_report(&auditor, &pass).is_err());
    pass.checks
        .iter_mut()
        .find(|c| c.entry_id == "milestone:a")
        .unwrap()
        .state = EvidenceState::Verified;
    pass.checks
        .iter_mut()
        .find(|c| c.entry_id == "note:assumption")
        .unwrap()
        .state = EvidenceState::Refuted;
    store.finish_completion_audit(&auditor, pass).unwrap();
    let saved = store
        .plan_evidence(&plan.id, 1)
        .unwrap()
        .into_iter()
        .find(|e| e.input.id == "note:assumption")
        .unwrap();
    assert_eq!(saved.input.state, EvidenceState::Refuted);
    assert_eq!(saved.input.source, "Unconfirmed inference");
    assert!(!saved.observations.is_empty());
}

#[test]
fn decision_flag_adds_review_without_marking_delivery_verified() {
    let (_dir, store, plan) = setup(vec![node("small", &[])]);
    let worker = child(&store, &plan, "small");
    store
        .attach_plan_worker(&plan.id, plan.version, "small", &worker)
        .unwrap();
    store
        .put("decision_review_required", &worker, &true)
        .unwrap();
    store
        .finish_subagent(&worker, "completed", &json!({"summary":"Claimed delivery"}))
        .unwrap();
    store.reconcile_plan_worker(&worker).unwrap();
    assert_eq!(store.plan(&plan.id).unwrap().nodes[0].status, "verifying");
    assert_eq!(
        store.completion_audits(&plan.id).unwrap()[0].status,
        "queued"
    );
}
