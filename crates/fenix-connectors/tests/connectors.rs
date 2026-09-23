use fenix_connectors::{ConnectionInput, Connectors};
use serde_json::{Value, json};

fn input(provider: &str, credentials: Value, settings: Value, write: bool) -> ConnectionInput {
    ConnectionInput {
        id: None,
        provider: provider.into(),
        name: "Personal account".into(),
        credentials,
        settings,
        allow_write: write,
    }
}

#[tokio::test]
async fn vaults_keep_accounts_and_plaintext_secrets_separate() {
    let directory = tempfile::tempdir().unwrap();
    let alice = Connectors::open(&directory.path().join("alice")).unwrap();
    let bob = Connectors::open(&directory.path().join("bob")).unwrap();
    let saved = alice
        .save(input(
            "github",
            json!({"access_token":"test-only-private-token"}),
            json!({}),
            false,
        ))
        .unwrap();
    let id = saved["id"].as_str().unwrap();
    assert!(
        !alice
            .list()
            .unwrap()
            .to_string()
            .contains("test-only-private-token")
    );
    assert!(
        !std::fs::read_to_string(directory.path().join("alice/connectors/accounts.enc"))
            .unwrap()
            .contains("test-only-private-token")
    );
    assert!(bob.describe(id, "repositories").is_err());
    assert!(bob.forget(id).is_err());
    assert!(
        bob.invoke(id, "test", &json!({}), directory.path())
            .await
            .is_err()
    );
    drop(alice);
    let reopened = Connectors::open(&directory.path().join("alice")).unwrap();
    assert_eq!(reopened.list().unwrap()[0]["id"], id);
    reopened.forget(id).unwrap();
    assert_eq!(reopened.list().unwrap(), json!([]));
}

#[tokio::test]
async fn sqlite_queries_bind_data_and_cannot_escape_or_write_without_permission() {
    let directory = tempfile::tempdir().unwrap();
    let work = directory.path().join("workspace");
    std::fs::create_dir(&work).unwrap();
    let db = rusqlite::Connection::open(work.join("reports.sqlite")).unwrap();
    db.execute_batch("CREATE TABLE reports(id INTEGER, title TEXT); INSERT INTO reports VALUES (1,'Report A'),(2,'Report B');").unwrap();
    drop(db);
    let connectors = Connectors::open(&directory.path().join("state")).unwrap();
    let saved = connectors
        .save(input(
            "sqlite",
            json!({}),
            json!({"path":"reports.sqlite"}),
            false,
        ))
        .unwrap();
    let id = saved["id"].as_str().unwrap();
    let result = connectors
        .invoke(
            id,
            "query",
            &json!({"sql":"SELECT title FROM reports WHERE id=?1","parameters":[2]}),
            &work,
        )
        .await
        .unwrap();
    assert_eq!(result["rows"][0]["title"], "Report B");
    for sql in [
        "DELETE FROM reports",
        "ATTACH DATABASE '/etc/passwd' AS other",
        "PRAGMA writable_schema=ON",
    ] {
        assert!(
            connectors
                .invoke(id, "query", &json!({"sql":sql}), &work)
                .await
                .is_err(),
            "{sql}"
        );
    }
    let bad = connectors
        .save(input(
            "sqlite",
            json!({}),
            json!({"path":"../outside.sqlite"}),
            false,
        ))
        .unwrap();
    assert!(
        connectors
            .test(bad["id"].as_str().unwrap(), &work)
            .await
            .is_err()
    );
    std::os::unix::fs::symlink(work.join("reports.sqlite"), work.join("alias.sqlite")).unwrap();
    let bad = connectors
        .save(input(
            "sqlite",
            json!({}),
            json!({"path":"alias.sqlite"}),
            false,
        ))
        .unwrap();
    assert!(
        connectors
            .test(bad["id"].as_str().unwrap(), &work)
            .await
            .is_err()
    );
    assert_eq!(
        connectors.test(id, &work).await.unwrap()["state"],
        "connected"
    );
}

#[tokio::test]
async fn oauth_has_pkce_single_owner_state_and_a_fixed_callback_path() {
    let directory = tempfile::tempdir().unwrap();
    let connectors = Connectors::open(directory.path()).unwrap();
    let saved = connectors
        .save(input(
            "google",
            json!({"client_id":"fixture-client","client_secret":"fixture-secret"}),
            json!({}),
            false,
        ))
        .unwrap();
    let id = saved["id"].as_str().unwrap();
    let start = connectors
        .oauth_start(id, "https://fenix.example/api/connectors/oauth/callback")
        .await
        .unwrap();
    let url = reqwest::Url::parse(start["url"].as_str().unwrap()).unwrap();
    let query: std::collections::HashMap<_, _> = url.query_pairs().collect();
    assert_eq!(url.host_str(), Some("accounts.google.com"));
    assert_eq!(query["code_challenge_method"], "S256");
    assert_eq!(query["access_type"], "offline");
    assert!(!start.to_string().contains("fixture-secret"));
    assert!(
        connectors
            .oauth_start(id, "https://evil.example/not-the-callback")
            .await
            .is_err()
    );
    assert!(
        connectors
            .oauth_finish("wrong-state", "code")
            .await
            .is_err()
    );
    let other = Connectors::open(&directory.path().join("other")).unwrap();
    assert!(other.oauth_finish(&query["state"], "code").await.is_err());
}
