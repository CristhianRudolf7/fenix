use fenix_storage::{
    Plan, PlanBrief, PlanNodeSpec, PlanReview, ReviewAssignment, ReviewReport, SpecialistRequest,
    Store, SubagentSpec, TaskBinding,
};
use serde_json::json;

fn setup() -> (tempfile::TempDir, Store, Plan) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("db")).unwrap();
    let task = store
        .enqueue_bound("Create a report", "web", None, &TaskBinding::default())
        .unwrap();
    let nodes = serde_json::from_value::<Vec<PlanNodeSpec>>(json!([
        {"id":"source","objective":"Read sources","expected_output":"Source evidence"},
        {"id":"report","objective":"Report","expected_output":"A report","depends_on":["source"]},
        {"id":"independent","objective":"Independent summary","expected_output":"Summary"}
    ]))
    .unwrap();
    let plan = store.create_plan(&task.id, "A report", "", nodes).unwrap();
    (dir, store, plan)
}

fn review(plan: &Plan, scope: &[&str]) -> PlanReview {
    PlanReview {
        id: String::new(),
        plan_id: plan.id.clone(),
        version: plan.version,
        round: 0,
        status: "queued".into(),
        node_ids: scope.iter().map(|s| s.to_string()).collect(),
        reason: "Check dependencies".into(),
        brief: PlanBrief::default(),
        assignments: vec![ReviewAssignment {
            role: "principal".into(),
            request: SpecialistRequest {
                specialty: "Coherence".into(),
                question: "Are dependencies consistent?".into(),
                node_ids: vec![],
                skills: vec![],
            },
            worker_id: None,
            report: None,
        }],
        coordinator_id: None,
        resolutions: vec![],
        previous_review_id: None,
        created_at: String::new(),
        updated_at: String::new(),
    }
}

fn child(store: &Store, plan: &Plan, objective: &str) -> String {
    store
        .create_subagent(
            &plan.task_id,
            &SubagentSpec {
                profile: "general".into(),
                objective: objective.into(),
                expected_output: "Evidence".into(),
                context: String::new(),
                constraints: String::new(),
            },
        )
        .unwrap()
        .id
}

#[test]
fn selective_gates_cover_dependants_but_leave_independent_work_available() {
    let (_dir, store, plan) = setup();
    let review = store.start_plan_review(review(&plan, &["source"])).unwrap();
    assert!(store.review_blocks_node(&plan, "source").unwrap());
    assert!(store.review_blocks_node(&plan, "report").unwrap());
    assert!(!store.review_blocks_node(&plan, "independent").unwrap());
    let source = child(&store, &plan, "Source work");
    assert!(
        store
            .attach_plan_worker(&plan.id, 1, "source", &source)
            .is_err()
    );
    let independent = child(&store, &plan, "Independent work");
    store
        .attach_plan_worker(&plan.id, 1, "independent", &independent)
        .unwrap();
    let reviewer = child(&store, &plan, "Review dependencies");
    store
        .attach_review_worker(&review.id, "principal", &reviewer)
        .unwrap();
    assert!(
        store
            .create_subagent(
                &reviewer,
                &SubagentSpec {
                    profile: "general".into(),
                    objective: "Nested reviewer".into(),
                    expected_output: "Review".into(),
                    context: String::new(),
                    constraints: String::new()
                }
            )
            .is_err()
    );
    store
        .record_review_report(
            &reviewer,
            serde_json::from_value(json!({"verdict":"pass","findings":[]})).unwrap(),
        )
        .unwrap();
    store
        .attach_plan_worker(&plan.id, 1, "source", &source)
        .unwrap();
}

#[test]
fn review_reports_cannot_claim_pass_with_blockers_or_outlive_the_plan() {
    let (_dir, store, plan) = setup();
    let review = store.start_plan_review(review(&plan, &[])).unwrap();
    let reviewer = child(&store, &plan, "Review full plan");
    store
        .attach_review_worker(&review.id, "principal", &reviewer)
        .unwrap();
    let contradiction:ReviewReport=serde_json::from_value(json!({"verdict":"pass","findings":[{"id":"missing","category":"intent","severity":"blocking","description":"Missing preference","evidence":"Owner did not specify a format"}]})).unwrap();
    assert!(
        store
            .validate_review_report(&reviewer, &contradiction)
            .is_err()
    );
    store.cancel_plan(&plan.id, 1, &plan.task_id).unwrap();
    assert!(
        store
            .record_review_report(
                &reviewer,
                serde_json::from_value(json!({"verdict":"pass","findings":[]})).unwrap()
            )
            .is_err()
    );
    assert_ne!(store.plan_review(&review.id).unwrap().status, "passed");
}

#[test]
fn restart_preserves_audit_and_marks_review_work_interrupted() {
    let (dir, store, plan) = setup();
    let review = store.start_plan_review(review(&plan, &[])).unwrap();
    let worker = child(&store, &plan, "Review before restart");
    store
        .attach_review_worker(&review.id, "principal", &worker)
        .unwrap();
    store
        .event(
            &worker,
            "model_response",
            &json!({"usage":{"input_tokens":10,"output_tokens":3,"total_tokens":13}}),
        )
        .unwrap();
    drop(store);
    let store = Store::open(&dir.path().join("db")).unwrap();
    store.interrupt_unfinished().unwrap();
    assert_eq!(store.plan_review(&review.id).unwrap().status, "interrupted");
    assert_eq!(
        store.plan_review_metrics(&plan.id).unwrap()[0]["reported_total_tokens"],
        13
    );
    assert!(store.check_review_worker(&worker).is_err());
}
