use super::*;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path, query_param},
};

fn account(connectors: &Connectors, services: &[&str], write: bool) -> String {
    connectors
        .save(ConnectionInput {
            id: None,
            provider: "google".into(),
            name: "Work".into(),
            credentials: json!({"access_token":"private-test-token"}),
            settings: json!({"services":services}),
            allow_write: write,
        })
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned()
}
async fn setup() -> (tempfile::TempDir, MockServer, Connectors) {
    let root = tempfile::tempdir().unwrap();
    let server = MockServer::start().await;
    let mut connectors = Connectors::open(root.path()).unwrap();
    connectors.test_origin = Some(server.uri());
    (root, server, connectors)
}

#[test]
fn permissions_are_service_specific_and_legacy_connections_keep_their_surface() {
    let root = tempfile::tempdir().unwrap();
    let connectors = Connectors::open(root.path()).unwrap();
    let id = account(&connectors, &["sheets"], false);
    let c = connectors.connection(&id).unwrap();
    let read = scopes(&c).unwrap();
    assert!(read.contains("spreadsheets.readonly"));
    assert!(!read.contains("gmail") && !read.contains("/drive"));
    assert!(ensure_selected(&c, "gmail").is_err());
    let write_id = account(&connectors, &["tasks", "forms", "meet"], true);
    let write = scopes(&connectors.connection(&write_id).unwrap()).unwrap();
    assert!(
        write.contains("/tasks ")
            && write.contains("/forms.body ")
            && write.contains("/meetings.space.created")
    );
    assert!(!write.contains("gmail") && !write.contains("cloud-platform"));
    assert_eq!(selected(&json!({})).unwrap().len(), 6);
    assert!(validate_settings(&json!({"services":[]})).is_err());
    assert!(validate_settings(&json!({"services":["gmail","gmail"]})).is_err());
    assert!(validate_settings(&json!({"services":["admin"]})).is_err());
    let custom = account(&connectors, &["gmail"], false);
    let mut c = connectors.connection(&custom).unwrap();
    c.settings["scopes"] = json!("https://www.googleapis.com/auth/gmail.metadata");
    assert_eq!(
        scopes(&c).unwrap(),
        "https://www.googleapis.com/auth/gmail.metadata"
    );
}

#[tokio::test]
async fn preview_never_authenticates_or_writes_and_service_gates_apply() {
    let (root, server, connectors) = setup().await;
    let id = account(&connectors, &["tasks"], false);
    let mut c = connectors.connection(&id).unwrap();
    c.credentials = json!({}); // Dry-run must not require or refresh a token.
    connectors.update(&c).unwrap();
    let args = json!({"service":"tasks","method":"POST","path":"/lists/list1/tasks","body":{"title":"Prepare report"},"dry_run":true});
    let preview = connectors
        .invoke(&id, "request", &args, root.path())
        .await
        .unwrap();
    assert_eq!(preview["executed"], false);
    assert_eq!(
        preview["url"],
        "https://tasks.googleapis.com/tasks/v1/lists/list1/tasks"
    );
    assert_eq!(connectors.effect(&id, "request", &args).unwrap(), "read");
    let mut execute = args.clone();
    execute["dry_run"] = json!(false);
    assert!(
        connectors
            .invoke(&id, "request", &execute, root.path())
            .await
            .is_err()
    );
    for bad in [
        json!({"service":"gmail","path":"/users/me/profile","dry_run":true}),
        json!({"service":"tasks","path":"//evil.test","dry_run":true}),
        json!({"service":"tasks","path":"/lists","dry_run":"true"}),
    ] {
        assert!(
            connectors
                .invoke(&id, "request", &bad, root.path())
                .await
                .is_err()
        );
    }
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn discovery_is_anonymous_cached_and_scoped_to_known_services() {
    let (root, server, connectors) = setup().await;
    let id = account(&connectors, &["drive"], false);
    Mock::given(method("GET")).and(path("/$discovery/rest")).and(query_param("version", "v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"name":"gmail","version":"v1","revision":"fixture","resources":{"users":{"resources":{"messages":{"methods":{"get":{"id":"gmail.users.messages.get","httpMethod":"GET","path":"gmail/v1/users/{userId}/messages/{id}","response":{"$ref":"Message"},"scopes":["gmail.readonly"]}}}}}},"schemas":{"Message":{"type":"object","properties":{"payload":{"$ref":"Part"}}},"Part":{"type":"object"}}})))
        .expect(1).mount(&server).await;
    let schema = connectors
        .invoke(
            &id,
            "workspace_schema",
            &json!({"service":"gmail","method_id":"gmail.users.messages.get"}),
            root.path(),
        )
        .await
        .unwrap();
    assert_eq!(schema["method"]["httpMethod"], "GET");
    assert_eq!(schema["schemas"]["Message"]["type"], "object");
    assert!(
        schema["referenced_schemas"]
            .as_array()
            .unwrap()
            .contains(&json!("Part"))
    );
    let part = connectors
        .invoke(
            &id,
            "workspace_schema",
            &json!({"service":"gmail","schema_name":"Part"}),
            root.path(),
        )
        .await
        .unwrap();
    assert_eq!(part["schema"]["type"], "object");
    assert!(
        connectors
            .invoke(
                &id,
                "workspace_schema",
                &json!({"service":"https://evil.test"}),
                root.path()
            )
            .await
            .is_err()
    );
    assert!(
        connectors
            .invoke(
                &id,
                "workspace_schema",
                &json!({"service":"gmail","method_id":"missing"}),
                root.path()
            )
            .await
            .is_err()
    );
    assert!(
        !server.received_requests().await.unwrap()[0]
            .headers
            .contains_key("authorization")
    );
}

#[tokio::test]
async fn pagination_reports_continuations_and_never_repeats_a_write() {
    let (root, server, connectors) = setup().await;
    let id = account(&connectors, &["gmail"], false);
    Mock::given(method("GET"))
        .and(path("/gmail/v1/users/me/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"messages":[{"id":"one"}],"nextPageToken":"next"})),
        )
        .with_priority(10)
        .mount(&server)
        .await;
    Mock::given(query_param("pageToken", "next"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"messages":[{"id":"two"}]})))
        .with_priority(1)
        .mount(&server)
        .await;
    let partial = connectors
        .invoke(
            &id,
            "mail_list",
            &json!({"page_all":true,"page_limit":1}),
            root.path(),
        )
        .await
        .unwrap();
    assert_eq!(partial["complete"], false);
    assert_eq!(partial["next_page_token"], "next");
    let result = connectors
        .invoke(&id, "mail_list", &json!({"page_all":true}), root.path())
        .await
        .unwrap();
    assert_eq!(result["complete"], true);
    assert_eq!(result["pages"].as_array().unwrap().len(), 2);
    let before = server.received_requests().await.unwrap().len();
    assert!(connectors.invoke(&id, "request", &json!({"service":"gmail","method":"POST","path":"/users/me/messages/send","page_all":true}), root.path()).await.is_err());
    assert!(
        connectors
            .invoke(
                &id,
                "mail_list",
                &json!({"page_all":true,"page_limit":11}),
                root.path()
            )
            .await
            .is_err()
    );
    assert_eq!(before, server.received_requests().await.unwrap().len());
}

#[tokio::test]
async fn pagination_stops_on_repeated_tokens_and_reports_partial_provider_failure() {
    let (root, server, connectors) = setup().await;
    let id = account(&connectors, &["tasks"], false);
    Mock::given(method("GET"))
        .and(path("/tasks/v1/users/@me/lists"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"items":[],"nextPageToken":"repeat"})),
        )
        .mount(&server)
        .await;
    let result = connectors
        .invoke(
            &id,
            "request",
            &json!({"service":"tasks","path":"/users/@me/lists","page_all":true}),
            root.path(),
        )
        .await
        .unwrap();
    assert_eq!(result["complete"], false);
    assert_eq!(result["stop_reason"], "repeated_page_token");
    assert_eq!(server.received_requests().await.unwrap().len(), 2);
    server.reset().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"items":[1],"nextPageToken":"later"})),
        )
        .with_priority(10)
        .mount(&server)
        .await;
    Mock::given(query_param("pageToken", "later"))
        .respond_with(ResponseTemplate::new(429))
        .with_priority(1)
        .mount(&server)
        .await;
    let result = connectors
        .invoke(
            &id,
            "request",
            &json!({"service":"tasks","path":"/users/@me/lists","page_all":true}),
            root.path(),
        )
        .await
        .unwrap();
    assert_eq!(result["stop_reason"], "provider_error");
    assert_eq!(result["next_page_token"], "later");
    assert_eq!(result["pages"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn checks_do_not_claim_that_every_google_service_passed() {
    let (root, server, connectors) = setup().await;
    let id = account(&connectors, &["drive", "gmail", "docs"], false);
    Mock::given(path("/drive/v3/about"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"user":{"emailAddress":"user@example.test"}})),
        )
        .mount(&server)
        .await;
    Mock::given(path("/gmail/v1/users/me/profile"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;
    let result = connectors.test(&id, root.path()).await.unwrap();
    assert_eq!(result["state"], "needs_attention");
    assert_eq!(result["service_checks"]["drive"]["status"], "passed");
    assert_eq!(result["service_checks"]["gmail"]["status"], "failed");
    assert_eq!(result["service_checks"]["docs"]["status"], "not_checked");
    let docs = account(&connectors, &["docs"], false);
    assert_eq!(
        connectors.test(&docs, root.path()).await.unwrap()["state"],
        "unverified"
    );
    assert_eq!(
        connectors.list().unwrap()[0]["service_checks"],
        result["service_checks"]
    );
}

#[tokio::test]
async fn gmail_drafts_preserve_thread_headers_and_attach_only_workspace_files() {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    let (root, server, connectors) = setup().await;
    let id = account(&connectors, &["gmail"], true);
    std::fs::write(root.path().join("report.txt"), "Report attachment").unwrap();
    Mock::given(path("/gmail/v1/users/me/profile"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"emailAddress":"owner@example.test"})),
        )
        .mount(&server)
        .await;
    Mock::given(path("/gmail/v1/users/me/messages/m1")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"threadId":"thread1","payload":{"headers":[{"name":"Subject","value":"Project"},{"name":"Message-ID","value":"<message1@example.test>"},{"name":"References","value":"<older@example.test>"},{"name":"To","value":"unrequested@example.test"}]}}))).mount(&server).await;
    Mock::given(method("POST"))
        .and(path("/gmail/v1/users/me/drafts"))
        .and(header("authorization", "Bearer private-test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id":"draft1"})))
        .expect(1)
        .mount(&server)
        .await;
    let args = json!({"to":["recipient@example.test"],"bcc":["hidden@example.test"],"text":"Here is the report.","reply_to_message_id":"m1","attachments":["report.txt"]});
    let receipt = connectors
        .invoke(&id, "mail_compose", &args, root.path())
        .await
        .unwrap();
    assert_eq!(receipt["draft"], true);
    assert_eq!(receipt["sent"], false);
    let requests = server.received_requests().await.unwrap();
    let request: Value = serde_json::from_slice(&requests.last().unwrap().body).unwrap();
    assert_eq!(request["message"]["threadId"], "thread1");
    let mime = String::from_utf8(
        URL_SAFE_NO_PAD
            .decode(request["message"]["raw"].as_str().unwrap())
            .unwrap(),
    )
    .unwrap();
    assert!(mime.contains("In-Reply-To: <message1@example.test>"));
    assert!(mime.contains("References: <older@example.test> <message1@example.test>"));
    assert!(mime.contains("Bcc: hidden@example.test"));
    assert!(mime.contains("report.txt"));
    assert!(!mime.contains("unrequested@example.test"));
    let before = requests.len();
    for file in ["../outside", "/etc/passwd", ".env"] {
        let mut bad = args.clone();
        bad["attachments"] = json!([file]);
        assert!(
            connectors
                .invoke(&id, "mail_compose", &bad, root.path())
                .await
                .is_err()
        );
    }
    assert_eq!(before, server.received_requests().await.unwrap().len());
}

#[tokio::test]
async fn mail_preview_is_offline_and_sending_is_explicit_and_not_retried() {
    let (root, server, connectors) = setup().await;
    let readonly = account(&connectors, &["gmail"], false);
    let mut args =
        json!({"to":["recipient@example.test"],"subject":"Hello","text":"Message","dry_run":true});
    assert_eq!(
        connectors
            .invoke(&readonly, "mail_compose", &args, root.path())
            .await
            .unwrap()["executed"],
        false
    );
    assert!(server.received_requests().await.unwrap().is_empty());
    args["dry_run"] = json!(false);
    assert!(
        connectors
            .invoke(&readonly, "mail_compose", &args, root.path())
            .await
            .is_err()
    );
    let id = account(&connectors, &["gmail"], true);
    Mock::given(path("/gmail/v1/users/me/profile"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"emailAddress":"owner@example.test"})),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/gmail/v1/users/me/messages/send"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&server)
        .await;
    args["send"] = json!(true);
    assert!(
        connectors
            .invoke(&id, "mail_compose", &args, root.path())
            .await
            .is_err()
    );
    assert_eq!(
        server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .filter(|r| r.method.as_str() == "POST")
            .count(),
        1
    );
}

#[tokio::test]
async fn new_service_requests_use_fixed_endpoints_and_array_queries() {
    let (root, server, connectors) = setup().await;
    let id = account(
        &connectors,
        &["tasks", "people", "forms", "chat", "meet"],
        false,
    );
    for (service, relative, route) in [
        ("tasks", "/users/@me/lists", "/tasks/v1/users/@me/lists"),
        (
            "people",
            "/people/me/connections",
            "/v1/people/me/connections",
        ),
        ("forms", "/forms/f1", "/v1/forms/f1"),
        ("chat", "/spaces", "/v1/spaces"),
        ("meet", "/conferenceRecords", "/v2/conferenceRecords"),
    ] {
        Mock::given(method("GET"))
            .and(path(route))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok":true})))
            .expect(1)
            .mount(&server)
            .await;
        connectors
            .invoke(
                &id,
                "request",
                &json!({"service":service,"path":relative,"query":{"fields":["one","two"]}}),
                root.path(),
            )
            .await
            .unwrap();
    }
    let requests = server.received_requests().await.unwrap();
    assert_eq!(
        requests[0]
            .url
            .query_pairs()
            .filter(|(k, _)| k == "fields")
            .count(),
        2
    );
}

#[tokio::test]
async fn oauth_uses_selected_scopes_and_editing_does_not_expand_services() {
    let (root, _server, connectors) = setup().await;
    let id = connectors
        .save(ConnectionInput {
            id: None,
            provider: "google".into(),
            name: "Sheets".into(),
            credentials: json!({"client_id":"client","client_secret":"secret"}),
            settings: json!({"services":["sheets"]}),
            allow_write: false,
        })
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let start = connectors
        .oauth_start(&id, "https://fenix.example/api/connectors/oauth/callback")
        .await
        .unwrap();
    let url = reqwest::Url::parse(start["url"].as_str().unwrap()).unwrap();
    let scopes = url
        .query_pairs()
        .find(|(k, _)| k == "scope")
        .unwrap()
        .1
        .into_owned();
    assert!(scopes.contains("spreadsheets.readonly"));
    assert!(!scopes.contains("gmail") && !scopes.contains("/drive"));
    let state = url
        .query_pairs()
        .find(|(k, _)| k == "state")
        .unwrap()
        .1
        .into_owned();
    connectors
        .save(ConnectionInput {
            id: Some(id.clone()),
            provider: "google".into(),
            name: "Renamed".into(),
            credentials: json!({}),
            settings: json!({}),
            allow_write: false,
        })
        .unwrap();
    assert_eq!(
        connectors.connection(&id).unwrap().settings["services"],
        json!(["sheets"])
    );
    assert!(connectors.oauth_finish(&state, "old-code").await.is_err());
    let other = Connectors::open(&root.path().join("other")).unwrap();
    assert!(other.describe(&id, "workspace_schema").is_err());
}

#[tokio::test]
async fn drive_upload_previews_locally_and_posts_a_single_multipart_request() {
    let (root, server, connectors) = setup().await;
    let id = account(&connectors, &["drive"], true);
    std::fs::write(root.path().join("report.pdf"), b"%PDF-fixture").unwrap();
    let mut args = json!({"source":"report.pdf","folder_id":"folder1","mime_type":"application/pdf","dry_run":true});
    let preview = connectors
        .invoke(&id, "drive_upload", &args, root.path())
        .await
        .unwrap();
    assert_eq!(preview["executed"], false);
    assert!(
        preview["url"]
            .as_str()
            .unwrap()
            .starts_with("https://www.googleapis.com/upload/drive/v3/files?")
    );
    assert!(server.received_requests().await.unwrap().is_empty());
    Mock::given(method("POST"))
        .and(path("/upload/drive/v3/files"))
        .and(query_param("uploadType", "multipart"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"id":"uploaded1","name":"report.pdf"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    args["dry_run"] = json!(false);
    assert_eq!(
        connectors
            .invoke(&id, "drive_upload", &args, root.path())
            .await
            .unwrap()["id"],
        "uploaded1"
    );
    let requests = server.received_requests().await.unwrap();
    let body = String::from_utf8_lossy(&requests[0].body);
    assert!(
        body.contains("%PDF-fixture")
            && body.contains("folder1")
            && body.contains("application/pdf")
    );
    args["source"] = json!("../other-user/report.pdf");
    assert!(
        connectors
            .invoke(&id, "drive_upload", &args, root.path())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn pagination_budget_retains_the_first_unreturned_page_token() {
    let (root, server, connectors) = setup().await;
    let id = account(&connectors, &["tasks"], false);
    let payload = "a".repeat(1700 * 1024);
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"items":[payload],"nextPageToken":"page2"})),
        )
        .with_priority(10)
        .mount(&server)
        .await;
    Mock::given(query_param("pageToken", "page2"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"items":[payload],"nextPageToken":"page3"})),
        )
        .with_priority(1)
        .mount(&server)
        .await;
    let result = connectors
        .invoke(
            &id,
            "request",
            &json!({"service":"tasks","path":"/users/@me/lists","page_all":true}),
            root.path(),
        )
        .await
        .unwrap();
    assert_eq!(result["stop_reason"], "byte_limit");
    assert_eq!(result["next_page_token"], "page2");
    assert_eq!(result["pages"].as_array().unwrap().len(), 1);
}

#[tokio::test]
#[ignore = "Read-only live verification against Google's public Discovery endpoints"]
async fn live_google_discovery_matches_registered_services_and_scope_presets() {
    let root = tempfile::tempdir().unwrap();
    let connectors = Connectors::open(root.path()).unwrap();
    for s in SERVICES {
        let result = connectors
            .google_schema(&json!({"service":s.id}))
            .await
            .unwrap();
        assert!(result["total"].as_u64().unwrap() > 0);
        let document = connectors
            .discovery_cache
            .lock()
            .unwrap()
            .get(s.id)
            .unwrap()
            .1
            .clone();
        let scopes = document["auth"]["oauth2"]["scopes"].as_object().unwrap();
        for name in s.read.split_whitespace().chain(s.write.split_whitespace()) {
            assert!(
                scopes.contains_key(&format!("https://www.googleapis.com/auth/{name}")),
                "{}: unrecognized scope {name}",
                s.id
            );
        }
        println!(
            "discovery verified: {} {} revision {}",
            s.id, s.version, document["revision"]
        );
    }
}

#[tokio::test]
async fn reconnect_drops_old_refresh_identity_and_records_actual_grants() {
    let (_root, server, connectors) = setup().await;
    let id = connectors.save(ConnectionInput { id:None, provider:"google".into(), name:"Work".into(), credentials:json!({"client_id":"client","client_secret":"secret","access_token":"old-access","refresh_token":"old-refresh"}), settings:json!({"services":["gmail","tasks"]}), allow_write:false }).unwrap()["id"].as_str().unwrap().to_owned();
    let start = connectors
        .oauth_start(&id, "https://fenix.example/api/connectors/oauth/callback")
        .await
        .unwrap();
    let url = reqwest::Url::parse(start["url"].as_str().unwrap()).unwrap();
    let state = url
        .query_pairs()
        .find(|(k, _)| k == "state")
        .unwrap()
        .1
        .into_owned();
    Mock::given(method("POST")).and(path("/token")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"access_token":"new-access","expires_in":3600,"scope":"https://www.googleapis.com/auth/tasks.readonly"}))).expect(1).mount(&server).await;
    connectors
        .oauth_finish(&state, "fixture-code")
        .await
        .unwrap();
    let c = connectors.connection(&id).unwrap();
    assert!(c.credentials["refresh_token"].is_null());
    assert_eq!(c.credentials["access_token"], "new-access");
    assert_eq!(
        c.view()["granted_scopes"],
        json!(["https://www.googleapis.com/auth/tasks.readonly"])
    );
    assert_eq!(c.state, "authorized");
    assert!(c.checked_at.is_none());
    assert!(
        connectors
            .oauth_finish(&state, "fixture-code")
            .await
            .is_err()
    );
}

#[test]
fn legacy_connected_records_do_not_masquerade_as_service_verified() {
    let c: Connection = serde_json::from_value(json!({"id":"old","provider":"google","name":"Legacy","credentials":{},"settings":{},"allow_write":false,"checked_at":null,"state":"connected"})).unwrap();
    assert_eq!(c.view()["state"], "authorized");
    assert!(c.view()["service_checks"].is_null());
    assert_eq!(selected(&c.settings).unwrap().len(), 6);
}
