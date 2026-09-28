use fenix_storage::Store;
use serde_json::{Value, json};

fn save(store: &Store, task: &str, call: &str, tool: &str, result: Value) {
    store
        .event(
            task,
            "tool_result",
            &json!({"call_id":call,"tool":tool,"result":result}),
        )
        .unwrap();
}

#[test]
fn scalar_outputs_and_owner_stores_remain_independent() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("owner-a.db")).unwrap();
    let other = Store::open(&dir.path().join("owner-b.db")).unwrap();
    let task = store.enqueue("A", "web", None).unwrap();
    let other_task = other.enqueue("B", "web", None).unwrap();
    save(&store, &task.id, "numeric", "command_run", json!(7654321));
    let result = store
        .search_tool_outputs(&task.id, "7654321", None, 0, 3)
        .unwrap();
    assert_eq!(result["results"][0]["content"], "7654321");
    assert_eq!(
        store.tool_output(&task.id, "numeric", 0, 100).unwrap()["content"],
        "7654321"
    );
    assert_eq!(
        other
            .search_tool_outputs(&other_task.id, "7654321", None, 0, 3)
            .unwrap()["results"],
        json!([])
    );
    assert!(
        other
            .search_tool_outputs(&task.id, "7654321", None, 0, 3)
            .is_err()
    );
}

#[test]
fn retrieves_middle_evidence_with_exact_unicode_offsets_and_call_filter() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("state.db")).unwrap();
    let task = store.enqueue("Debug", "web", None).unwrap();
    let log = format!(
        "{}\nAUTHENTICATION_FAILURE sesión inválida\n{}",
        "🦀 start ".repeat(1200),
        "tail ".repeat(1000)
    );
    save(
        &store,
        &task.id,
        "build",
        "command_run",
        json!({"exit_code":1,"output":log}),
    );
    save(
        &store,
        &task.id,
        "other",
        "file_read",
        json!({"content":"authentication reference"}),
    );
    let result = store
        .search_tool_outputs(&task.id, "AUTHENTICATION_FAILURE", Some("build"), 0, 3)
        .unwrap();
    assert_eq!(result["index_complete"], true);
    let hits = result["results"].as_array().unwrap();
    assert!(!hits.is_empty());
    assert!(hits.iter().all(|h| h["call_id"] == "build"));
    let hit = hits
        .iter()
        .find(|h| {
            h["content"]
                .as_str()
                .unwrap()
                .contains("AUTHENTICATION_FAILURE")
        })
        .unwrap();
    assert!(hit["offset"].as_u64().unwrap() > 2000);
    let original = store
        .tool_output(
            &task.id,
            "build",
            hit["offset"].as_u64().unwrap() as usize,
            hit["length"].as_u64().unwrap() as usize,
        )
        .unwrap();
    assert_eq!(hit["content"], original["content"]);
}

#[test]
fn incremental_backfill_reports_gaps_excludes_inspection_and_survives_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    let store = Store::open(&path).unwrap();
    let task = store.enqueue("Old task", "web", None).unwrap();
    for n in 0..70 {
        save(
            &store,
            &task.id,
            &format!("call-{n}"),
            "command_run",
            json!({"output":format!("needle{n}")}),
        );
    }
    save(
        &store,
        &task.id,
        "inspection",
        "task_inspect",
        json!({"content":"recursivephantom"}),
    );
    let first = store
        .search_tool_outputs(&task.id, "needle69", None, 0, 3)
        .unwrap();
    assert_eq!(first["index_complete"], false);
    assert_eq!(first["results"], json!([]));
    drop(store);
    let store = Store::open(&path).unwrap();
    let second = store
        .search_tool_outputs(&task.id, "needle69", None, 0, 3)
        .unwrap();
    assert_eq!(second["index_complete"], true);
    assert_eq!(second["results"].as_array().unwrap().len(), 1);
    assert_eq!(
        store
            .search_tool_outputs(&task.id, "recursivephantom", None, 0, 3)
            .unwrap()["results"],
        json!([])
    );
    save(
        &store,
        &task.id,
        "new-call",
        "file_read",
        json!("newmarker"),
    );
    assert_eq!(
        store
            .search_tool_outputs(&task.id, "newmarker", None, 0, 3)
            .unwrap()["results"][0]["call_id"],
        "new-call"
    );
}

#[test]
fn scope_pagination_and_latest_call_evidence_are_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("state.db")).unwrap();
    let a = store.enqueue("A", "web", None).unwrap();
    let b = store.enqueue("B", "web", None).unwrap();
    save(
        &store,
        &b.id,
        "foreign",
        "file_read",
        json!("confidentialmarker"),
    );
    store
        .search_tool_outputs(&b.id, "confidentialmarker", None, 0, 3)
        .unwrap();
    for n in 0..5 {
        save(
            &store,
            &a.id,
            &format!("c{n}"),
            "file_read",
            json!("sharedword"),
        );
    }
    let first = store
        .search_tool_outputs(&a.id, "sharedword", None, 0, 2)
        .unwrap();
    let second = store
        .search_tool_outputs(&a.id, "sharedword", None, 2, 2)
        .unwrap();
    assert_eq!(first["next_offset"], 2);
    assert_ne!(
        first["results"][0]["call_id"],
        second["results"][0]["call_id"]
    );
    assert_eq!(
        store
            .search_tool_outputs(&a.id, "confidentialmarker", None, 0, 3)
            .unwrap()["results"],
        json!([])
    );
    save(&store, &a.id, "reused", "file_read", json!("oldmarker"));
    assert_eq!(
        store
            .search_tool_outputs(&a.id, "oldmarker", None, 0, 3)
            .unwrap()["results"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    save(
        &store,
        &a.id,
        "reused",
        "file_read",
        json!("replacementmarker"),
    );
    assert_eq!(
        store
            .search_tool_outputs(&a.id, "oldmarker", None, 0, 3)
            .unwrap()["results"],
        json!([])
    );
    assert_eq!(
        store
            .search_tool_outputs(&a.id, "replacementmarker", None, 0, 3)
            .unwrap()["results"][0]["call_id"],
        "reused"
    );
    assert!(
        store
            .search_tool_outputs(&a.id, "\"*()", None, 0, 3)
            .is_err()
    );
    assert!(
        store
            .search_tool_outputs(&a.id, "\" OR sharedword* NEAR(whatever)", None, 0, 3)
            .is_ok()
    );
}

#[test]
fn migration_indexes_existing_journal_without_rewriting_it_and_deletion_cleans_index() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    let store = Store::open(&path).unwrap();
    let task = store.enqueue("Legacy", "web", None).unwrap();
    save(
        &store,
        &task.id,
        "old-call",
        "file_read",
        json!({"text":"legacymarker"}),
    );
    let before = store.events(&task.id, 0, 100).unwrap();
    drop(store);
    let db = rusqlite::Connection::open(&path).unwrap();
    // Recreate the full v11 shape, including removal of later memory-review columns.
    db.execute_batch("DROP INDEX memory_review_pending; DROP INDEX memory_revision_record; ALTER TABLE memory_review_pairs DROP COLUMN first_revision; ALTER TABLE memory_review_pairs DROP COLUMN second_revision; ALTER TABLE memory_review_pairs DROP COLUMN assessment; ALTER TABLE memory_review_pairs DROP COLUMN attempts; ALTER TABLE memory_review_pairs DROP COLUMN review_after;").unwrap();
    db.execute_batch("DROP TRIGGER tool_output_search_delete; DROP TABLE tool_output_fts; DROP TABLE tool_output_index_cursors; DROP INDEX events_tool_output_call; PRAGMA user_version=11;").unwrap();
    drop(db);
    let store = Store::open(&path).unwrap();
    assert_eq!(
        store
            .search_tool_outputs(&task.id, "legacymarker", None, 0, 3)
            .unwrap()["results"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(store.events(&task.id, 0, 100).unwrap(), before);
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute("DELETE FROM events WHERE task_id=?1", [&task.id])
        .unwrap();
    assert_eq!(
        db.query_row("SELECT count(*) FROM tool_output_fts", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn recent_execution_evidence_is_bounded_scoped_and_excludes_transcripts() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("recent.db")).unwrap();
    let own = store.enqueue("Recover my task", "web", None).unwrap();
    let other = store.enqueue("Unrelated task", "web", None).unwrap();
    for n in 0..60 {
        save(
            &store,
            &own.id,
            &format!("own-{n}"),
            "command_run",
            json!({"output":n}),
        );
        save(
            &store,
            &other.id,
            &format!("other-{n}"),
            "command_run",
            json!({"output":"PRIVATE"}),
        );
    }
    store
        .event(
            &own.id,
            "model_response",
            &json!({"private_reasoning":"OMIT"}),
        )
        .unwrap();
    store
        .event(
            &own.id,
            "subagent_message",
            &json!({"provider_data":"OMIT"}),
        )
        .unwrap();
    save(
        &store,
        &own.id,
        "inspection",
        "task_inspect",
        json!({"nested":"OMIT"}),
    );
    let latest = store.recent_execution_events(&own.id, 2).unwrap();
    assert_eq!(latest.len(), 2);
    assert_eq!(latest[0]["data"]["call_id"], "own-58");
    assert_eq!(latest[1]["data"]["call_id"], "own-59");
    assert!(latest[0]["seq"].as_i64() < latest[1]["seq"].as_i64());
    assert!(!serde_json::to_string(&latest).unwrap().contains("PRIVATE"));
    assert_eq!(
        store.recent_execution_events(&own.id, 1000).unwrap().len(),
        50
    );
    assert!(
        store
            .recent_execution_events("unknown", 5)
            .unwrap()
            .is_empty()
    );
    // Existing forward pagination retains its original semantics.
    assert_eq!(
        store.events(&own.id, 0, 1).unwrap()[0]["data"]["call_id"],
        "own-0"
    );
}
