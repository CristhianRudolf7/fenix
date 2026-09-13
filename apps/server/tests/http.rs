use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use fenix_agent::Runtime;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

#[tokio::test]
async fn progress_sends_a_fresh_snapshot_and_keeps_the_local_origin_guard() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(dir.path(), None).await.unwrap();
    runtime
        .store
        .put("credentials", "openai:default", &"private-not-for-events")
        .unwrap();
    let app = fenix_server::router(runtime.clone(), 4173);
    for origin in ["https://foreign.example", "http://localhost:4173"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/progress")
                    .header("host", "localhost:4173")
                    .header("origin", origin)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        if origin.starts_with("https") {
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
            continue;
        }
        assert_eq!(response.headers()["content-type"], "text/event-stream");
        let mut body = response.into_body();
        let frame = tokio::time::timeout(std::time::Duration::from_secs(1), body.frame())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let text = std::str::from_utf8(frame.data_ref().unwrap()).unwrap();
        assert!(text.contains("event: progress"));
        assert!(text.contains("\"phase\":\"idle\""));
        assert!(!text.contains("private-not-for-events"));
        runtime.shutdown().await.unwrap();
        assert!(
            tokio::time::timeout(std::time::Duration::from_secs(1), body.frame())
                .await
                .unwrap()
                .is_none()
        );
    }
}

#[tokio::test]
async fn loopback_guard_blocks_foreign_hosts_origins_and_form_posts() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(dir.path(), None).await.unwrap();
    let app = fenix_server::router(runtime, 4173);
    for (host, origin, method, content_type, expected) in [
        (
            "evil.example:4173",
            None,
            "GET",
            None,
            StatusCode::FORBIDDEN,
        ),
        (
            "127.0.0.1:4173",
            Some("https://evil.example"),
            "GET",
            None,
            StatusCode::FORBIDDEN,
        ),
        (
            "127.0.0.1:4173",
            None,
            "POST",
            Some("text/plain"),
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (
            "127.0.0.1:4173",
            Some("http://127.0.0.1:4173"),
            "GET",
            None,
            StatusCode::OK,
        ),
    ] {
        let mut request = Request::builder()
            .method(method)
            .uri("/api/status")
            .header("host", host);
        if let Some(origin) = origin {
            request = request.header("origin", origin);
        }
        if let Some(content_type) = content_type {
            request = request.header("content-type", content_type);
        }
        let response = app
            .clone()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
    }
}

#[tokio::test]
async fn api_exposes_catalog_and_memory_but_never_private_credentials() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(dir.path(), None).await.unwrap();
    runtime
        .store
        .put("credentials", "openai:default", &"secret-should-not-appear")
        .unwrap();
    let app = fenix_server::router(runtime, 4173);
    let get = |uri: &str| {
        Request::builder()
            .uri(uri)
            .header("host", "localhost:4173")
            .body(Body::empty())
            .unwrap()
    };
    let response = app.clone().oneshot(get("/api/providers")).await.unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let catalog: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(catalog.as_array().unwrap().len(), 17);
    let response = app.clone().oneshot(get("/api/status")).await.unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert!(!String::from_utf8_lossy(&bytes).contains("secret-should-not-appear"));
    let request = Request::builder()
        .method("POST")
        .uri("/api/memory")
        .header("host", "localhost:4173")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"title":"Preference","content":"I prefer short answers"}).to_string(),
        ))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response = app.oneshot(get("/api/memory?query=short")).await.unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(
        serde_json::from_slice::<Value>(&bytes)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
