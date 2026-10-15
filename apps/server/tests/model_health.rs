use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use fenix_agent::{Runtime, config::Settings, interlocutor::FastModelSettings};
use fenix_core::ProviderId;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};
fn answer() -> Value {
    json!({"id":"health-fixture","status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":"OK"}]}],"usage":{"input_tokens":4,"output_tokens":1,"total_tokens":5}})
}
async fn configured() -> (tempfile::TempDir, Arc<Runtime>, MockServer) {
    let dir = tempfile::tempdir().unwrap();
    let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(answer()))
        .mount(&server)
        .await;
    let mut settings = Settings::initial(dir.path().to_owned());
    settings.base_url = Some(server.uri());
    rt.configure(settings, Some("test-master-key".into()))
        .await
        .unwrap();
    rt.configure_fast_model(
        FastModelSettings {
            provider: ProviderId::OpenAi,
            model: "test-fast".into(),
            base_url: Some(server.uri()),
            reasoning_effort: None,
        },
        Some("test-fast-key".into()),
    )
    .await
    .unwrap();
    (dir, rt, server)
}
#[tokio::test]
async fn status_is_cached_across_restart_and_unavailable_models_block_http_messages() {
    let (dir, rt, server) = configured().await;
    assert_eq!(rt.model_health_status().await.unwrap()["can_send"], true);
    let requests = server.received_requests().await.unwrap().len();
    for _ in 0..3 {
        rt.check_model_health(false).await.unwrap();
    }
    assert_eq!(server.received_requests().await.unwrap().len(), requests);
    rt.shutdown().await.unwrap();
    drop(rt);
    let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    assert_eq!(
        rt.check_model_health(false).await.unwrap()["can_send"],
        true
    );
    assert_eq!(server.received_requests().await.unwrap().len(), requests);
    // Expire cached probes to exercise provider failure without sleeping or real credentials.
    for role in ["intelligent", "fast"] {
        let mut health = rt
            .store
            .get::<Value>("model_health", role)
            .unwrap()
            .unwrap();
        health["checked_at"] = json!(0);
        health["next_check"] = json!(0);
        rt.store.put("model_health", role, &health).unwrap();
    }
    server.reset().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;
    let value = rt.check_model_health(false).await.unwrap();
    assert_eq!(value["can_send"], false);
    assert_eq!(value["roles"]["intelligent"]["reason"], "authentication");
    let requests = server.received_requests().await.unwrap().len();
    rt.check_model_health(true).await.unwrap();
    assert_eq!(
        server.received_requests().await.unwrap().len(),
        requests,
        "manual retries honor cooldown"
    );
    let app = fenix_server::router(rt.clone(), 4173);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/chat")
                .header("host", "127.0.0.1:4173")
                .header("content-type", "application/json")
                .method("POST")
                .body(Body::from(json!({"text":"Hello"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert!(String::from_utf8_lossy(&body).contains("modelo"));
    assert!(rt.store.tasks(10).unwrap().is_empty());
    rt.shutdown().await.unwrap();
}
#[tokio::test]
async fn rotated_credentials_invalidate_cached_success_and_probes_are_single_flight() {
    let (dir, rt, server) = configured().await;
    let settings = rt.settings().unwrap().unwrap();
    rt.store
        .put(
            "credentials",
            &settings.credential_id(),
            &"rotated-test-key",
        )
        .unwrap();
    rt.shutdown().await.unwrap();
    drop(rt);
    let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    assert_eq!(rt.model_health_status().await.unwrap()["can_send"], false);
    server.reset().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(answer())
                .set_delay(std::time::Duration::from_millis(100)),
        )
        .mount(&server)
        .await;
    let (one, two) = tokio::join!(rt.check_model_health(false), rt.check_model_health(false));
    one.unwrap();
    two.unwrap();
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
    assert_eq!(rt.model_health_status().await.unwrap()["can_send"], true);
    rt.shutdown().await.unwrap();
}

#[tokio::test]
async fn real_conversation_failure_invalidates_health_without_exposing_provider_details() {
    let (_dir, rt, server) = configured().await;
    server.reset().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(401).set_body_string("private fixture provider response"),
        )
        .mount(&server)
        .await;
    assert!(rt.conversation_turn("Hello", "web", None).await.is_err());
    let health = rt.model_health_status().await.unwrap();
    assert_eq!(health["can_send"], false);
    assert_eq!(health["roles"]["fast"]["reason"], "authentication");
    assert!(!health.to_string().contains("private fixture"));
    rt.shutdown().await.unwrap();
}

#[tokio::test]
async fn changing_model_with_the_same_key_invalidates_persisted_health() {
    let (dir, rt, _server) = configured().await;
    let mut settings = rt.settings().unwrap().unwrap();
    settings.model = "different-model".into();
    rt.store.put("settings", "active", &settings).unwrap();
    rt.shutdown().await.unwrap();
    drop(rt);
    let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let health = rt.model_health_status().await.unwrap();
    assert_eq!(health["can_send"], false);
    assert_eq!(health["roles"]["intelligent"]["status"], "unchecked");
    rt.shutdown().await.unwrap();
}
