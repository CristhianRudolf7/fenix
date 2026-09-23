use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use fenix_agent::Runtime;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

#[tokio::test]
async fn web_status_is_local_metadata_and_remains_origin_guarded() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(dir.path(), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let app = fenix_server::router(runtime.clone(), 4173);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/web/status")
                .header("host", "localhost:4173")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["enabled"], true);
    assert_eq!(body["crawler_running"], false);
    assert_eq!(body["max_concurrent_searches"], 10);
    assert_eq!(body["active_operations"], 0);
    assert_eq!(body["queued_operations"], 0);
    assert_eq!(body["active_downloads"], 0);
    assert_eq!(body["coverage"]["indexed_pages"], 0);
    assert_eq!(body["coverage"]["requests_today"], 0);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/web/status")
                .header("host", "localhost:4173")
                .header("origin", "https://foreign.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    runtime.shutdown().await.unwrap();
}

#[tokio::test]
async fn owner_search_uses_its_own_index_without_profiles_models_or_network() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(dir.path(), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let app = fenix_server::router(runtime.clone(), 4173);
    let request = |origin: &str, input: Value| {
        Request::builder()
            .method("POST")
            .uri("/api/web/search")
            .header("host", "localhost:4173")
            .header("origin", origin)
            .header("content-type", "application/json")
            .body(Body::from(input.to_string()))
            .unwrap()
    };
    for input in [
        json!({"query":"Rust ownership"}),
        json!({"query":"Recent research", "domains":["arxiv.org"], "days":30, "read_pages":0}),
    ] {
        let response = app
            .clone()
            .oneshot(request("http://localhost:4173", input))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body: Value =
            serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(body["results"], json!([]));
        assert_eq!(body["coverage"]["requests_today"], 0);
        assert!(body["warnings"].as_array().is_some_and(|w| !w.is_empty()));
    }
    for input in [
        json!({"query":""}),
        json!({"query":"x", "read_pages":4}),
        json!({"query":"x", "days":0}),
        json!({"query":"x", "source_urls":["http://127.0.0.1/private"]}),
        json!([]),
    ] {
        let response = app
            .clone()
            .oneshot(request("http://localhost:4173", input))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
    let response = app
        .oneshot(request("https://foreign.example", json!({"query":"x"})))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        runtime.web.status().await.unwrap().coverage.requests_today,
        0
    );
    assert_eq!(
        runtime.browser.status().await.unwrap()["profiles"],
        json!([])
    );
    runtime.shutdown().await.unwrap();
}
