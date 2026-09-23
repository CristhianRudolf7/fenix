use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use fenix_agent::Runtime;
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test]
async fn integrations_are_owner_scoped_passive_and_csrf_protected() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(dir.path(), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let app = fenix_server::router(runtime.clone(), 4173);
    let get = |path: &str| {
        Request::builder()
            .uri(path)
            .header("host", "localhost:4173")
            .body(Body::empty())
            .unwrap()
    };
    assert_eq!(
        app.clone()
            .oneshot(get("/integrations"))
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    let response = app.clone().oneshot(get("/api/integrations")).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(json["connections"][0]["state"], "disconnected");
    assert_eq!(
        app.clone()
            .oneshot(get("/api/integrations/whatsapp/qr"))
            .await
            .unwrap()
            .status(),
        StatusCode::BAD_REQUEST
    );
    for action in ["connect", "pause", "forget"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/integrations/whatsapp/{action}"))
                    .header("host", "localhost:4173")
                    .header("origin", "https://evil.example")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
    for action in ["pause", "forget"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/integrations/whatsapp/{action}"))
                    .header("host", "localhost:4173")
                    .header("origin", "http://localhost:4173")
                    .header("content-type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    let response = app
        .clone()
        .oneshot(get("/api/browser/status"))
        .await
        .unwrap();
    let json: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(json["sessions"], serde_json::json!([]));
    let response = app.oneshot(get("/api/messages")).await.unwrap();
    let json: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(json, serde_json::json!([]));
}

#[tokio::test]
#[ignore = "Requires sandbox-capable Chromium and network access to WhatsApp; never links an account"]
async fn real_whatsapp_qr_is_served_only_as_an_uncached_owner_image() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(dir.path(), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let app = fenix_server::router(runtime.clone(), 4173);
    let get = |path: &str| {
        Request::builder()
            .uri(path)
            .header("host", "localhost:4173")
            .body(Body::empty())
            .unwrap()
    };
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/integrations/whatsapp/connect")
                .header("host", "localhost:4173")
                .header("origin", "http://localhost:4173")
                .header("content-type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let start = std::time::Instant::now();
    loop {
        let response = app.clone().oneshot(get("/api/integrations")).await.unwrap();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&bytes).unwrap();
        if json["connections"][0]["state"] == "awaiting_qr" {
            break;
        }
        assert!(start.elapsed().as_secs() < 60, "QR not available: {json}");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    let response = app
        .oneshot(get("/api/integrations/whatsapp/qr"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "image/png");
    assert!(
        response.headers()["cache-control"]
            .to_str()
            .unwrap()
            .contains("no-store")
    );
    assert_eq!(response.headers()["x-content-type-options"], "nosniff");
    assert!(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .starts_with(b"\x89PNG\r\n\x1a\n")
    );
    runtime.browser.whatsapp_forget().await.unwrap();
}
