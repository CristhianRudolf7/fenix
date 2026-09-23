use fenix_core::{ChatRequest, ContentBlock, Message, ProviderId};
use fenix_credentials::{ApiKey, Credential, CredentialStore};
use fenix_providers::{ClientOptions, ProviderClient, ProviderConfig, ProviderError};
use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_partial_json, method, path},
};

fn client(options: ClientOptions) -> ProviderClient {
    let mut credentials = CredentialStore::default();
    credentials.set_default(
        ProviderId::OpenAi,
        Credential::inline(ApiKey::new("fake-stream-key").unwrap()),
    );
    ProviderClient::with_options(credentials, options).unwrap()
}
fn event(value: serde_json::Value) -> String {
    format!("data: {value}\n\n")
}

#[tokio::test]
async fn preview_masks_split_credentials_and_rejects_unsafe_replay() {
    let server = MockServer::start().await;
    let secret = "sk-abcdefghijklmnopqrstuvwxyz";
    let mut body = String::new();
    for delta in [
        "Working\nPassword: ",
        "'private ",
        "value'\n",
        "sk-abc",
        "defghijklmnopqrstuvwxyz",
        "\n",
    ] {
        body.push_str(&event(
            json!({"type":"response.output_text.delta","delta":delta}),
        ));
    }
    body.push_str(&event(json!({"type":"response.completed","response":{"id":"r1","status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":format!("Working\nPassword: 'private value'\n{secret}\n")}]}]}})));
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "text/event-stream"))
        .mount(&server)
        .await;
    let mut preview = String::new();
    let result = client(ClientOptions::default())
        .complete_with_preview(
            &ProviderConfig::new(ProviderId::OpenAi).with_base_url(server.uri()),
            None,
            &ChatRequest::new("test", vec![Message::user("Hello")]),
            &mut |delta| {
                assert!(!delta.contains("private"));
                assert!(!delta.contains("sk-"));
                preview.push_str(delta);
            },
        )
        .await;
    assert!(result.is_err());
    assert!(preview.contains("[REDACTED]"));
    assert!(!preview.contains(secret));
}

#[tokio::test]
#[ignore = "Opt-in billable OpenAI streaming check; requires OPENAI_API_KEY"]
async fn live_openai_preview_matches_the_completed_response() {
    let key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be configured privately");
    let mut credentials = CredentialStore::default();
    credentials.set_default(
        ProviderId::OpenAi,
        Credential::inline(ApiKey::new(key).unwrap()),
    );
    let client = ProviderClient::new(credentials).unwrap();
    let mut request = ChatRequest::new(
        "gpt-5.6-luna",
        vec![Message::user(
            "Say exactly: Fenix streaming connection is ready.",
        )],
    );
    request.reasoning_effort = Some(fenix_core::ReasoningEffort::Low);
    let mut preview = String::new();
    let mut chunks = 0;
    let response = client
        .complete_with_preview(
            &ProviderConfig::new(ProviderId::OpenAi),
            None,
            &request,
            &mut |text| {
                chunks += 1;
                preview.push_str(text);
            },
        )
        .await
        .unwrap();
    assert!(chunks > 0);
    assert!(!preview.trim().is_empty());
    assert_eq!(preview, response.message.text_content());
    assert_eq!(response.finish_reason, fenix_core::FinishReason::Stop);
}

#[tokio::test]
async fn streaming_preserves_terminal_tools_usage_and_reasoning_without_previewing_them() {
    let server = MockServer::start().await;
    let output = json!([
        {"type":"reasoning","summary":[],"encrypted_content":"opaque-signed-data"},
        {"type":"message","role":"assistant","content":[{"type":"output_text","text":"Reading now"}]},
        {"type":"function_call","call_id":"call-1","name":"file_read","arguments":"{\"path\":\"README.md\"}"}
    ]);
    let body = event(json!({"type":"response.output_text.delta","delta":"Reading "}))
        + &event(json!({"type":"response.function_call_arguments.delta","delta":"not executable"}))
        + &event(json!({"type":"response.reasoning_text.delta","delta":"hidden reasoning"}))
        + &event(json!({"type":"response.output_text.delta","delta":"now"}))
        + &event(
            json!({"type":"response.completed","response":{"id":"r1","status":"completed","output":output,"usage":{"input_tokens":10,"output_tokens":8,"total_tokens":18}}}),
        );
    Mock::given(method("POST"))
        .and(path("/responses"))
        .and(body_partial_json(json!({"stream":true,"store":false})))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(body, "text/event-stream; charset=utf-8"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let mut preview = String::new();
    let result = client(ClientOptions::default())
        .complete_with_preview(
            &ProviderConfig::new(ProviderId::OpenAi).with_base_url(server.uri()),
            None,
            &ChatRequest::new("test", vec![Message::user("Read the file")]),
            &mut |text| preview.push_str(text),
        )
        .await
        .unwrap();
    assert_eq!(preview, "Reading now");
    assert_eq!(result.usage.total_tokens, Some(18));
    assert_eq!(result.message.provider_data.unwrap().data, output);
    assert!(
        matches!(&result.message.content[1],ContentBlock::ToolCall { name, arguments, .. } if name == "file_read" && arguments["path"] == "README.md")
    );
}

#[tokio::test]
async fn interrupted_failed_and_oversized_streams_never_become_complete_responses() {
    for (body, limit) in [
        (
            event(json!({"type":"response.output_text.delta","delta":"partial"})),
            8192,
        ),
        (
            event(json!({"type":"response.failed","error":{"message":"private diagnostic"}})),
            8192,
        ),
        ("data: not-json\n\n".into(), 8192),
        ("data: ".repeat(100), 32),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "text/event-stream"))
            .expect(1)
            .mount(&server)
            .await;
        let error = client(ClientOptions {
            max_response_bytes: limit,
            ..ClientOptions::default()
        })
        .complete_with_preview(
            &ProviderConfig::new(ProviderId::OpenAi).with_base_url(server.uri()),
            None,
            &ChatRequest::new("test", vec![Message::user("Hello")]),
            &mut |_| {},
        )
        .await
        .unwrap_err();
        assert!(matches!(
            error,
            ProviderError::MalformedResponse | ProviderError::ResponseTooLarge { .. }
        ));
        assert!(!error.to_string().contains("private diagnostic"));
    }
}
