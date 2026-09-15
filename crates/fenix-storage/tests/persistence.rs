use fenix_core::{ContentBlock, Message, Role};
use fenix_storage::Store;
use serde_json::json;

#[test]
fn queue_does_not_inject_new_messages_into_an_active_turn() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("test.db")).unwrap();
    let a = store.enqueue("First request", "web", None).unwrap();
    assert_eq!(store.claim_next().unwrap().unwrap().id, a.id);
    let b = store
        .enqueue("Correction queued separately", "telegram", None)
        .unwrap();
    let call = Message {
        role: Role::Assistant,
        content: vec![ContentBlock::ToolCall {
            id: "call-a".into(),
            name: "test".into(),
            arguments: json!({}),
        }],
        provider_data: None,
    };
    store
        .append(Some(&a.id), &call, false, true, "agent")
        .unwrap();
    store
        .append(
            Some(&a.id),
            &Message {
                role: Role::Tool,
                content: vec![ContentBlock::ToolResult {
                    tool_call_id: "call-a".into(),
                    name: Some("test".into()),
                    content: json!({"ok":true}),
                    is_error: false,
                }],
                provider_data: None,
            },
            false,
            true,
            "tool",
        )
        .unwrap();
    store
        .append(
            Some(&a.id),
            &Message::text(Role::Assistant, "First result"),
            true,
            true,
            "agent",
        )
        .unwrap();
    assert!(
        !store
            .context(0)
            .unwrap()
            .iter()
            .any(|m| m.task_id.as_deref() == Some(&b.id))
    );
    store.set_task_status(&a.id, "completed", None).unwrap();
    assert_eq!(store.claim_next().unwrap().unwrap().id, b.id);
    let context = store.context(0).unwrap();
    assert_eq!(context.last().unwrap().text, "Correction queued separately");
    assert_eq!(context[context.len() - 2].text, "First result");
    assert!(context.windows(2).all(|m| m[0].seq < m[1].seq));
}

#[test]
fn restart_keeps_conversation_and_marks_work_interrupted() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let store = Store::open(&path).unwrap();
    let session = store.session_id().to_owned();
    let task = store.enqueue("Do work", "web", None).unwrap();
    store.claim_next().unwrap();
    let queued = store.enqueue("Later work", "web", None).unwrap();
    drop(store);
    let store = Store::open(&path).unwrap();
    assert_eq!(store.session_id(), session);
    assert_eq!(store.interrupt_unfinished().unwrap(), 2);
    for id in [&task.id, &queued.id] {
        assert_eq!(store.task(id).unwrap().status, "interrupted");
    }
    assert!(store.claim_next().unwrap().is_none());
    assert!(store.context(0).unwrap().is_empty());
    assert_eq!(store.conversation(None, 100).unwrap().len(), 2);
}

#[test]
fn lexical_memory_handles_accents_duplicates_corrections_and_deletions() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("test.db")).unwrap();
    let memory = store
        .save_memory(None, "Café", "Prefiero café peruano", "message-1")
        .unwrap();
    let duplicate = store
        .save_memory(
            None,
            "Another title",
            "  PREFIERO  café peruano ",
            "message-2",
        )
        .unwrap();
    assert_eq!(memory.id, duplicate.id);
    assert_eq!(duplicate.source, "message-1");
    assert_eq!(store.memories("cafe", 10).unwrap().len(), 1);
    assert!(store.memories("\" OR * ) --", 10).is_ok());
    store
        .save_memory(Some(&memory.id), "Tea", "Prefiero té verde", "message-3")
        .unwrap();
    assert!(store.memories("cafe", 10).unwrap().is_empty());
    assert_eq!(store.memories("verde", 10).unwrap().len(), 1);
    store.delete_memory(&memory.id).unwrap();
    assert!(store.memories("verde", 10).unwrap().is_empty());
}

#[test]
fn one_schedule_cannot_overlap_itself_and_outputs_are_recoverable() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("test.db")).unwrap();
    let task = store
        .enqueue("Daily job", "schedule", Some("daily"))
        .unwrap();
    assert!(
        store
            .enqueue("Daily job", "schedule", Some("daily"))
            .is_err()
    );
    store
        .event(
            &task.id,
            "tool_result",
            &json!({"call_id":"call-1","result":{"output":"áβ🙂 complete evidence"}}),
        )
        .unwrap();
    let first = store.tool_output(&task.id, "call-1", 0, 10).unwrap();
    let rest = store.tool_output(&task.id, "call-1", 10, 100).unwrap();
    let joined = format!(
        "{}{}",
        first["content"].as_str().unwrap(),
        rest["content"].as_str().unwrap()
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&joined).unwrap()["output"],
        "áβ🙂 complete evidence"
    );
    store.set_task_status(&task.id, "completed", None).unwrap();
    assert!(
        store
            .enqueue("Daily job", "schedule", Some("daily"))
            .is_ok()
    );
}

#[test]
fn evaluation_pagination_keeps_tasks_beyond_the_recent_ui_limit() {
    let dir = tempfile::tempdir().unwrap();
    let store = fenix_storage::Store::open(&dir.path().join("evaluation.sqlite3")).unwrap();
    let mut expected = Vec::new();
    for index in 0..205 {
        expected.push(
            store
                .enqueue(&format!("Task {index}"), "evaluation", None)
                .unwrap()
                .id,
        );
    }
    let mut ids = Vec::new();
    let mut cursor = 0;
    loop {
        let page = store.tasks_page(cursor, 64).unwrap();
        if page.is_empty() {
            break;
        }
        for (seq, task) in page {
            assert!(seq > cursor);
            cursor = seq;
            ids.push(task.id);
        }
    }
    assert_eq!(ids, expected);
    assert_eq!(store.tasks(1000).unwrap().len(), 200);
}
