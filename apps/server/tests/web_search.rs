use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use fenix_agent::Runtime;
use http_body_util::BodyExt;
use serde_json::Value;
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
