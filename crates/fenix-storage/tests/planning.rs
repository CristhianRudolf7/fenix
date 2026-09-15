use fenix_core::{Message, Role};
use fenix_storage::{PlanNodeSpec, Store, SubagentSpec, TaskBinding};
use serde_json::json;

fn node(id: &str, dependencies: &[&str]) -> PlanNodeSpec {
    PlanNodeSpec {
        id: id.into(),
        objective: format!("Complete {id}"),
        expected_output: format!("Evidence for {id}"),
        depends_on: dependencies.iter().map(|s| s.to_string()).collect(),
        skills: vec![],
    }
}
fn owner(store: &Store) -> String {
    store
        .enqueue_bound(
            "Prepare an office report",
            "web",
            None,
            &TaskBinding::default(),
        )
        .unwrap()
        .id
}
fn worker(store: &Store, parent: &str, label: &str) -> String {
    store
        .create_subagent(
            parent,
            &SubagentSpec {
                objective: label.into(),
                expected_output: "An evidenced outcome".into(),
                context: String::new(),
                constraints: String::new(),
            },
        )
        .unwrap()
        .id
}

#[test]
fn graph_validation_rejects_cycles_unknown_dependencies_and_duplicate_ids() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("db")).unwrap();
    let task = owner(&store);
    for nodes in [
        vec![node("a", &["b"]), node("b", &["a"])],
        vec![node("a", &["missing"])],
        vec![node("a", &[]), node("a", &[])],
    ] {
        assert!(store.create_plan(&task, "A report", "", nodes).is_err());
    }
    assert!(store.plans().unwrap().is_empty());
    let plan = store
        .create_plan(
            &task,
            "A report",
            "",
            vec![node("a", &[]), node("b", &["a"])],
        )
        .unwrap();
    let premature = worker(&store, &task, "premature");
    assert!(
        store
            .attach_plan_worker(&plan.id, 1, "b", &premature)
            .is_err()
    );
}

#[test]
fn revision_preserves_valid_completed_work_and_fences_changed_dependants() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("db")).unwrap();
    let task = owner(&store);
    let plan = store
        .create_plan(
            &task,
            "A report",
            "",
            vec![node("a", &[]), node("b", &["a"]), node("c", &["b"])],
        )
        .unwrap();
    let a = worker(&store, &task, "first");
    store.attach_plan_worker(&plan.id, 1, "a", &a).unwrap();
    store
        .finish_subagent(
            &a,
            "completed",
            &json!({"summary":"File saved","path":"report.csv"}),
        )
        .unwrap();
    assert!(store.reconcile_plan_worker(&a).unwrap());
    let b = worker(&store, &task, "second");
    store.attach_plan_worker(&plan.id, 1, "b", &b).unwrap();
    let mut revised = node("b", &["a"]);
    revised.objective = "Use the updated office template".into();
    let (revision, cancel) = store
        .revise_plan(
            &plan.id,
            1,
            &task,
            fenix_storage::PlanRevision {
                objective: "A report",
                constraints: "",
                nodes: vec![node("a", &[]), revised, node("c", &["b"])],
                reason: "Change the template",
            },
        )
        .unwrap();
    assert_eq!(cancel, vec![b.clone()]);
    assert_eq!(revision.nodes[0].status, "completed");
    assert_eq!(
        revision.nodes[0].result.as_ref().unwrap()["path"],
        "report.csv"
    );
    assert_eq!(revision.nodes[2].generation, 2);
    assert!(store.check_plan_worker(&b).is_err());
    store
        .finish_subagent(&b, "completed", &json!({"summary":"Late old result"}))
        .unwrap();
    assert!(store.reconcile_plan_worker(&b).unwrap());
    assert_eq!(store.plan(&plan.id).unwrap().nodes[1].status, "pending");
    assert!(
        store
            .bus_events(0, 200)
            .unwrap()
            .iter()
            .any(|e| e.topic == "plan.stale_result")
    );
    assert!(
        store
            .revise_plan(
                &plan.id,
                1,
                &task,
                fenix_storage::PlanRevision {
                    objective: "Old revision",
                    constraints: "",
                    nodes: vec![node("a", &[])],
                    reason: "stale edit"
                }
            )
            .is_err()
    );
    store.activate_plan_revision(&plan.id, 2).unwrap();
    assert_eq!(store.plan_history(&plan.id).unwrap().len(), 2);
    let newer = store.plan(&plan.id).unwrap();
    assert_eq!(newer.status, "running");
    assert!(
        newer.nodes[1]
            .result
            .as_ref()
            .unwrap()
            .to_string()
            .contains(&b)
    );
}

#[test]
fn edit_pause_preserves_finished_milestones_and_prior_effect_references() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("db")).unwrap();
    let task = owner(&store);
    let plan = store
        .create_plan(
            &task,
            "Prepare report",
            "",
            vec![node("a", &[]), node("b", &[])],
        )
        .unwrap();
    let a = worker(&store, &task, "first");
    let b = worker(&store, &task, "second");
    store.attach_plan_worker(&plan.id, 1, "a", &a).unwrap();
    store.attach_plan_worker(&plan.id, 1, "b", &b).unwrap();
    store
        .finish_subagent(&a, "completed", &json!({"summary":"Done"}))
        .unwrap();
    store.reconcile_plan_worker(&a).unwrap();
    let (paused, cancelled) = store
        .pause_plan_for_edit(&plan.id, &task, "Change the date")
        .unwrap();
    assert_eq!(cancelled, vec![b.clone()]);
    store.interrupt_unfinished().unwrap();
    assert_eq!(store.plan(&plan.id).unwrap().status, "interrupted");

    assert_eq!(paused.status, "paused");
    assert_eq!(paused.nodes[0].status, "completed");
    assert!(
        paused.nodes[1]
            .result
            .as_ref()
            .unwrap()
            .to_string()
            .contains(&b)
    );
    assert!(store.check_plan_worker(&b).is_err());
    assert!(
        store
            .attach_plan_worker(&plan.id, paused.version, "b", &b)
            .is_err()
    );
}

#[test]
fn event_delivery_survives_reopen_and_replay_without_duplicate_messages_or_rerouting() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("db");
    let store = Store::open(&path).unwrap();
    let task = owner(&store);
    store
        .set_task_destination(&task, "telegram", Some("original-bot:7"))
        .unwrap();
    store
        .set_task_destination(&task, "telegram", Some("replacement-bot:8"))
        .unwrap();
    store
        .publish_message_once(
            "stable-notice",
            Some(&task),
            &Message::text(Role::Assistant, "Report ready"),
            false,
            "agent",
        )
        .unwrap();
    store
        .publish_message_once(
            "stable-notice",
            Some(&task),
            &Message::text(Role::Assistant, "Report ready"),
            false,
            "agent",
        )
        .unwrap();
    let events = store.bus_events(0, 100).unwrap();
    let message = events
        .iter()
        .find(|e| e.topic == "message.created")
        .unwrap();
    assert_eq!(message.data["destination"][1], "original-bot:7");
    assert_eq!(message.data["session_id"], store.session_id());
    store
        .queue_event_delivery(message, "original-bot:7", "Report ready")
        .unwrap();
    store
        .queue_event_delivery(message, "original-bot:7", "Report ready")
        .unwrap();
    store.acknowledge_event("telegram", message.seq).unwrap();
    store.acknowledge_event("telegram", 0).unwrap();
    let seq = message.seq;
    assert_eq!(store.deliveries_for("original-bot:7").unwrap().len(), 1);
    assert!(
        store
            .deliveries_for("replacement-bot:8")
            .unwrap()
            .is_empty()
    );
    drop(store);
    let store = Store::open(&path).unwrap();
    assert_eq!(store.subscriber_cursor("telegram").unwrap(), seq);
    assert_eq!(
        store
            .conversation(None, 100)
            .unwrap()
            .iter()
            .filter(|m| m.text == "Report ready")
            .count(),
        1
    );
    let delivery = store.deliveries_for("original-bot:7").unwrap()[0].0;
    store.delivery_result(delivery, true).unwrap();
    assert!(store.deliveries_for("original-bot:7").unwrap().is_empty());
}

#[test]
fn waiting_questions_are_durable_and_only_explicit_answers_resolve_them() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("db");
    let store = Store::open(&path).unwrap();
    let task = owner(&store);
    store.save_question(&json!({"id":"question","task_id":task,"kind":"clarification","question":"Which date format?"})).unwrap();
    drop(store);
    let store = Store::open(&path).unwrap();
    assert_eq!(
        store.saved_questions(Some(&task)).unwrap()[0]["status"],
        "pending"
    );
    store
        .resolve_question("question", "answered", Some("ISO"))
        .unwrap();
    assert!(
        !store
            .resolve_question("question", "answered", Some("US"))
            .unwrap()
    );
    assert_eq!(
        store.saved_questions(Some(&task)).unwrap()[0]["answer"],
        "ISO"
    );
}

#[test]
fn sealed_task_context_keeps_concurrent_transcripts_contiguous() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("db")).unwrap();
    let a = owner(&store);
    let b = owner(&store);
    store.claim_next().unwrap();
    store.claim_next().unwrap();
    store
        .append(
            Some(&a),
            &Message::text(Role::Assistant, "A step"),
            true,
            true,
            "agent",
        )
        .unwrap();
    store
        .append(
            Some(&b),
            &Message::text(Role::Assistant, "B step"),
            true,
            true,
            "agent",
        )
        .unwrap();
    store.finish_master_context(&b, "completed", None).unwrap();
    store.finish_master_context(&a, "completed", None).unwrap();
    let ids = store
        .context(0)
        .unwrap()
        .into_iter()
        .map(|m| m.task_id.unwrap())
        .collect::<Vec<_>>();
    assert_eq!(ids, vec![b.clone(), b, a.clone(), a]);
}

#[test]
fn plan_evidence_reads_exclude_independent_branches_and_other_projects() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("db")).unwrap();
    let task = owner(&store);
    let plan = store
        .create_plan(
            &task,
            "A report",
            "",
            vec![node("a", &[]), node("b", &["a"]), node("private", &[])],
        )
        .unwrap();
    let a = worker(&store, &task, "first");
    let b = worker(&store, &task, "second");
    let private = worker(&store, &task, "independent");
    store.attach_plan_worker(&plan.id, 1, "a", &a).unwrap();
    store
        .attach_plan_worker(&plan.id, 1, "private", &private)
        .unwrap();
    store
        .finish_subagent(&a, "completed", &json!({"summary":"sources ready"}))
        .unwrap();
    store.reconcile_plan_worker(&a).unwrap();
    store.attach_plan_worker(&plan.id, 1, "b", &b).unwrap();
    assert_eq!(store.plan_evidence_sources(&b).unwrap(), vec![a]);
    assert!(store.plan_evidence_sources(&private).unwrap().is_empty());
    let unrelated = worker(&store, &owner(&store), "other plan");
    assert!(
        !store
            .plan_evidence_sources(&b)
            .unwrap()
            .contains(&unrelated)
    );
}

#[test]
fn slow_routing_cannot_overwrite_newer_conversation_focus() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("db")).unwrap();
    let a = owner(&store);
    let b = owner(&store);
    let project = store
        .save_project(
            None,
            &fenix_storage::ProjectInput {
                name: "Project A".into(),
                ..Default::default()
            },
        )
        .unwrap();
    store.set_task_focus(&b, None).unwrap();
    store.set_task_focus(&a, Some(&project.id)).unwrap();
    assert_eq!(store.conversation_focus().unwrap(), None);
}
