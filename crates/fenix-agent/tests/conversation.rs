use fenix_agent::{Runtime, config::Settings, interlocutor::FastModelSettings};
use fenix_core::ProviderId;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use wiremock::{Mock, MockServer, Request, ResponseTemplate, matchers::method};
fn answer(text: &str) -> Value {
    json!({"id":"conversation-test","status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":text}]}],"usage":{"input_tokens":10,"output_tokens":10,"total_tokens":20}})
}
async fn fixture() -> (
    tempfile::TempDir,
    Arc<Runtime>,
    MockServer,
    Arc<Mutex<String>>,
) {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let server = MockServer::start().await;
    let decision = Arc::new(Mutex::new(
        r#"{"reply":"Hello","action":"observe"}"#.to_string(),
    ));
    let result = decision.clone();
    Mock::given(method("POST"))
        .respond_with(move |r: &Request| {
            let body: Value = serde_json::from_slice(&r.body).unwrap();
            let text = if body["model"] == "conversation-model-test"
                && body["input"].as_array().unwrap().len() > 1
            {
                result.lock().unwrap().clone()
            } else {
                "OK".into()
            };
            ResponseTemplate::new(200).set_body_json(answer(&text))
        })
        .mount(&server)
        .await;
    let mut settings = Settings::initial(dir.path().to_owned());
    settings.base_url = Some(server.uri());
    runtime
        .configure(settings, Some("master-test-key".into()))
        .await
        .unwrap();
    runtime
        .configure_fast_model(
            FastModelSettings {
                reasoning_effort: None,
                provider: ProviderId::OpenAi,
                model: "conversation-model-test".into(),
                base_url: Some(server.uri()),
            },
            Some("conversation-model-test-key".into()),
        )
        .await
        .unwrap();
    (dir, runtime, server, decision)
}
#[tokio::test]
async fn text_channels_use_the_interlocutor_and_preserve_master_tasks() {
    let (_dir, runtime, _server, decision) = fixture().await;
    for origin in ["web", "telegram"] {
        let reply = runtime
            .conversation_turn("Hello", origin, None)
            .await
            .unwrap();
        assert_eq!(reply["reply"], "Hello");
        assert_eq!(reply["delegated"], false);
    }
    assert!(runtime.store.tasks(10).unwrap().is_empty());
    *decision.lock().unwrap() = r#"{"reply":"Working","action":"delegate"}"#.into();
    let response = runtime
        .conversation_turn("Fix the login", "telegram", None)
        .await
        .unwrap();
    let task = runtime
        .store
        .task(response["task_id"].as_str().unwrap())
        .unwrap();
    assert_eq!(task.origin, "telegram");
    assert_eq!(task.request, "Fix the login");
    assert_eq!(runtime.store.task(&task.id).unwrap().status, "queued");
    *decision.lock().unwrap() = r#"{"reply":"Still here","action":"observe"}"#.into();
    assert_eq!(
        runtime
            .conversation_turn("Hello again", "web", None)
            .await
            .unwrap()["reply"],
        "Still here"
    );
}

#[tokio::test]
async fn invalid_text_decision_never_queues_master_work() {
    let (_dir, runtime, _server, decision) = fixture().await;
    *decision.lock().unwrap() = "invalid".into();
    assert!(
        runtime
            .conversation_turn("Do work", "web", None)
            .await
            .is_err()
    );
    assert!(runtime.store.tasks(10).unwrap().is_empty());
}

#[tokio::test]
async fn telegram_conversation_is_delivered_once_without_a_master_task() {
    let (_dir, runtime, _server, _) = fixture().await;
    runtime
        .store
        .put(
            "telegram",
            "settings",
            &fenix_agent::telegram::TelegramSettings {
                enabled: true,
                username: "test_bot".into(),
                owner_id: Some(123),
                pairing_code: None,
                offset: 0,
                last_error: None,
            },
        )
        .unwrap();
    runtime
        .conversation_turn("Hello", "telegram", None)
        .await
        .unwrap();
    runtime.deliver_events().unwrap();
    runtime.deliver_events().unwrap();
    let pending = runtime.store.deliveries_for("test_bot:123").unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].1, "Hello");
    assert!(runtime.store.tasks(10).unwrap().is_empty());
}
