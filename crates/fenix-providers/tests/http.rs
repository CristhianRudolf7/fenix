use fenix_core::{
    ChatRequest, ContentBlock, FinishReason, Message, ProviderId, Role, ToolDefinition,
};
use fenix_credentials::{ApiKey, Credential, CredentialSource, CredentialStore};
use fenix_providers::{ClientOptions, ProviderClient, ProviderConfig, ProviderError};
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

const DEFAULT_KEY: &str = "test-default-credential";
const PROJECT_KEY: &str = "test-project-credential";

fn credential(value: &str) -> Credential {
    Credential::inline(ApiKey::new(value).expect("valid test credential"))
}

fn request() -> ChatRequest {
    ChatRequest::new("test-model", vec![Message::user("Say hello.")])
}

fn fixture(provider: ProviderId) -> Value {
    match provider {
        ProviderId::OpenAi => json!({
            "id": "response-test", "object": "response", "status": "completed",
            "output": [{"type": "message", "role": "assistant", "content": [
                {"type": "output_text", "text": "Ready."}
            ]}],
            "usage": {"input_tokens": 8, "output_tokens": 3, "total_tokens": 11}
        }),
        ProviderId::Anthropic => json!({
            "id": "message-test", "type": "message", "role": "assistant",
            "model": "test-model", "content": [{"type": "text", "text": "Ready."}],
            "stop_reason": "end_turn", "usage": {"input_tokens": 8, "output_tokens": 3}
        }),
        ProviderId::Gemini | ProviderId::Vertex => json!({
            "responseId": "response-test", "candidates": [{
                "content": {"role": "model", "parts": [{"text": "Ready."}]},
                "finishReason": "STOP"
            }],
            "usageMetadata": {"promptTokenCount": 8, "candidatesTokenCount": 3,
                "totalTokenCount": 11}
        }),
        ProviderId::Bedrock => json!({
            "output": {"message": {"role": "assistant", "content": [{"text": "Ready."}]}},
            "stopReason": "end_turn",
            "usage": {"inputTokens": 8, "outputTokens": 3, "totalTokens": 11}
        }),
        _ => json!({
            "id": "completion-test", "object": "chat.completion", "choices": [{
                "index": 0, "message": {"role": "assistant", "content": "Ready."},
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 8, "completion_tokens": 3, "total_tokens": 11}
        }),
    }
}

fn expected_path(provider: ProviderId) -> &'static str {
    match provider {
        ProviderId::OpenAi => "/v1/responses",
        ProviderId::Anthropic => "/v1/messages",
        ProviderId::Gemini | ProviderId::Vertex => "/v1/models/test-model:generateContent",
        ProviderId::Bedrock => "/v1/model/test-model/converse",
        _ => "/v1/chat/completions",
    }
}

fn tool_fixture(provider: ProviderId) -> Value {
    let mut body = fixture(provider);
    let arguments = json!({"city": "Lima"});
    match provider {
        ProviderId::OpenAi => {
            body["output"] = json!([{
                "type": "function_call", "call_id": "call-1", "name": "lookup_weather",
                "arguments": arguments.to_string()
            }]);
        }
        ProviderId::Anthropic => {
            body["content"] = json!([{
                "type": "tool_use", "id": "call-1", "name": "lookup_weather", "input": arguments
            }]);
            body["stop_reason"] = json!("tool_use");
        }
        ProviderId::Gemini | ProviderId::Vertex => {
            body["candidates"][0]["content"]["parts"] = json!([{
                "functionCall": {"id": "call-1", "name": "lookup_weather", "args": arguments}
            }]);
        }
        ProviderId::Bedrock => {
            body["output"]["message"]["content"] = json!([{
                "toolUse": {"toolUseId": "call-1", "name": "lookup_weather", "input": arguments}
            }]);
            body["stopReason"] = json!("tool_use");
        }
        _ => {
            body["choices"][0]["message"]["content"] = Value::Null;
            body["choices"][0]["message"]["tool_calls"] = json!([{
                "id": "call-1", "type": "function", "function": {
                    "name": "lookup_weather", "arguments": arguments.to_string()
                }
            }]);
            body["choices"][0]["finish_reason"] = json!("tool_calls");
        }
    }
    body
}

fn config(provider: ProviderId, server: &MockServer) -> ProviderConfig {
    ProviderConfig::new(provider).with_base_url(format!("{}/v1", server.uri()))
}

fn client(provider: ProviderId) -> ProviderClient {
    let mut credentials = CredentialStore::default();
    credentials.set_default(provider, credential(DEFAULT_KEY));
    ProviderClient::new(credentials).expect("valid test client")
}

#[tokio::test]
async fn provider_boundary_redacts_prompts_but_preserves_authentication() {
    let provider = ProviderId::Groq;
    let server = MockServer::start().await;
    let mut body = fixture(provider);
    body["choices"][0]["message"]["content"] = json!("Ready.");
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    let request = ChatRequest::new(
        "test-model",
        vec![Message::user(format!(
            "Credential {DEFAULT_KEY}\nPassword: synthetic-request-secret"
        ))],
    );
    let result = client(provider)
        .complete(&config(provider, &server), None, &request)
        .await
        .unwrap();
    assert_eq!(result.message.text_content(), "Ready.");
    let requests = server.received_requests().await.unwrap();
    let sent = String::from_utf8(requests[0].body.clone()).unwrap();
    assert!(!sent.contains(DEFAULT_KEY));
    assert!(!sent.contains("synthetic-request-secret"));
    assert_eq!(
        requests[0].headers.get("authorization").unwrap(),
        &format!("Bearer {DEFAULT_KEY}")
    );
}

#[tokio::test]
async fn secret_bearing_chat_replay_is_never_returned_and_is_masked_like_content() {
    // Unsigned Chat Completions replay can be masked; signed or opaque replay is
    // still rejected unchanged (see fenix-core safety tests).
    let provider = ProviderId::Groq;
    let server = MockServer::start().await;
    let mut body = fixture(provider);
    body["choices"][0]["message"]["content"] = json!("Password: synthetic-response-secret");
    body["choices"][0]["message"]["reasoning"] =
        json!("Try Authorization: Bearer synthetic-response-secret");
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    let response = client(provider)
        .complete(&config(provider, &server), None, &request())
        .await
        .unwrap();
    let returned = serde_json::to_string(&response.message).unwrap();
    assert!(!returned.contains("synthetic-response-secret"));
    assert!(response.message.text_content().contains("[REDACTED]"));
    assert!(returned.contains("Authorization: [REDACTED]"));
}

#[tokio::test]
async fn every_provider_sends_default_and_project_keys_with_its_native_protocol() {
    for provider in ProviderId::ALL {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(fixture(provider)))
            .expect(2)
            .mount(&server)
            .await;
        let mut credentials = CredentialStore::default();
        credentials.set_default(provider, credential(DEFAULT_KEY));
        credentials
            .set_project("project-a", provider, credential(PROJECT_KEY))
            .expect("valid project scope");
        let client = ProviderClient::new(credentials).expect("valid client");

        for scope in [None, Some("project-a")] {
            let response = client
                .complete(&config(provider, &server), scope, &request())
                .await
                .unwrap_or_else(|error| panic!("{provider} completion failed: {error}"));
            assert_eq!(response.message.text_content(), "Ready.", "{provider}");
            assert_eq!(response.usage.input_tokens, Some(8), "{provider}");
            assert_eq!(response.usage.output_tokens, Some(3), "{provider}");
            assert_eq!(response.usage.total_tokens, Some(11), "{provider}");
            assert_eq!(response.finish_reason, FinishReason::Stop, "{provider}");
        }

        let requests = server.received_requests().await.expect("request recording");
        assert_eq!(requests.len(), 2);
        for (recorded, expected_key) in requests.iter().zip([DEFAULT_KEY, PROJECT_KEY]) {
            assert_eq!(recorded.url.path(), expected_path(provider), "{provider}");
            assert!(
                recorded.url.query().is_none(),
                "credentials must not be URL parameters"
            );
            let header_name = match provider {
                ProviderId::Anthropic => "x-api-key",
                ProviderId::Gemini | ProviderId::Vertex => "x-goog-api-key",
                _ => "authorization",
            };
            let expected_header = if header_name == "authorization" {
                format!("Bearer {expected_key}")
            } else {
                expected_key.to_owned()
            };
            let actual = recorded
                .headers
                .get(header_name)
                .and_then(|value| value.to_str().ok());
            assert!(
                actual == Some(expected_header.as_str()),
                "{provider} credential mismatch"
            );
            if provider == ProviderId::Anthropic {
                assert_eq!(recorded.headers["anthropic-version"], "2023-06-01");
            }
            assert!(!recorded.headers.contains_key("OpenAI-Project"));
            let body: Value = recorded.body_json().expect("JSON request");
            assert!(!String::from_utf8_lossy(&recorded.body).contains(expected_key));
            if provider == ProviderId::Azure {
                assert_eq!(body["max_completion_tokens"], 1024);
                assert!(body.get("max_tokens").is_none());
            }
            match provider {
                ProviderId::OpenAi => {
                    assert_eq!(body["model"], "test-model");
                    assert!(body["input"].is_array());
                }
                ProviderId::Gemini | ProviderId::Vertex => {
                    assert_eq!(body["contents"][0]["parts"][0]["text"], "Say hello.");
                }
                ProviderId::Bedrock => {
                    assert_eq!(body["messages"][0]["content"][0]["text"], "Say hello.");
                }
                _ => {
                    assert_eq!(body["model"], "test-model");
                    assert!(body["messages"].is_array());
                }
            }
        }
        server.verify().await;
    }
}

#[tokio::test]
async fn all_five_protocols_complete_an_http_tool_call_and_result_roundtrip() {
    for provider in [
        ProviderId::OpenAi,
        ProviderId::DeepSeek,
        ProviderId::Anthropic,
        ProviderId::Gemini,
        ProviderId::Bedrock,
    ] {
        let server = MockServer::start().await;
        let replies = Arc::new(AtomicUsize::new(0));
        let replies_for_server = Arc::clone(&replies);
        Mock::given(method("POST"))
            .respond_with(move |_request: &wiremock::Request| {
                let body = if replies_for_server.fetch_add(1, Ordering::SeqCst) == 0 {
                    tool_fixture(provider)
                } else {
                    fixture(provider)
                };
                ResponseTemplate::new(200).set_body_json(body)
            })
            .expect(2)
            .mount(&server)
            .await;
        let mut credentials = CredentialStore::default();
        credentials
            .set_project("weather-project", provider, credential(PROJECT_KEY))
            .expect("valid project");
        let client = ProviderClient::new(credentials).expect("valid client");
        let config = config(provider, &server);
        let mut initial = ChatRequest::new(
            "test-model",
            vec![Message::user("What is the temperature in Lima?")],
        );
        let parameters = json!({"type": "object", "properties": {"city": {"type": "string"}}, "required": ["city"]});
        initial.tools.push(ToolDefinition {
            name: "lookup_weather".into(),
            description: "Get a city's current weather.".into(),
            parameters: parameters.clone(),
        });

        let response = client
            .complete(&config, Some("weather-project"), &initial)
            .await
            .expect("tool-call response");
        assert_eq!(
            response.finish_reason,
            FinishReason::ToolCalls,
            "{provider}"
        );
        let call_id = match response.message.content.as_slice() {
            [
                ContentBlock::ToolCall {
                    id,
                    name,
                    arguments,
                },
            ] => {
                assert_eq!(id, "call-1", "{provider}");
                assert_eq!(name, "lookup_weather", "{provider}");
                assert_eq!(*arguments, json!({"city": "Lima"}), "{provider}");
                id.clone()
            }
            _ => panic!("{provider} must return exactly one structured tool call"),
        };
        initial.messages.push(response.message);
        // Simulate the application's tool executor while preserving the complete
        // assistant response, including any protocol-specific replay metadata.
        initial.messages.push(Message {
            role: Role::Tool,
            content: vec![ContentBlock::ToolResult {
                tool_call_id: call_id,
                name: None,
                content: json!({"temperature_c": 21}),
                is_error: false,
            }],
            provider_data: None,
        });
        let final_response = client
            .complete(&config, Some("weather-project"), &initial)
            .await
            .expect("final completion after tool result");
        assert_eq!(
            final_response.message.text_content(),
            "Ready.",
            "{provider}"
        );
        assert_eq!(
            final_response.finish_reason,
            FinishReason::Stop,
            "{provider}"
        );
        assert_eq!(replies.load(Ordering::SeqCst), 2);

        let requests = server.received_requests().await.expect("request recording");
        let first: Value = requests[0].body_json().expect("initial request JSON");
        let second: Value = requests[1].body_json().expect("follow-up request JSON");
        for recorded in &requests {
            let header_name = match provider {
                ProviderId::Anthropic => "x-api-key",
                ProviderId::Gemini => "x-goog-api-key",
                _ => "authorization",
            };
            let expected = if header_name == "authorization" {
                format!("Bearer {PROJECT_KEY}")
            } else {
                PROJECT_KEY.to_owned()
            };
            assert!(
                recorded.headers[header_name]
                    .to_str()
                    .expect("credential header")
                    == expected,
                "{provider} must retain the selected project's credential through the tool roundtrip"
            );
        }
        match provider {
            ProviderId::OpenAi => {
                assert_eq!(first["tools"][0]["name"], "lookup_weather");
                assert_eq!(first["tools"][0]["parameters"], parameters);
                assert_eq!(second["input"][1]["type"], "function_call");
                assert_eq!(second["input"][1]["call_id"], "call-1");
                assert_eq!(second["input"][2]["type"], "function_call_output");
                assert_eq!(second["input"][2]["call_id"], "call-1");
                let output: Value = serde_json::from_str(
                    second["input"][2]["output"]
                        .as_str()
                        .expect("serialized tool result"),
                )
                .expect("tool result JSON");
                assert_eq!(output, json!({"temperature_c": 21}));
            }
            ProviderId::Anthropic => {
                assert_eq!(first["tools"][0]["name"], "lookup_weather");
                assert_eq!(first["tools"][0]["input_schema"], parameters);
                assert_eq!(second["messages"][1]["content"][0]["type"], "tool_use");
                assert_eq!(second["messages"][2]["content"][0]["type"], "tool_result");
                assert_eq!(second["messages"][2]["content"][0]["tool_use_id"], "call-1");
                let output: Value = serde_json::from_str(
                    second["messages"][2]["content"][0]["content"]
                        .as_str()
                        .expect("serialized tool result"),
                )
                .expect("tool result JSON");
                assert_eq!(output, json!({"temperature_c": 21}));
            }
            ProviderId::Gemini => {
                assert_eq!(
                    first["tools"][0]["functionDeclarations"][0]["name"],
                    "lookup_weather"
                );
                assert_eq!(
                    first["tools"][0]["functionDeclarations"][0]["parametersJsonSchema"],
                    parameters
                );
                assert_eq!(
                    second["contents"][1]["parts"][0]["functionCall"]["id"],
                    "call-1"
                );
                let output = &second["contents"][2]["parts"][0]["functionResponse"];
                assert_eq!(output["id"], "call-1");
                assert_eq!(output["name"], "lookup_weather");
                assert_eq!(output["response"], json!({"temperature_c": 21}));
            }
            ProviderId::Bedrock => {
                assert_eq!(
                    first["toolConfig"]["tools"][0]["toolSpec"]["name"],
                    "lookup_weather"
                );
                assert_eq!(
                    first["toolConfig"]["tools"][0]["toolSpec"]["inputSchema"]["json"],
                    parameters
                );
                assert_eq!(
                    second["messages"][1]["content"][0]["toolUse"]["toolUseId"],
                    "call-1"
                );
                let output = &second["messages"][2]["content"][0]["toolResult"];
                assert_eq!(output["toolUseId"], "call-1");
                assert_eq!(output["status"], "success");
                assert_eq!(output["content"][0]["json"], json!({"temperature_c": 21}));
            }
            _ => {
                assert_eq!(first["tools"][0]["function"]["name"], "lookup_weather");
                assert_eq!(first["tools"][0]["function"]["parameters"], parameters);
                assert_eq!(second["messages"][1]["tool_calls"][0]["id"], "call-1");
                assert_eq!(second["messages"][2]["role"], "tool");
                assert_eq!(second["messages"][2]["tool_call_id"], "call-1");
                let output: Value = serde_json::from_str(
                    second["messages"][2]["content"]
                        .as_str()
                        .expect("serialized tool result"),
                )
                .expect("tool result JSON");
                assert_eq!(output, json!({"temperature_c": 21}));
            }
        }
        server.verify().await;
    }
}

#[tokio::test]
async fn azure_v1_uses_max_completion_tokens_for_output_limits() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture(ProviderId::Azure)))
        .expect(1)
        .mount(&server)
        .await;
    let mut request = request();
    request.max_output_tokens = 512;
    client(ProviderId::Azure)
        .complete(&config(ProviderId::Azure, &server), None, &request)
        .await
        .expect("Azure v1 completion");
    let requests = server.received_requests().await.expect("request recording");
    let body: Value = requests[0].body_json().expect("Azure request JSON");
    assert_eq!(body["max_completion_tokens"], 512);
    assert!(body.get("max_tokens").is_none());
}

#[tokio::test]
async fn dot_segment_model_ids_are_rejected_before_outbound_http() {
    let server = MockServer::start().await;
    for provider in ProviderId::ALL {
        for model in [".", ".."] {
            let request = ChatRequest::new(model, vec![Message::user("Hello")]);
            let result = client(provider)
                .complete(&config(provider, &server), None, &request)
                .await;
            assert!(
                matches!(result, Err(ProviderError::InvalidRequest(_))),
                "{provider} must reject dot-segment model IDs"
            );
        }
    }
    assert!(
        server
            .received_requests()
            .await
            .expect("request recording")
            .is_empty()
    );
}

#[tokio::test]
async fn native_account_headers_are_distinct_from_internal_project_selection() {
    for provider in [ProviderId::OpenAi, ProviderId::Anthropic] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(fixture(provider)))
            .expect(1)
            .mount(&server)
            .await;
        let mut scoped = credential(PROJECT_KEY);
        scoped.provider_project = Some("native-project".into());
        if provider == ProviderId::OpenAi {
            scoped.organization = Some("native-organization".into());
        }
        let mut credentials = CredentialStore::default();
        credentials
            .set_project("internal-project", provider, scoped)
            .expect("valid scope");
        let client = ProviderClient::new(credentials).expect("valid client");
        client
            .complete(
                &config(provider, &server),
                Some("internal-project"),
                &request(),
            )
            .await
            .expect("native account selection");
        let requests = server.received_requests().await.expect("request recording");
        let headers = &requests[0].headers;
        if provider == ProviderId::OpenAi {
            assert_eq!(headers["OpenAI-Project"], "native-project");
            assert_eq!(headers["OpenAI-Organization"], "native-organization");
        } else {
            assert_eq!(headers["anthropic-workspace-id"], "native-project");
            assert!(!headers.contains_key("OpenAI-Project"));
        }
    }
}

#[tokio::test]
async fn project_selection_never_falls_back_to_default_or_another_project() {
    let server = MockServer::start().await;
    let provider = ProviderId::OpenAi;
    let mut credentials = CredentialStore::default();
    credentials.set_default(provider, credential(DEFAULT_KEY));
    credentials
        .set_project("project-a", provider, credential(PROJECT_KEY))
        .expect("valid scope");
    credentials
        .set_project("project-b", ProviderId::Anthropic, credential(PROJECT_KEY))
        .expect("valid scope");
    credentials
        .set_project(
            "missing-secret",
            provider,
            Credential {
                source: CredentialSource::Environment(
                    "FENIX_TEST_MISSING_PROJECT_KEY_73605762".into(),
                ),
                provider_project: None,
                organization: None,
            },
        )
        .expect("valid scope");
    let client = ProviderClient::new(credentials).expect("valid client");
    for project in ["unknown", "project-b", "missing-secret"] {
        let result = client
            .complete(&config(provider, &server), Some(project), &request())
            .await;
        assert!(
            matches!(result, Err(ProviderError::Credential(_))),
            "{project} must fail closed"
        );
    }
    assert!(
        server
            .received_requests()
            .await
            .expect("request recording")
            .is_empty()
    );
}

#[tokio::test]
async fn concurrent_requests_keep_each_projects_credential_isolated() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_millis(20))
                .set_body_json(fixture(ProviderId::OpenAi)),
        )
        .expect(2)
        .mount(&server)
        .await;
    let mut credentials = CredentialStore::default();
    credentials
        .set_project("project-a", ProviderId::OpenAi, credential(PROJECT_KEY))
        .expect("valid scope");
    credentials
        .set_project("project-b", ProviderId::OpenAi, credential(DEFAULT_KEY))
        .expect("valid scope");
    let first = ProviderClient::new(credentials).expect("valid client");
    let second = first.clone();
    let config = config(ProviderId::OpenAi, &server);
    let request_a = ChatRequest::new("test-model", vec![Message::user("project-a")]);
    let request_b = ChatRequest::new("test-model", vec![Message::user("project-b")]);
    let (result_a, result_b) = tokio::join!(
        first.complete(&config, Some("project-a"), &request_a),
        second.complete(&config, Some("project-b"), &request_b),
    );
    result_a.expect("first project completion");
    result_b.expect("second project completion");
    let requests = server.received_requests().await.expect("request recording");
    assert_eq!(requests.len(), 2);
    for recorded in requests {
        let body = String::from_utf8(recorded.body).expect("JSON text");
        let expected = if body.contains("project-a") {
            PROJECT_KEY
        } else {
            DEFAULT_KEY
        };
        let actual = recorded.headers["authorization"]
            .to_str()
            .expect("authorization header");
        assert!(
            actual == format!("Bearer {expected}"),
            "concurrent project credentials must not cross"
        );
    }
}

#[tokio::test]
async fn unsupported_native_project_metadata_is_rejected_before_sending() {
    let server = MockServer::start().await;
    for provider in ProviderId::ALL {
        if matches!(provider, ProviderId::OpenAi | ProviderId::Anthropic) {
            continue;
        }
        let mut credentials = CredentialStore::default();
        let mut scoped = credential(PROJECT_KEY);
        scoped.provider_project = Some("native-project".into());
        credentials.set_default(provider, scoped);
        let client = ProviderClient::new(credentials).expect("valid client");
        assert!(
            matches!(
                client
                    .complete(&config(provider, &server), None, &request())
                    .await,
                Err(ProviderError::InvalidConfiguration(_))
            ),
            "{provider} must not silently discard native project metadata"
        );
    }
    assert!(
        server
            .received_requests()
            .await
            .expect("request recording")
            .is_empty()
    );
}

#[tokio::test]
async fn unsupported_organization_metadata_is_rejected_before_sending() {
    let server = MockServer::start().await;
    for provider in ProviderId::ALL {
        if provider == ProviderId::OpenAi {
            continue;
        }
        let mut credentials = CredentialStore::default();
        let mut scoped = credential(PROJECT_KEY);
        scoped.organization = Some("native-organization".into());
        credentials.set_default(provider, scoped);
        let client = ProviderClient::new(credentials).expect("valid client");
        assert!(
            matches!(
                client
                    .complete(&config(provider, &server), None, &request())
                    .await,
                Err(ProviderError::InvalidConfiguration(_))
            ),
            "{provider} must not silently discard organization metadata"
        );
    }
    assert!(
        server
            .received_requests()
            .await
            .expect("request recording")
            .is_empty()
    );
}

#[tokio::test]
async fn authentication_failures_are_classified_without_echoing_response_secrets() {
    for status in [401, 403] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(status).set_body_string(DEFAULT_KEY))
            .expect(1)
            .mount(&server)
            .await;
        let error = client(ProviderId::OpenAi)
            .complete(&config(ProviderId::OpenAi, &server), None, &request())
            .await
            .expect_err("authentication failure");
        assert!(
            matches!(&error, ProviderError::Authentication { status: actual } if *actual == status)
        );
        assert!(!format!("{error}").contains(DEFAULT_KEY));
        assert!(!format!("{error:?}").contains(DEFAULT_KEY));
    }
}

#[tokio::test]
async fn rate_limits_preserve_retry_after_without_automatic_duplicate_requests() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "7"))
        .expect(1)
        .mount(&server)
        .await;
    let result = client(ProviderId::OpenAi)
        .complete(&config(ProviderId::OpenAi, &server), None, &request())
        .await;
    assert!(
        matches!(result, Err(ProviderError::RateLimited { retry_after: Some(delay) })
        if delay == Duration::from_secs(7))
    );
    server.verify().await;
}

#[tokio::test]
async fn server_errors_are_classified_without_exposing_the_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(503).set_body_string(DEFAULT_KEY))
        .expect(1)
        .mount(&server)
        .await;
    let error = client(ProviderId::OpenAi)
        .complete(&config(ProviderId::OpenAi, &server), None, &request())
        .await
        .expect_err("upstream unavailable");
    assert!(matches!(error, ProviderError::Http { status: 503 }));
    assert!(!format!("{error:?}").contains(DEFAULT_KEY));
}

#[tokio::test]
async fn redirects_are_not_followed_and_credentials_stay_at_the_initial_endpoint() {
    let server = MockServer::start().await;
    let destination = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(307).insert_header("location", destination.uri()))
        .expect(1)
        .mount(&server)
        .await;
    let result = client(ProviderId::Anthropic)
        .complete(&config(ProviderId::Anthropic, &server), None, &request())
        .await;
    assert!(matches!(result, Err(ProviderError::Http { status: 307 })));
    assert!(
        destination
            .received_requests()
            .await
            .expect("request recording")
            .is_empty()
    );
}

#[tokio::test]
async fn request_timeout_is_bounded_and_reported() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_millis(200))
                .set_body_json(fixture(ProviderId::OpenAi)),
        )
        .mount(&server)
        .await;
    let mut credentials = CredentialStore::default();
    credentials.set_default(ProviderId::OpenAi, credential(DEFAULT_KEY));
    let client = ProviderClient::with_options(
        credentials,
        ClientOptions {
            timeout: Duration::from_millis(30),
            ..ClientOptions::default()
        },
    )
    .expect("valid client");
    assert!(matches!(
        client
            .complete(&config(ProviderId::OpenAi, &server), None, &request())
            .await,
        Err(ProviderError::Timeout)
    ));
}

#[tokio::test]
async fn externally_supervised_requests_can_outlive_the_default_deadline() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_millis(200))
                .set_body_json(fixture(ProviderId::OpenAi)),
        )
        .mount(&server)
        .await;
    let mut credentials = CredentialStore::default();
    credentials.set_default(ProviderId::OpenAi, credential(DEFAULT_KEY));
    let client = ProviderClient::with_options(
        credentials,
        ClientOptions {
            timeout: Duration::from_millis(30),
            disable_request_timeout: true,
            ..ClientOptions::default()
        },
    )
    .expect("valid client");
    assert!(
        client
            .complete(&config(ProviderId::OpenAi, &server), None, &request())
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn response_size_is_bounded_before_decoding() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string("x".repeat(1024)))
        .mount(&server)
        .await;
    let mut credentials = CredentialStore::default();
    credentials.set_default(ProviderId::OpenAi, credential(DEFAULT_KEY));
    let client = ProviderClient::with_options(
        credentials,
        ClientOptions {
            max_response_bytes: 64,
            ..ClientOptions::default()
        },
    )
    .expect("valid client");
    assert!(matches!(
        client
            .complete(&config(ProviderId::OpenAi, &server), None, &request())
            .await,
        Err(ProviderError::ResponseTooLarge { limit: 64 })
    ));
}

#[tokio::test]
async fn chunked_responses_are_bounded_without_a_content_length_header() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("local listener");
    let address = listener.local_addr().expect("listener address");
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("client connection");
        let mut buffer = [0_u8; 4096];
        let mut headers = Vec::new();
        while !headers.windows(4).any(|window| window == b"\r\n\r\n") {
            let read = stream.read(&mut buffer).await.expect("request headers");
            assert_ne!(read, 0, "request must contain headers");
            headers.extend_from_slice(&buffer[..read]);
        }
        stream.write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n")
            .await.expect("response headers");
        let chunk = "x".repeat(80);
        stream
            .write_all(format!("50\r\n{chunk}\r\n0\r\n\r\n").as_bytes())
            .await
            .expect("chunked response body");
    });
    let mut credentials = CredentialStore::default();
    credentials.set_default(ProviderId::OpenAi, credential(DEFAULT_KEY));
    let client = ProviderClient::with_options(
        credentials,
        ClientOptions {
            max_response_bytes: 64,
            timeout: Duration::from_secs(2),
            ..ClientOptions::default()
        },
    )
    .expect("valid client");
    let config =
        ProviderConfig::new(ProviderId::OpenAi).with_base_url(format!("http://{address}/v1"));
    assert!(matches!(
        client.complete(&config, None, &request()).await,
        Err(ProviderError::ResponseTooLarge { limit: 64 })
    ));
    server.await.expect("mock server completed");
}

#[tokio::test]
async fn request_size_is_bounded_before_network_io() {
    let server = MockServer::start().await;
    let mut credentials = CredentialStore::default();
    credentials.set_default(ProviderId::OpenAi, credential(DEFAULT_KEY));
    let client = ProviderClient::with_options(
        credentials,
        ClientOptions {
            max_request_bytes: 64,
            ..ClientOptions::default()
        },
    )
    .expect("valid client");
    let request = ChatRequest::new("test-model", vec![Message::user("x".repeat(1024))]);
    assert!(matches!(
        client
            .complete(&config(ProviderId::OpenAi, &server), None, &request)
            .await,
        Err(ProviderError::RequestTooLarge { limit: 64 })
    ));
    assert!(
        server
            .received_requests()
            .await
            .expect("request recording")
            .is_empty()
    );
}

#[tokio::test]
async fn malformed_success_responses_are_not_accepted_as_completions() {
    for body in ["not JSON", "null", "{}"] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&server)
            .await;
        let result = client(ProviderId::OpenAi)
            .complete(&config(ProviderId::OpenAi, &server), None, &request())
            .await;
        assert!(matches!(
            result,
            Err(ProviderError::MalformedResponse | ProviderError::Codec(_))
        ));
    }
}

#[tokio::test]
async fn unsafe_base_urls_are_rejected_without_network_io() {
    for url in [
        "http://example.com/v1",
        "https://username:password@example.com/v1",
        "https://example.com/v1?key=secret",
        "https://example.com/v1#fragment",
        "file:///tmp/models",
    ] {
        let result = client(ProviderId::OpenAi)
            .complete(
                &ProviderConfig::new(ProviderId::OpenAi).with_base_url(url),
                None,
                &request(),
            )
            .await;
        assert!(matches!(
            result,
            Err(ProviderError::InvalidConfiguration(_))
        ));
    }
}

#[tokio::test]
async fn loopback_http_can_be_disabled_explicitly() {
    let server = MockServer::start().await;
    let mut credentials = CredentialStore::default();
    credentials.set_default(ProviderId::OpenAi, credential(DEFAULT_KEY));
    let client = ProviderClient::with_options(
        credentials,
        ClientOptions {
            allow_http_loopback: false,
            ..ClientOptions::default()
        },
    )
    .expect("valid client");
    assert!(matches!(
        client
            .complete(&config(ProviderId::OpenAi, &server), None, &request())
            .await,
        Err(ProviderError::InvalidConfiguration(_))
    ));
    assert!(
        server
            .received_requests()
            .await
            .expect("request recording")
            .is_empty()
    );
}

#[tokio::test]
async fn model_identifiers_are_encoded_as_a_single_path_segment() {
    for provider in [ProviderId::Gemini, ProviderId::Vertex, ProviderId::Bedrock] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(fixture(provider)))
            .expect(1)
            .mount(&server)
            .await;
        let request = ChatRequest::new(
            "account/model?override=true#fragment",
            vec![Message::user("Hello")],
        );
        client(provider)
            .complete(&config(provider, &server), None, &request)
            .await
            .expect("encoded model path");
        let requests = server.received_requests().await.expect("request recording");
        let url = &requests[0].url;
        assert!(url.query().is_none());
        assert!(url.fragment().is_none());
        assert!(
            url.path()
                .contains("account%2Fmodel%3Foverride=true%23fragment")
        );
        assert_eq!(
            url.path_segments().expect("path segments").count(),
            if provider == ProviderId::Bedrock {
                4
            } else {
                3
            }
        );
    }
}

#[tokio::test]
async fn local_ollama_can_omit_credentials_but_explicit_projects_must_have_a_key() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture(ProviderId::Ollama)))
        .expect(1)
        .mount(&server)
        .await;
    let client = ProviderClient::new(CredentialStore::default()).expect("valid client");
    let config = config(ProviderId::Ollama, &server);
    client
        .complete(&config, None, &request())
        .await
        .expect("local inference without credential");
    let result = client
        .complete(&config, Some("project-a"), &request())
        .await;
    assert!(matches!(result, Err(ProviderError::Credential(_))));
    let requests = server.received_requests().await.expect("request recording");
    assert_eq!(requests.len(), 1);
    assert!(!requests[0].headers.contains_key("authorization"));
}
