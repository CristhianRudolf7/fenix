use fenix_storage::Store;
#[test]
fn reopen_preserves_session_tasks_and_values() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("data/state.sqlite3");
    let session;
    let task;
    {
        let store = Store::open(&path).unwrap();
        session = store.session_id().to_owned();
        task = store.enqueue("Keep this request", "web", None).unwrap().id;
        store.put("preferences", "language", &"es").unwrap();
    }
    let store = Store::open(&path).unwrap();
    assert_eq!(store.session_id(), session);
    assert_eq!(store.task(&task).unwrap().request, "Keep this request");
    assert_eq!(
        store
            .get::<String>("preferences", "language")
            .unwrap()
            .unwrap(),
        "es"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            std::fs::metadata(path.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }
}
#[test]
fn database_hard_links_are_rejected() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("state.sqlite3");
    drop(Store::open(&path).unwrap());
    let linked = temporary.path().join("linked.sqlite3");
    std::fs::hard_link(&path, &linked).unwrap();
    assert!(Store::open(&linked).is_err());
}
#[cfg(unix)]
#[test]
fn database_symbolic_links_are_rejected() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("state.sqlite3");
    drop(Store::open(&path).unwrap());
    let linked = temporary.path().join("linked.sqlite3");
    std::os::unix::fs::symlink(&path, &linked).unwrap();
    assert!(Store::open(&linked).is_err());
}

#[test]
fn retiring_voice_preserves_fast_configuration_and_usage() {
    let temporary = tempfile::tempdir().unwrap();
    let store = Store::open(&temporary.path().join("state.sqlite3")).unwrap();
    store.put("live", "settings", &"old-settings").unwrap();
    store.put("model_roles", "fast", &"current-settings").unwrap();
    store.put("credentials", "live", &"fixture-key").unwrap();
    store.put("live_usage", "usage-one", &serde_json::json!({"tokens":42})).unwrap();
    store.migrate_voice_model_records().unwrap();
    store.migrate_voice_model_records().unwrap();
    assert_eq!(store.get::<String>("model_roles", "fast").unwrap().unwrap(), "current-settings");
    assert_eq!(store.get::<String>("credentials", "fast").unwrap().unwrap(), "fixture-key");
    assert!(store.get::<String>("live", "settings").unwrap().is_none());
    assert!(store.get::<String>("credentials", "live").unwrap().is_none());
    assert!(store.records::<serde_json::Value>("live_usage").unwrap().is_empty());
    assert_eq!(store.records::<serde_json::Value>("fast_usage").unwrap(), vec![serde_json::json!({"tokens":42})]);
}
