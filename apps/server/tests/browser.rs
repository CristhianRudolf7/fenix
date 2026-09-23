use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use fenix_agent::Runtime;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

#[tokio::test]
async fn browser_profiles_and_credentials_are_local_and_do_not_create_chat_messages() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(dir.path(), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let app = fenix_server::router(runtime.clone(), 4173);
    let page = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/browser")
                .header("host", "localhost:4173")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        page.status(),
        StatusCode::OK,
        "Direct navigation to the owner browser must serve the application"
    );
    let request = |uri: &str, origin: &str, value: Value| {
        Request::builder()
            .method("POST")
            .uri(uri)
            .header("host", "localhost:4173")
            .header("origin", origin)
            .header("content-type", "application/json")
            .body(Body::from(value.to_string()))
            .unwrap()
    };
    let response = app
        .clone()
        .oneshot(request(
            "/api/browser/profiles",
            "https://evil.example",
            json!({"name":"personal"}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let response = app
        .clone()
        .oneshot(request(
            "/api/browser/profiles",
            "http://localhost:4173",
            json!({"name":"personal"}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/browser/status")
                .header("host", "localhost:4173")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["profiles"][0]["allow_agent"], false);
    assert_eq!(body["sessions"], json!([]));
    let response=app.clone().oneshot(request("/api/browser/credentials","http://localhost:4173",json!({"profile":"personal","action":"save","master":"master-password-long","password":"private-site-password"}))).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(!body.contains("private-site-password"));
    assert!(!body.contains("master-password-long"));
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/messages")
                .header("host", "localhost:4173")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(!body.contains("private-site-password"));
    let downloads = dir.path().join("browser/artifacts/personal/downloads");
    std::fs::create_dir_all(&downloads).unwrap();
    let id = "11111111-2222-4333-8444-555555555555";
    std::fs::write(downloads.join(id), b"download fixture").unwrap();
    let app = fenix_server::router(runtime.clone(), 4173);
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/browser/downloads/personal/{id}"))
                .header("host", "localhost:4173")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response.headers()["content-disposition"]
            .to_str()
            .unwrap()
            .contains("attachment")
    );
    assert_eq!(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .as_ref(),
        b"download fixture"
    );
    runtime.shutdown().await.unwrap();
}
