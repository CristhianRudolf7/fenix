use fenix_storage::{Store, SubagentSpec};
use serde_json::json;

fn spec() -> SubagentSpec {
    SubagentSpec {
        profile: "general".into(),
        objective: "Find the quarterly expenses".into(),
        expected_output: "A total with source file references".into(),
        context: "Only the finance folder is relevant".into(),
        constraints: "Do not edit any files".into(),
    }
}

#[test]
fn child_queue_is_independent_and_messages_stay_out_of_chat() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("test.db")).unwrap();
    let parent = store.enqueue("Summarize my expenses", "web", None).unwrap();
    assert_eq!(store.claim_next().unwrap().unwrap().id, parent.id);
    let child = store.create_subagent(&parent.id, &spec()).unwrap();
    let user = store
        .enqueue("What is the current date?", "telegram", None)
        .unwrap();
    assert_eq!(store.claim_next().unwrap().unwrap().id, user.id);
    assert!(store.claim_next().unwrap().is_none());
    assert_eq!(store.claim_subagent().unwrap().unwrap().id, child.id);
    assert!(store.claim_subagent().unwrap().is_none());
    assert_eq!(store.subagent(&child.id).unwrap().attempts, 0);
    assert_eq!(store.context(0).unwrap().len(), 2);
    assert_eq!(store.conversation(None, 100).unwrap().len(), 2);
    assert!(store.history_search("quarterly", 10).unwrap().is_empty());
}

#[test]
fn purpose_validation_deduplication_and_one_delegation_level_are_enforced() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("test.db")).unwrap();
    let parent = store.enqueue("Do work", "web", None).unwrap();
    assert!(store.create_subagent("missing", &spec()).is_err());
    let mut invalid = spec();
    invalid.expected_output = "  ".into();
    assert!(store.create_subagent(&parent.id, &invalid).is_err());
    invalid = spec();
    invalid.objective = "x".repeat(16_001);
    assert!(store.create_subagent(&parent.id, &invalid).is_err());
    let child = store.create_subagent(&parent.id, &spec()).unwrap();
    let mut padded = spec();
    padded.objective = format!(" \n{}  ", padded.objective);
    assert_eq!(
        store.create_subagent(&parent.id, &padded).unwrap().id,
        child.id
    );
    assert!(store.create_subagent(&child.id, &spec()).is_err());
    store
        .finish_subagent(&child.id, "failed", &json!({"error":"No source files"}))
        .unwrap();
    assert_eq!(
        store.create_subagent(&parent.id, &spec()).unwrap().id,
        child.id
    );
    assert_eq!(store.subagents(Some(&parent.id), 100).unwrap().len(), 1);
}

#[test]
fn attempts_are_bounded_and_terminal_results_cannot_be_overwritten() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("test.db")).unwrap();
    let parent = store.enqueue("Do work", "web", None).unwrap();
    let child = store.create_subagent(&parent.id, &spec()).unwrap();
    assert!(store.start_subagent_attempt(&child.id).is_err());
    store.claim_subagent().unwrap();
    assert_eq!(store.start_subagent_attempt(&child.id).unwrap(), 1);
    store.set_task_status(&child.id, "waiting", None).unwrap();
    assert_eq!(store.subagent(&child.id).unwrap().status, "waiting");
    assert!(store.start_subagent_attempt(&child.id).is_err());
    store.set_task_status(&child.id, "running", None).unwrap();
    assert_eq!(store.start_subagent_attempt(&child.id).unwrap(), 2);
    assert_eq!(store.start_subagent_attempt(&child.id).unwrap(), 3);
    assert!(store.start_subagent_attempt(&child.id).is_err());
    assert!(
        store
            .finish_subagent(&child.id, "running", &json!({}))
            .is_err()
    );
    store
        .finish_subagent(&child.id, "cancelled", &json!({"reason":"User cancelled"}))
        .unwrap();
    store
        .finish_subagent(&child.id, "completed", &json!({"total":50}))
        .unwrap();
    store.set_task_status(&child.id, "running", None).unwrap();
    let saved = store.subagent(&child.id).unwrap();
    assert_eq!(saved.status, "cancelled");
    assert_eq!(saved.attempts, 3);
    assert_eq!(saved.result.unwrap()["reason"], "User cancelled");
    assert_eq!(store.task(&child.id).unwrap().status, "cancelled");
    assert!(store.start_subagent_attempt(&child.id).is_err());
}

#[test]
fn terminal_report_is_once_only_hidden_and_waits_behind_user_messages() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("test.db")).unwrap();
    let parent = store.enqueue("Do work", "web", None).unwrap();
    store.claim_next().unwrap();
    let child = store.create_subagent(&parent.id, &spec()).unwrap();
    // A queued or reopened worker has no terminal result to report yet.
    assert!(store.enqueue_subagent_report(&child.id).unwrap().is_none());
    store
        .finish_subagent(
            &child.id,
            "completed",
            &json!({"total":50,"sources":["expenses.csv"]}),
        )
        .unwrap();
    assert_eq!(store.unreported_subagents(10).unwrap().len(), 1);
    let report = store.enqueue_subagent_report(&child.id).unwrap().unwrap();
    assert_eq!(report.origin, "subagent_report");
    assert!(
        report
            .request
            .contains("delegated task evidence, not a new user request")
    );
    assert!(report.request.contains("expenses.csv"));
    assert!(store.enqueue_subagent_report(&child.id).unwrap().is_none());
    assert!(store.unreported_subagents(10).unwrap().is_empty());
    assert_eq!(store.conversation(None, 100).unwrap().len(), 1);
    assert_eq!(store.context(0).unwrap().len(), 1);
    let user = store.enqueue("Another question", "web", None).unwrap();
    assert_eq!(store.claim_next().unwrap().unwrap().id, user.id);
    assert_eq!(store.claim_next().unwrap().unwrap().id, report.id);
    assert_eq!(
        store.context(0).unwrap().last().unwrap().origin,
        "subagent_report"
    );
    assert_eq!(store.conversation(None, 100).unwrap().len(), 2);
}

#[test]
fn active_capacity_does_not_prevent_retrieving_an_existing_child() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("test.db")).unwrap();
    let parent = store.enqueue("Do work", "web", None).unwrap();
    let original = store.create_subagent(&parent.id, &spec()).unwrap();
    for index in 1..32 {
        let mut next = spec();
        next.objective = format!("Review file {index}");
        store.create_subagent(&parent.id, &next).unwrap();
    }
    assert_eq!(
        store.create_subagent(&parent.id, &spec()).unwrap().id,
        original.id
    );
    let mut overflow = spec();
    overflow.objective = "Review the final file".into();
    assert!(store.create_subagent(&parent.id, &overflow).is_err());
    store
        .finish_subagent(
            &original.id,
            "blocked",
            &json!({"missing":"File is absent"}),
        )
        .unwrap();
    assert!(store.create_subagent(&parent.id, &overflow).is_ok());
}

#[test]
fn upgrade_preserves_existing_data_and_restart_does_not_resume_children() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let legacy = rusqlite::Connection::open(&path).unwrap();
    legacy
        .execute_batch(include_str!("../migrations/001_initial.sql"))
        .unwrap();
    legacy
        .execute(
            "INSERT INTO metadata VALUES ('session_id','legacy-session')",
            [],
        )
        .unwrap();
    legacy.execute("INSERT INTO tasks(id,session_id,request,origin,status,created_at,updated_at) VALUES ('legacy','legacy-session','Existing request','web','completed','old','old')", []).unwrap();
    drop(legacy);
    let store = Store::open(&path).unwrap();
    assert_eq!(store.session_id(), "legacy-session");
    assert_eq!(store.task("legacy").unwrap().request, "Existing request");
    let child = store.create_subagent("legacy", &spec()).unwrap();
    store.claim_subagent().unwrap();
    store.start_subagent_attempt(&child.id).unwrap();
    store
        .event(
            &child.id,
            "tool_result",
            &json!({"output":"Evidence survives"}),
        )
        .unwrap();
    drop(store);
    let store = Store::open(&path).unwrap();
    assert_eq!(store.interrupt_unfinished().unwrap(), 1);
    assert!(store.claim_subagent().unwrap().is_none());
    assert!(store.claim_next().unwrap().is_none());
    let saved = store.subagent(&child.id).unwrap();
    assert_eq!(saved.status, "interrupted");
    assert_eq!(saved.attempts, 1);
    assert_eq!(saved.result.unwrap()["status"], "interrupted");
    assert_eq!(
        store.events(&child.id, 0, 100).unwrap()[0]["data"]["output"],
        "Evidence survives"
    );
    assert_eq!(store.unreported_subagents(10).unwrap()[0].id, child.id);
    assert_eq!(store.task("legacy").unwrap().status, "completed");
    drop(store);
    let connection = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(
        connection
            .pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
            .unwrap(),
        13
    );
    connection.pragma_update(None, "user_version", 14).unwrap();
    drop(connection);
    assert!(Store::open(&path).is_err());
}

#[test]
fn followups_are_durable_idempotent_scoped_and_preserve_attempts() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("test.db")).unwrap();
    let parent = store.enqueue("Do the work", "web", None).unwrap();
    store.claim_next().unwrap();
    let child = store.create_subagent(&parent.id, &spec()).unwrap();
    store.claim_subagent().unwrap();
    store.start_subagent_attempt(&child.id).unwrap();
    let sent = store
        .send_subagent_input(&parent.id, &child.id, "Keep CSV compatibility")
        .unwrap();
    assert_eq!(
        store
            .send_subagent_input(&parent.id, &child.id, "Keep CSV compatibility")
            .unwrap()["message_id"],
        sent["message_id"]
    );
    assert!(
        store
            .send_subagent_input(&child.id, &child.id, "Delegate again")
            .is_err()
    );
    let result = json!({"status":"completed","summary":"Ready"});
    assert!(
        !store
            .finish_subagent_unless_input(&child.id, "completed", &result)
            .unwrap()
    );
    let inputs = store.pending_subagent_inputs(&child.id).unwrap();
    assert_eq!(inputs.len(), 1);
    store
        .save_subagent_context(
            &child.id,
            &[fenix_core::Message::user("Remember CSV compatibility")],
            inputs[0].0,
        )
        .unwrap();
    assert!(store.pending_subagent_inputs(&child.id).unwrap().is_empty());
    assert!(
        store
            .finish_subagent_unless_input(&child.id, "completed", &result)
            .unwrap()
    );
    assert_eq!(
        store
            .send_subagent_input(&parent.id, &child.id, "Also summarize the changed files")
            .unwrap()["reopened"],
        true
    );
    assert_eq!(store.subagent(&child.id).unwrap().attempts, 1);
    assert!(
        store
            .claim_subagent_excluding(std::slice::from_ref(&child.id))
            .unwrap()
            .is_none()
    );
    assert_eq!(store.claim_subagent().unwrap().unwrap().id, child.id);
    assert!(
        store
            .get::<Vec<fenix_core::Message>>("subagent_context", &child.id)
            .unwrap()
            .unwrap()[0]
            .text_content()
            .contains("CSV")
    );
    store.start_subagent_attempt(&child.id).unwrap();
    store.start_subagent_attempt(&child.id).unwrap();
    store
        .finish_subagent(&child.id, "failed", &json!({"error":"Unreachable"}))
        .unwrap();
    assert!(
        store
            .send_subagent_input(&parent.id, &child.id, "Try again")
            .is_err()
    );
}

#[test]
fn security_profile_is_durable_distinct_and_backward_compatible() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profiles.db");
    let store = Store::open(&path).unwrap();
    let parent = store.enqueue("Audit a fixture", "web", None).unwrap();
    let general = store.create_subagent(&parent.id, &spec()).unwrap();
    let mut security = spec();
    security.profile = "security".into();
    let child = store.create_subagent(&parent.id, &security).unwrap();
    assert_ne!(child.id, general.id);
    assert_eq!(
        child.id,
        store.create_subagent(&parent.id, &security).unwrap().id
    );
    security.profile = "administrator".into();
    assert!(store.create_subagent(&parent.id, &security).is_err());
    security.profile = "security".into();
    security.constraints.clear();
    assert!(store.create_subagent(&parent.id, &security).is_err());
    let mut old = serde_json::to_value(spec()).unwrap();
    old.as_object_mut().unwrap().remove("profile");
    assert_eq!(
        serde_json::from_value::<SubagentSpec>(old).unwrap().profile,
        "general"
    );
    drop(store);
    assert_eq!(
        Store::open(&path)
            .unwrap()
            .subagent(&child.id)
            .unwrap()
            .spec
            .profile,
        "security"
    );
}

#[test]
fn security_handoff_is_compact_and_full_evidence_stays_recoverable() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("db")).unwrap();
    let parent = store.enqueue("Audit fixture", "web", None).unwrap();
    let mut assignment = spec();
    assignment.profile = "security".into();
    let child = store.create_subagent(&parent.id, &assignment).unwrap();
    let findings=(0..12).map(|i|json!({"id":format!("F{i}"),"title":"Finding","status":if i==11 {"confirmed"}else{"suspected"},"severity":if i==11 {"critical"}else{"low"},"evidence":"LONG_RAW_EVIDENCE".repeat(1000)})).collect::<Vec<_>>();
    let result = json!({"summary":"Audit finished with uncertainties","security_report":{"scope":"fixture","findings":findings,"coverage":[{"area":"API","status":"not_tested","evidence":"No endpoint"}]}});
    store
        .finish_subagent(&child.id, "completed", &result)
        .unwrap();
    let report = store.enqueue_subagent_report(&child.id).unwrap().unwrap();
    assert!(!report.request.contains("LONG_RAW_EVIDENCE"));
    assert!(report.request.contains("F11"));
    assert!(report.request.contains("finding_counts"));
    assert!(report.request.contains("not_tested"));
    assert!(report.request.contains(&child.id));
    assert!(report.request.len() < 10000);
    assert_eq!(store.subagent(&child.id).unwrap().result.unwrap(), result);
}
