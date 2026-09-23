use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use fenix_agent::{Runtime, config::Settings};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;
async fn request(
    app: &axum::Router,
    path: &str,
    body: Option<Value>,
    origin: &str,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .uri(path)
        .header("host", "127.0.0.1:4173");
    if body.is_some() {
        request = request
            .method("POST")
            .header("origin", origin)
            .header("content-type", "application/json");
    }
    let response = app
        .clone()
        .oneshot(
            request
                .body(body.map_or(Body::empty(), |b| Body::from(b.to_string())))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}
#[tokio::test]
async fn owner_routes_cover_connections_flows_runs_and_csrf() {
    let dir = tempfile::tempdir().unwrap();
    let mut apps = vec![];
    for name in ["alice", "bob"] {
        let root = dir.path().join(name);
        std::fs::create_dir_all(root.join("workspace")).unwrap();
        let runtime = Runtime::open(&root.join("state"), Some(&root.join("skills")))
            .await
            .unwrap();
        runtime
            .store
            .put(
                "settings",
                "active",
                &Settings::initial(root.join("workspace")),
            )
            .unwrap();
        apps.push(fenix_server::router(runtime, 4173));
    }
    let origin = "http://127.0.0.1:4173";
    assert_eq!(
        request(
            &apps[0],
            "/api/connectors/oauth/callback",
            Some(json!({"state":"unknown","code":"unused"})),
            origin
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        request(
            &apps[0],
            "/api/connectors/oauth/callback",
            Some(json!({"error":"access_denied"})),
            "https://evil.example"
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let (_, declined) = request(
        &apps[0],
        "/api/connectors/oauth/callback",
        Some(json!({"error":"access_denied"})),
        origin,
    )
    .await;
    assert_eq!(declined["redirect"], "/integrations?connection=declined");
    let (status, catalog) = request(&apps[0], "/api/connectors", None, origin).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(catalog["catalog"].as_array().unwrap().len(), 12);
    let (_,connection)=request(&apps[0],"/api/connectors",Some(json!({"provider":"github","name":"Work","credentials":{"access_token":"test-only-owner-token"}})),origin).await;
    assert!(!connection.to_string().contains("test-only-owner-token"));
    let (status, _) = request(
        &apps[1],
        &format!(
            "/api/connectors/{}/forget",
            connection["id"].as_str().unwrap()
        ),
        Some(json!({})),
        origin,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let body = json!({"workflow":{"name":"Daily digest","nodes":[{"id":"notice","label":"Notice","kind":"notify","arguments":{"text":"Digest"}}]}});
    let (status, _) = request(
        &apps[0],
        "/api/workflows",
        Some(body.clone()),
        "https://evil.example",
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, flow) = request(&apps[0], "/api/workflows", Some(body), origin).await;
    assert_eq!(status, StatusCode::OK, "{flow}");
    let id = flow["id"].as_str().unwrap();
    assert_eq!(
        request(&apps[1], &format!("/api/workflows/{id}"), None, origin)
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    let (status, run) = request(
        &apps[0],
        &format!("/api/workflows/{id}/run"),
        Some(json!({"test_mode":true})),
        origin,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{run}");
    let run_id = run["id"].as_str().unwrap();
    assert_eq!(
        request(
            &apps[1],
            &format!("/api/workflow-runs/{run_id}"),
            None,
            origin
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let (status, cancelled) = request(
        &apps[0],
        &format!("/api/workflow-runs/{run_id}/cancel"),
        Some(json!({})),
        origin,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn google_routes_preserve_selection_and_report_unchecked_document_access() {
    let dir = tempfile::tempdir().unwrap();
    let workspace = dir.path().join("workspace");
    std::fs::create_dir_all(&workspace).unwrap();
    let runtime = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    runtime
        .store
        .put("settings", "active", &Settings::initial(workspace))
        .unwrap();
    let app = fenix_server::router(runtime, 4173);
    let origin = "http://127.0.0.1:4173";
    let (status, saved) = request(&app,"/api/connectors",Some(json!({"provider":"google","name":"Docs","settings":{"services":["docs"]},"allow_write":false})),origin).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(saved["settings"]["services"], json!(["docs"]));
    let id = saved["id"].as_str().unwrap();
    let (status, checked) = request(
        &app,
        &format!("/api/connectors/{id}/test"),
        Some(json!({})),
        origin,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(checked["state"], "unverified");
    assert_eq!(checked["service_checks"]["docs"]["status"], "not_checked");
    let (_, catalog) = request(&app, "/api/connectors", None, origin).await;
    assert_eq!(
        catalog["connections"][0]["service_checks"],
        checked["service_checks"]
    );
    assert_eq!(
        catalog["catalog"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == "google")
            .unwrap()["service_options"]
            .as_array()
            .unwrap()
            .len(),
        11
    );
    assert!(!catalog.to_string().contains("access_token"));
}
