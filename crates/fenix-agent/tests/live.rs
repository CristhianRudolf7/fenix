use fenix_agent::{
    Runtime,
    config::Settings,
    live::{LiveInput, LiveSettings},
};
use fenix_core::ProviderId;
use serde_json::{Value, json};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use wiremock::{Mock, MockServer, Request, ResponseTemplate, matchers::method};
fn answer(text: &str) -> Value {
    json!({"id":"live-test","status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":text}]}],"usage":{"input_tokens":10,"output_tokens":10,"total_tokens":20}})
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
            let text =
                if body["model"] == "voice-test" && body["input"].as_array().unwrap().len() > 1 {
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
        .configure_live(
            LiveSettings {
                reasoning_effort: None,
                provider: ProviderId::OpenAi,
                model: "voice-test".into(),
                base_url: Some(server.uri()),
            },
            Some("voice-test-key".into()),
        )
        .await
        .unwrap();
    (dir, runtime, server, decision)
}
fn input(id: &str, text: &str) -> LiveInput {
    LiveInput {
        request_id: id.into(),
        text: Some(text.into()),
        language: Some("en".into()),
    }
}
#[tokio::test]
async fn live_observation_does_not_queue_work_and_shares_history() {
    let (_dir, r, server, _) = fixture().await;
    let session = r.start_live().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let before = r.settings().unwrap().unwrap();
    let result = r
        .live_turn(&session, input("question", "How is project A?"))
        .await
        .unwrap();
    assert_eq!(result["delegated"], false);
    assert!(r.store.tasks(10).unwrap().is_empty());
    assert_eq!(r.settings().unwrap().unwrap().model, before.model);
    assert!(
        r.store
            .context(0)
            .unwrap()
            .iter()
            .any(|m| m.text == "How is project A?")
    );
    let history = r.store.conversation(None, 10).unwrap();
    assert!(history.iter().any(|m| m.text == "How is project A?"));
    assert!(history.iter().any(|m| m.text == "Hello"));
    for request in server.received_requests().await.unwrap() {
        let body: Value = serde_json::from_slice(&request.body).unwrap();
        if body["model"] == "voice-test" {
            assert!(
                body["tools"].is_null() || body["tools"].as_array().is_some_and(|v| v.is_empty())
            );
        }
    }
}
#[tokio::test]
async fn forwarding_is_exact_idempotent_and_survives_hangup() {
    let (_dir, r, _server, decision) = fixture().await;
    *decision.lock().unwrap() = r#"{"reply":"I sent your request","action":"delegate"}"#.into();
    let session = r.start_live().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let text = "Create a short project brief";
    let first = r
        .live_turn(&session, input("same-turn", text))
        .await
        .unwrap();
    let second = r
        .live_turn(&session, input("same-turn", text))
        .await
        .unwrap();
    assert_eq!(first, second);
    let tasks = r.store.tasks(10).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].request, text);
    r.end_live(&session).await.unwrap();
    assert_eq!(r.store.task(&tasks[0].id).unwrap().status, "queued");
    assert!(
        r.live_turn(&session, input("late", "cancel"))
            .await
            .is_err()
    );
}
#[tokio::test]
async fn closing_live_cancels_only_the_pending_voice_response() {
    let (_dir, r, server, _) = fixture().await;
    let id = r.submit("Keep this master task", "web").await.unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    server.reset().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(2))
                .set_body_json(answer(r#"{"reply":"late","action":"delegate"}"#)),
        )
        .mount(&server)
        .await;
    let session = r.start_live().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let rt = r.clone();
    let sid = session.clone();
    let task =
        tokio::spawn(async move { rt.live_turn(&sid, input("pending", "Do more work")).await });
    tokio::time::sleep(Duration::from_millis(50)).await;
    r.end_live(&session).await.unwrap();
    assert!(
        tokio::time::timeout(Duration::from_millis(500), task)
            .await
            .unwrap()
            .unwrap()
            .is_err()
    );
    assert_eq!(r.store.tasks(10).unwrap().len(), 1);
    assert_eq!(r.store.task(&id).unwrap().status, "queued");
}

#[tokio::test]
async fn text_channels_use_interlocutor_without_starting_or_closing_voice() {
    let (_dir, runtime, _server, decision) = fixture().await;
    let voice = runtime.start_live().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
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
    runtime.end_live(&voice).await.unwrap();
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
