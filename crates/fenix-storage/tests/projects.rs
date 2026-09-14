use fenix_core::{Message, Role};
use fenix_storage::{Project, ProjectInput, Store, SubagentSpec, TaskBinding};
use serde_json::json;

fn project(store: &Store, name: &str) -> Project {
    store
        .save_project(
            None,
            &ProjectInput {
                name: name.into(),
                workspace: Some(format!("/tmp/{name}")),
                ..ProjectInput::default()
            },
        )
        .unwrap()
}

fn binding(project: &Project) -> TaskBinding {
    TaskBinding {
        project_id: Some(project.id.clone()),
        mode: "project".into(),
        ..TaskBinding::default()
    }
}

fn spec() -> SubagentSpec {
    SubagentSpec {
        objective: "Inspect login latency".into(),
        expected_output: "Report measured bottlenecks".into(),
        context: String::new(),
        constraints: "Do not change unrelated files".into(),
    }
}

#[test]
fn routing_hints_are_captured_at_arrival_and_scope_is_immutable() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("test.db")).unwrap();
    let a = project(&store, "A");
    let b = project(&store, "B");
    store.set_conversation_focus(Some(&a.id)).unwrap();
    let original = store.enqueue("Inspect A", "web", None).unwrap();
    store.bind_task(&original.id, &binding(&a)).unwrap();
    let later = store
        .enqueue_referenced("Continue this", "telegram", None, Some(&original.id))
        .unwrap();
    store.set_conversation_focus(Some(&b.id)).unwrap();
    let captured = store.task_binding(&later.id).unwrap();
    assert!(!captured.bound);
    assert_eq!(captured.focus_project_id.as_deref(), Some(a.id.as_str()));
    assert_eq!(
        captured.reply_to_task_id.as_deref(),
        Some(original.id.as_str())
    );
    store.bind_task(&later.id, &binding(&a)).unwrap();
    store.bind_task(&later.id, &binding(&a)).unwrap();
    assert!(store.bind_task(&later.id, &binding(&b)).is_err());
    assert!(store.bind_task(&later.id, &TaskBinding::default()).is_err());
    assert_eq!(
        store.task_binding(&later.id).unwrap().workspace.as_deref(),
        Some("/tmp/A")
    );
    assert_eq!(
        store.conversation_focus().unwrap().as_deref(),
        Some(b.id.as_str())
    );
    assert!(
        store
            .enqueue_referenced("Unknown reply", "web", None, Some("missing"))
            .is_err()
    );
    assert!(store.task_binding("missing").is_err());
}

#[test]
fn project_aliases_are_unique_and_context_updates_use_versions() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("test.db")).unwrap();
    let a = store
        .save_project(
            None,
            &ProjectInput {
                name: "Accounting".into(),
                aliases: vec!["Finance".into()],
                ..ProjectInput::default()
            },
        )
        .unwrap();
    assert!(
        store
            .save_project(
                None,
                &ProjectInput {
                    name: "finance".into(),
                    ..ProjectInput::default()
                }
            )
            .is_err()
    );
    assert_eq!(store.projects().unwrap().len(), 1);
    let updated = store
        .update_project_brief(&a.id, "Use monthly reporting", a.context_version)
        .unwrap();
    assert_eq!(updated.context_version, a.context_version + 1);
    assert!(
        store
            .update_project_brief(&a.id, "Late old summary", a.context_version)
            .is_err()
    );
    assert_eq!(store.project(&a.id).unwrap().brief, "Use monthly reporting");
    assert!(
        store
            .save_project(
                None,
                &ProjectInput {
                    name: "Relative".into(),
                    workspace: Some("relative/path".into()),
                    ..ProjectInput::default()
                }
            )
            .is_err()
    );
}

#[test]
fn memory_ids_fts_updates_deletion_and_deduplication_respect_scope() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("test.db")).unwrap();
    let a = project(&store, "A");
    let b = project(&store, "B");
    let global = store
        .save_memory(None, "Preference", "Use PostgreSQL", "general")
        .unwrap();
    let ma = store
        .save_memory_scoped(None, "Database", "Use PostgreSQL", "A", Some(&a.id), None)
        .unwrap();
    let mb = store
        .save_memory_scoped(None, "Database", "Use PostgreSQL", "B", Some(&b.id), None)
        .unwrap();
    assert_ne!(ma.id, mb.id);
    assert_ne!(ma.id, global.id);
    assert_eq!(
        store
            .save_memory_scoped(
                None,
                "Same fact",
                " USE   postgresql ",
                "A",
                Some(&a.id),
                None
            )
            .unwrap()
            .id,
        ma.id
    );
    assert_eq!(
        store
            .memories_scoped("postgresql", 10, Some(&a.id), false)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        store
            .memories_scoped("postgresql", 10, Some(&a.id), true)
            .unwrap()
            .len(),
        2
    );
    assert_eq!(store.memories_scoped("", 10, None, false).unwrap().len(), 1);
    assert!(
        store
            .memories_scoped(&mb.id, 10, Some(&a.id), true)
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .memories_scoped(&global.id, 10, Some(&a.id), false)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        store
            .memories_scoped(&global.id, 10, Some(&a.id), true)
            .unwrap()[0]
            .id,
        global.id
    );
    assert!(
        store
            .save_memory_scoped(Some(&mb.id), "Hijack", "Use SQLite", "A", Some(&a.id), None)
            .is_err()
    );
    assert!(store.delete_memory_scoped(&mb.id, Some(&a.id)).is_err());
    assert!(store.delete_memory_scoped(&global.id, Some(&a.id)).is_err());
    store
        .save_memory_scoped(
            Some(&ma.id),
            "Database",
            "Use SQLite",
            "A",
            Some(&a.id),
            None,
        )
        .unwrap();
    assert!(
        store
            .memories_scoped("postgresql", 10, Some(&a.id), false)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        store
            .memories_scoped("postgresql", 10, Some(&b.id), false)
            .unwrap()
            .len(),
        1
    );
    store.delete_memory_scoped(&ma.id, Some(&a.id)).unwrap();
    assert!(
        store
            .memories_scoped("sqlite", 10, Some(&a.id), false)
            .unwrap()
            .is_empty()
    );
    assert_eq!(store.memories("postgresql", 10).unwrap().len(), 2);
}

#[test]
fn memory_provenance_cannot_reference_a_different_project_or_unresolved_task() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("test.db")).unwrap();
    let a = project(&store, "A");
    let b = project(&store, "B");
    let task = store.enqueue("Remember A", "web", None).unwrap();
    assert!(
        store
            .save_memory_scoped(
                None,
                "Decision",
                "PostgreSQL",
                "test",
                Some(&a.id),
                Some(&task.id)
            )
            .is_err()
    );
    store.bind_task(&task.id, &binding(&a)).unwrap();
    assert!(
        store
            .save_memory_scoped(
                None,
                "Decision",
                "PostgreSQL",
                "test",
                Some(&b.id),
                Some(&task.id)
            )
            .is_err()
    );
    assert!(
        store
            .save_memory_scoped(None, "Decision", "PostgreSQL", "test", None, Some(&task.id))
            .is_err()
    );
    let memory = store
        .save_memory_scoped(
            None,
            "Decision",
            "PostgreSQL",
            "test",
            Some(&a.id),
            Some(&task.id),
        )
        .unwrap();
    assert_eq!(memory.task_id, Some(task.id));
}

#[test]
fn history_scope_applies_to_exact_ids_neighbors_context_and_search() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("test.db")).unwrap();
    let a = project(&store, "A");
    let b = project(&store, "B");
    let ta = store.enqueue("Shared keyword A", "web", None).unwrap();
    store.bind_task(&ta.id, &binding(&a)).unwrap();
    store.claim_next().unwrap();
    let ma = store
        .append(
            Some(&ta.id),
            &Message::text(Role::Assistant, "Shared result A"),
            true,
            true,
            "agent",
        )
        .unwrap();
    let tb = store.enqueue("Shared keyword B", "telegram", None).unwrap();
    store.bind_task(&tb.id, &binding(&b)).unwrap();
    store.claim_next().unwrap();
    let mb = store
        .append(
            Some(&tb.id),
            &Message::text(Role::Assistant, "Shared result B"),
            true,
            true,
            "agent",
        )
        .unwrap();
    let general = store
        .enqueue("Shared general preference", "web", None)
        .unwrap();
    store
        .bind_task(&general.id, &TaskBinding::default())
        .unwrap();
    store.claim_next().unwrap();
    let coordination = store
        .enqueue("Shared compare A and B", "web", None)
        .unwrap();
    store
        .bind_task(
            &coordination.id,
            &TaskBinding {
                mode: "coordination".into(),
                read_project_ids: vec![a.id.clone(), b.id.clone()],
                ..TaskBinding::default()
            },
        )
        .unwrap();
    store.claim_next().unwrap();
    assert_eq!(store.context_scoped(0, Some(&a.id)).unwrap().len(), 2);
    assert_eq!(store.context_scoped(0, None).unwrap().len(), 1);
    assert_eq!(
        store
            .history_search_scoped("shared", 20, Some(&a.id))
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        store
            .history_search_scoped("shared", 20, None)
            .unwrap()
            .len(),
        1
    );
    assert!(store.history_read_scoped(&mb, 15, Some(&a.id)).is_err());
    assert!(store.history_read_scoped(&ma, 15, None).is_err());
    let neighbors = store.history_read_scoped(&ma, 15, Some(&a.id)).unwrap();
    assert_eq!(neighbors.len(), 2);
    assert!(
        neighbors
            .iter()
            .all(|message| message.task_id.as_deref() == Some(&ta.id))
    );
    assert_eq!(store.tasks_for_project(&a.id, 20).unwrap().len(), 1);
    let recent = store.recent_context_scoped(Some(&a.id), 1).unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].id, ma);
    assert_eq!(store.recent_context_scoped(None, 100).unwrap().len(), 1);
    assert_eq!(
        store.task_user_message(&ta.id).unwrap().unwrap().text,
        "Shared keyword A"
    );
    assert!(store.task_user_message("missing").unwrap().is_none());
    assert_eq!(store.conversation(None, 100).unwrap().len(), 6);
}

#[test]
fn child_and_late_report_keep_scope_and_workspace_without_changing_focus() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("test.db")).unwrap();
    let a = project(&store, "A");
    let b = project(&store, "B");
    let parent = store.enqueue("Inspect A", "web", None).unwrap();
    store.bind_task(&parent.id, &binding(&a)).unwrap();
    store
        .save_project(
            Some(&a.id),
            &ProjectInput {
                name: "A".into(),
                workspace: Some("/tmp/A-new".into()),
                ..ProjectInput::default()
            },
        )
        .unwrap();
    let child = store.create_subagent(&parent.id, &spec()).unwrap();
    let snapshot = store.task_binding(&child.id).unwrap();
    assert_eq!(snapshot.project_id.as_deref(), Some(a.id.as_str()));
    assert_eq!(snapshot.workspace.as_deref(), Some("/tmp/A"));
    assert_eq!(snapshot.context_version, a.context_version);
    assert!(
        store
            .create_subagent_in_project(&parent.id, &spec(), Some(&b.id))
            .is_err()
    );
    assert!(
        store
            .create_subagent_in_project(&parent.id, &spec(), None)
            .is_err()
    );
    assert!(store.create_subagent(&child.id, &spec()).is_err());
    store.set_conversation_focus(Some(&b.id)).unwrap();
    store
        .finish_subagent(&child.id, "completed", &json!({"result":"A is ready"}))
        .unwrap();
    let report = store.enqueue_subagent_report(&child.id).unwrap().unwrap();
    assert_eq!(store.task_binding(&report.id).unwrap(), snapshot);
    assert_eq!(
        store.conversation_focus().unwrap().as_deref(),
        Some(b.id.as_str())
    );
    store
        .queue_delivery_for_task("A completed", Some(&report.id))
        .unwrap();
    let (seq, _) = &store.deliveries().unwrap()[0];
    assert_eq!(store.delivery_task(*seq).unwrap(), Some(report.id));
}

#[test]
fn coordination_children_have_separate_dedup_scopes_and_no_read_scope_expansion() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("test.db")).unwrap();
    let a = project(&store, "A");
    let b = project(&store, "B");
    let c = project(&store, "C");
    let parent = store.enqueue("Inspect both projects", "web", None).unwrap();
    store
        .bind_task(
            &parent.id,
            &TaskBinding {
                mode: "coordination".into(),
                read_project_ids: vec![a.id.clone(), b.id.clone()],
                ..TaskBinding::default()
            },
        )
        .unwrap();
    let ca = store
        .create_subagent_in_project(&parent.id, &spec(), Some(&a.id))
        .unwrap();
    let cb = store
        .create_subagent_in_project(&parent.id, &spec(), Some(&b.id))
        .unwrap();
    assert_ne!(ca.id, cb.id);
    assert_eq!(
        store
            .create_subagent_in_project(&parent.id, &spec(), Some(&a.id))
            .unwrap()
            .id,
        ca.id
    );
    assert_eq!(
        store.task_binding(&ca.id).unwrap().read_project_ids,
        vec![a.id]
    );
    assert_eq!(
        store.task_binding(&cb.id).unwrap().read_project_ids,
        vec![b.id]
    );
    assert!(
        store
            .create_subagent_in_project(&parent.id, &spec(), Some(&c.id))
            .is_err()
    );
    assert!(
        store
            .create_subagent_in_project(&parent.id, &spec(), None)
            .is_err()
    );
}

#[test]
fn version_two_migration_preserves_memories_indexes_tasks_and_child_reports() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let legacy = rusqlite::Connection::open(&path).unwrap();
    legacy
        .execute_batch(include_str!("../migrations/001_initial.sql"))
        .unwrap();
    legacy
        .execute_batch(include_str!("../migrations/002_subagents.sql"))
        .unwrap();
    legacy.execute("INSERT INTO memories VALUES ('m1','Database','Use PostgreSQL','old','use postgresql','old','old')",[]).unwrap();
    legacy
        .execute(
            "INSERT INTO memory_fts(id,title,content) VALUES ('m1','Database','Use PostgreSQL')",
            [],
        )
        .unwrap();
    legacy.execute("INSERT INTO tasks(id,session_id,request,origin,status,created_at,updated_at) VALUES ('p','session','Request','web','completed','old','old'),('c','session','Child','subagent','completed','old','old'),('r','session','Report','subagent_report','queued','old','old')",[]).unwrap();
    legacy.execute("INSERT INTO subagents(id,parent_task_id,spec,attempts,result,report_task_id) VALUES ('c','p',?1,2,'{\"ok\":true}','r')",[serde_json::to_string(&spec()).unwrap()]).unwrap();
    legacy
        .execute(
            "INSERT INTO outbox(text,created_at) VALUES ('Old delivery','old')",
            [],
        )
        .unwrap();
    drop(legacy);
    let store = Store::open(&path).unwrap();
    assert_eq!(
        store
            .memories_scoped("postgresql", 10, None, false)
            .unwrap()[0]
            .id,
        "m1"
    );
    let child = store.subagent("c").unwrap();
    assert!(store.task_binding("p").unwrap().bound);
    assert!(store.task_binding("c").unwrap().bound);
    assert_eq!(store.task_binding("r").unwrap().mode, "general");
    assert_eq!(child.attempts, 2);
    assert_eq!(child.result.unwrap()["ok"], true);
    assert!(store.enqueue_subagent_report("c").unwrap().is_none());
    assert_eq!(store.task("r").unwrap().status, "queued");
    assert_eq!(
        store
            .delivery_task(store.deliveries().unwrap()[0].0)
            .unwrap(),
        None
    );
    let a = project(&store, "A");
    assert!(
        store
            .save_memory_scoped(None, "Database", "Use PostgreSQL", "new", Some(&a.id), None)
            .is_ok()
    );
    drop(store);
    let reopened = Store::open(&path).unwrap();
    assert_eq!(reopened.projects().unwrap().len(), 1);
    assert_eq!(reopened.memories("postgresql", 10).unwrap().len(), 2);
}

#[test]
fn competing_bindings_and_brief_updates_cannot_overwrite_the_winner() {
    use std::sync::{Arc, Barrier};
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(&dir.path().join("test.db")).unwrap());
    let a = project(&store, "A");
    let b = project(&store, "B");
    let task = store.enqueue("Ambiguous request", "web", None).unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let results = std::thread::scope(|scope| {
        let one = scope.spawn(|| {
            barrier.wait();
            store.bind_task(&task.id, &binding(&a))
        });
        let two = scope.spawn(|| {
            barrier.wait();
            store.bind_task(&task.id, &binding(&b))
        });
        [one.join().unwrap(), two.join().unwrap()]
    });
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    let stored = store.task_binding(&task.id).unwrap();
    assert!(stored.bound);
    assert!(stored.project_id == Some(a.id.clone()) || stored.project_id == Some(b.id.clone()));
    let barrier = Arc::new(Barrier::new(2));
    let results = std::thread::scope(|scope| {
        let one = scope.spawn(|| {
            barrier.wait();
            store.update_project_brief(&a.id, "First completion", a.context_version)
        });
        let two = scope.spawn(|| {
            barrier.wait();
            store.update_project_brief(&a.id, "Second completion", a.context_version)
        });
        [one.join().unwrap(), two.join().unwrap()]
    });
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        store.project(&a.id).unwrap().context_version,
        a.context_version + 1
    );
}

#[test]
fn scheduled_enqueue_is_atomic_and_preserves_foreground_focus() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("test.db")).unwrap();
    let a = project(&store, "A");
    let b = project(&store, "B");
    store.set_conversation_focus(Some(&b.id)).unwrap();
    let invalid = TaskBinding {
        project_id: Some("missing".into()),
        mode: "project".into(),
        ..TaskBinding::default()
    };
    assert!(
        store
            .enqueue_bound("Broken schedule", "schedule", Some("daily"), &invalid)
            .is_err()
    );
    assert!(store.claim_next().unwrap().is_none());
    assert!(store.conversation(None, 100).unwrap().is_empty());
    let task = store
        .enqueue_bound("Daily A review", "schedule", Some("daily"), &binding(&a))
        .unwrap();
    let claimed = store.claim_next().unwrap().unwrap();
    assert_eq!(claimed.id, task.id);
    let saved = store.task_binding(&claimed.id).unwrap();
    assert!(saved.bound);
    assert_eq!(saved.project_id, Some(a.id));
    assert_eq!(saved.workspace.as_deref(), Some("/tmp/A"));
    assert_eq!(store.conversation_focus().unwrap(), Some(b.id));
}

#[test]
fn reminder_notification_keeps_scope_and_reply_provenance_without_queue_work() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("test.db")).unwrap();
    let a = project(&store, "A");
    let b = project(&store, "B");
    store.set_conversation_focus(Some(&b.id)).unwrap();
    let notice = store
        .record_notification_task("Confidential A reminder", &binding(&a))
        .unwrap();
    assert_eq!(notice.status, "completed");
    assert_eq!(notice.origin, "reminder");
    assert!(store.claim_next().unwrap().is_none());
    assert!(store.conversation(None, 100).unwrap().is_empty());
    let message = store
        .append(
            Some(&notice.id),
            &Message::text(Role::Assistant, "Confidential A reminder"),
            true,
            false,
            "agent",
        )
        .unwrap();
    assert!(
        store
            .history_search_scoped("confidential", 10, None)
            .unwrap()
            .is_empty()
    );
    assert!(store.history_read_scoped(&message, 10, None).is_err());
    assert!(
        store
            .history_search_scoped("confidential", 10, Some(&b.id))
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        store
            .history_search_scoped("confidential", 10, Some(&a.id))
            .unwrap()
            .len(),
        1
    );
    assert!(store.context_scoped(0, Some(&a.id)).unwrap().is_empty());
    let reply = store
        .enqueue_referenced("Complete it", "telegram", None, Some(&notice.id))
        .unwrap();
    assert_eq!(
        store.task_binding(&reply.id).unwrap().reply_to_task_id,
        Some(notice.id)
    );
    assert_eq!(store.conversation_focus().unwrap(), Some(b.id));
}

#[test]
fn child_without_project_workspace_inherits_the_parents_pinned_fallback() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("test.db")).unwrap();
    let a = project(&store, "A");
    let office = store
        .save_project(
            None,
            &ProjectInput {
                name: "Office".into(),
                ..ProjectInput::default()
            },
        )
        .unwrap();
    let parent = store
        .enqueue_bound(
            "Report A and review Office",
            "web",
            None,
            &TaskBinding {
                mode: "coordination".into(),
                read_project_ids: vec![a.id.clone(), office.id.clone()],
                workspace: Some("/tmp/original-workspace".into()),
                ..TaskBinding::default()
            },
        )
        .unwrap();
    let child = store
        .create_subagent_in_project(&parent.id, &spec(), Some(&office.id))
        .unwrap();
    assert_eq!(
        store.task_binding(&child.id).unwrap().workspace.as_deref(),
        Some("/tmp/original-workspace")
    );
    assert_eq!(
        store.task_binding(&child.id).unwrap().project_id,
        Some(office.id)
    );
}
