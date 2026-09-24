use super::*;
use fenix_core::{ProviderData, ToolDefinition};

const PROTOCOLS: [Protocol; 5] = [
    Protocol::OpenAiResponses,
    Protocol::OpenAiChat,
    Protocol::Anthropic,
    Protocol::Gemini,
    Protocol::Bedrock,
];

// Synthetic fixtures follow the official REST response schemas. They contain no
// account data, and are not recordings of live provider requests.
// OpenAI: https://developers.openai.com/api/docs/guides/function-calling
// Claude: https://platform.claude.com/docs/en/agents-and-tools/tool-use/handle-tool-calls
// Gemini: https://ai.google.dev/api/generate-content
// Bedrock: https://docs.aws.amazon.com/bedrock/latest/APIReference/API_runtime_Converse.html
fn tool_fixture(protocol: Protocol) -> Value {
    match protocol {
        Protocol::OpenAiResponses => {
            json!({"id": "resp_fixture", "status": "completed", "output": [
            {"type": "reasoning", "id": "rs_fixture", "summary": [], "encrypted_content": "opaque-replay-token"},
            {"type": "function_call", "id": "fc_fixture", "call_id": "call_1", "name": "get_weather", "arguments": "{\"city\":\"Lima\"}", "status": "completed"}
        ], "usage": {"input_tokens": 17, "output_tokens": 8, "total_tokens": 25}})
        }
        Protocol::OpenAiChat => json!({"id": "chatcmpl_fixture", "choices": [{"index": 0,
            "message": {"role": "assistant", "content": null, "reasoning_content": "fixture reasoning",
                "tool_calls": [{"id": "call_1", "type": "function", "function": {"name": "get_weather", "arguments": "{\"city\":\"Lima\"}"}}]},
            "finish_reason": "tool_calls"}], "usage": {"prompt_tokens": 17, "completion_tokens": 8, "total_tokens": 25}}),
        Protocol::Anthropic => {
            json!({"id": "msg_fixture", "type": "message", "role": "assistant", "content": [
            {"type": "thinking", "thinking": "fixture reasoning", "signature": "opaque-replay-token"},
            {"type": "tool_use", "id": "call_1", "name": "get_weather", "input": {"city": "Lima"}}
        ], "stop_reason": "tool_use", "usage": {"input_tokens": 17, "output_tokens": 8}})
        }
        Protocol::Gemini => {
            json!({"responseId": "gemini_fixture", "candidates": [{"content": {"role": "model", "parts": [
            {"functionCall": {"id": "call_1", "name": "get_weather", "args": {"city": "Lima"}}, "thoughtSignature": "opaque-replay-token"}
        ]}, "finishReason": "STOP"}], "usageMetadata": {"promptTokenCount": 17, "candidatesTokenCount": 8, "totalTokenCount": 25}})
        }
        Protocol::Bedrock => json!({"output": {"message": {"role": "assistant", "content": [
            {"reasoningContent": {"reasoningText": {"text": "fixture reasoning", "signature": "opaque-replay-token"}}},
            {"toolUse": {"toolUseId": "call_1", "name": "get_weather", "input": {"city": "Lima"}}}
        ]}}, "stopReason": "tool_use", "usage": {"inputTokens": 17, "outputTokens": 8, "totalTokens": 25}}),
    }
}

fn text_fixture(protocol: Protocol) -> Value {
    match protocol {
        Protocol::OpenAiResponses => json!({"id": "resp_text", "status": "completed", "output": [
            {"type": "message", "role": "assistant", "content": [{"type": "output_text", "text": "Hello", "annotations": []}]}
        ]}),
        Protocol::OpenAiChat => {
            json!({"choices": [{"message": {"role": "assistant", "content": "Hello"}, "finish_reason": "stop"}]})
        }
        Protocol::Anthropic => {
            json!({"role": "assistant", "content": [{"type": "text", "text": "Hello"}], "stop_reason": "end_turn"})
        }
        Protocol::Gemini => {
            json!({"candidates": [{"content": {"role": "model", "parts": [{"text": "Hello"}]}, "finishReason": "STOP"}]})
        }
        Protocol::Bedrock => {
            json!({"output": {"message": {"role": "assistant", "content": [{"text": "Hello"}]}}, "stopReason": "end_turn"})
        }
    }
}

fn base_request() -> ChatRequest {
    ChatRequest {
        model: "test-model".to_owned(),
        messages: vec![
            Message::text(Role::System, "Be accurate."),
            Message::user("What is the weather in Lima?"),
        ],
        tools: vec![ToolDefinition {
            name: "get_weather".to_owned(),
            description: "Read the weather.".to_owned(),
            parameters: json!({"type": "object", "properties": {"city": {"type": "string"}, "units": {"type": "string"}}, "required": ["city"]}),
        }],
        max_output_tokens: 512,
        temperature: Some(0.2),
        reasoning_effort: None,
        flex_processing: false,
    }
}

#[test]
fn responses_preserves_max_reasoning_configuration() {
    let mut request = ChatRequest::new("gpt-6-luna", vec![Message::user("Hello")]);
    request.reasoning_effort = Some(serde_json::from_str("\"max\"").unwrap());
    let body = encode(Protocol::OpenAiResponses, &request).unwrap();
    assert_eq!(body["reasoning"]["effort"], "max");
}

#[test]
fn responses_sends_explicit_low_reasoning_without_temperature() {
    let mut request = ChatRequest::new("gpt-5.6-luna", vec![Message::user("Hello")]);
    request.reasoning_effort = Some(fenix_core::ReasoningEffort::Low);
    let body = encode(Protocol::OpenAiResponses, &request).unwrap();
    assert_eq!(body["reasoning"]["effort"], "low");
    assert!(body.get("temperature").is_none());
    assert_eq!(body["store"], false);
}

fn tool_result(id: &str, is_error: bool) -> Message {
    Message {
        role: Role::Tool,
        content: vec![ContentBlock::ToolResult {
            tool_call_id: id.to_owned(),
            name: None,
            content: json!({"temperature": 22}),
            is_error,
        }],
        provider_data: None,
    }
}

fn round_trip(protocol: Protocol) -> Value {
    let response = decode(protocol, tool_fixture(protocol)).unwrap();
    assert_eq!(response.finish_reason, FinishReason::ToolCalls);
    assert_eq!(
        response.message.content,
        vec![ContentBlock::ToolCall {
            id: "call_1".to_owned(),
            name: "get_weather".to_owned(),
            arguments: json!({"city": "Lima"}),
        }]
    );
    assert_eq!(response.usage.input_tokens, Some(17));
    assert_eq!(response.usage.output_tokens, Some(8));
    let mut request = base_request();
    request.messages.push(response.message);
    request.messages.push(tool_result("call_1", false));
    encode(protocol, &request).unwrap()
}

#[test]
fn responses_tool_round_trip_preserves_encrypted_reasoning() {
    let body = round_trip(Protocol::OpenAiResponses);
    assert_eq!(body["input"][2]["encrypted_content"], "opaque-replay-token");
    assert_eq!(body["input"][3]["call_id"], "call_1");
    assert_eq!(
        body["input"][4],
        json!({"type": "function_call_output", "call_id": "call_1", "output": "{\"temperature\":22}"})
    );
    assert_eq!(body["tools"][0]["strict"], false);
    assert_eq!(body["tools"][0]["parameters"]["required"], json!(["city"]));
    assert_eq!(body["store"], false);
}

#[test]
fn chat_tool_round_trip_preserves_reasoning_content() {
    let body = round_trip(Protocol::OpenAiChat);
    assert_eq!(
        body["messages"][2]["reasoning_content"],
        "fixture reasoning"
    );
    assert_eq!(
        body["messages"][3],
        json!({"role": "tool", "tool_call_id": "call_1", "content": "{\"temperature\":22}"})
    );
    assert_eq!(
        body["tools"][0]["function"]["parameters"],
        base_request().tools[0].parameters
    );
}

#[test]
fn anthropic_tool_round_trip_preserves_signed_thinking() {
    let body = round_trip(Protocol::Anthropic);
    assert_eq!(
        body["system"],
        json!([{"type": "text", "text": "Be accurate."}])
    );
    assert_eq!(
        body["messages"][1]["content"][0]["signature"],
        "opaque-replay-token"
    );
    assert_eq!(
        body["messages"][2],
        json!({"role": "user", "content": [{
            "type": "tool_result", "tool_use_id": "call_1", "content": "{\"temperature\":22}", "is_error": false
        }]})
    );
}

#[test]
fn gemini_tool_round_trip_preserves_thought_signature_and_call_id() {
    let body = round_trip(Protocol::Gemini);
    assert_eq!(
        body["systemInstruction"]["parts"][0]["text"],
        "Be accurate."
    );
    assert_eq!(
        body["contents"][1]["parts"][0]["thoughtSignature"],
        "opaque-replay-token"
    );
    assert_eq!(
        body["contents"][2]["parts"][0],
        json!({"functionResponse": {
            "id": "call_1", "name": "get_weather", "response": {"temperature": 22}
        }})
    );
    assert_eq!(
        body["tools"][0]["functionDeclarations"][0]["parametersJsonSchema"],
        base_request().tools[0].parameters
    );
    assert!(body.get("model").is_none());
}

#[test]
fn bedrock_tool_round_trip_preserves_signed_reasoning() {
    let body = round_trip(Protocol::Bedrock);
    assert_eq!(
        body["messages"][1]["content"][0]["reasoningContent"]["reasoningText"]["signature"],
        "opaque-replay-token"
    );
    assert_eq!(
        body["messages"][2]["content"][0],
        json!({"toolResult": {
            "toolUseId": "call_1", "content": [{"json": {"temperature": 22}}], "status": "success"
        }})
    );
    assert_eq!(
        body["toolConfig"]["tools"][0]["toolSpec"]["inputSchema"]["json"],
        base_request().tools[0].parameters
    );
    assert!(body.get("model").is_none());
}

#[test]
fn all_protocols_decode_text_without_inventing_usage() {
    for protocol in PROTOCOLS {
        let response = decode(protocol, text_fixture(protocol)).unwrap();
        assert_eq!(response.message.text_content(), "Hello");
        assert_eq!(response.usage, Usage::default());
        assert_eq!(response.finish_reason, FinishReason::Stop);
    }
}

#[test]
fn all_protocols_reject_malformed_success_envelopes() {
    for protocol in PROTOCOLS {
        for invalid in [
            json!({}),
            json!([]),
            json!({"error": {"message": "secret-provider-text"}}),
        ] {
            let error = decode(protocol, invalid).unwrap_err();
            assert!(!error.to_string().contains("secret-provider-text"));
        }
    }
    let malformed = [
        (
            Protocol::OpenAiResponses,
            json!({"status": "completed", "output": []}),
        ),
        (
            Protocol::OpenAiChat,
            json!({"choices": [{"message": {"role": "assistant", "content": "Hello"}}]}),
        ),
        (
            Protocol::Anthropic,
            json!({"role": "assistant", "content": [], "stop_reason": "end_turn"}),
        ),
        (
            Protocol::Gemini,
            json!({"candidates": [{"content": {"role": "model", "parts": []}, "finishReason": "STOP"}]}),
        ),
        (
            Protocol::Bedrock,
            json!({"output": {"message": {"role": "assistant", "content": [{}]}}, "stopReason": "end_turn"}),
        ),
    ];
    for (protocol, body) in malformed {
        assert!(decode(protocol, body).is_err(), "{protocol:?}");
    }
}

#[test]
fn cross_protocol_metadata_is_rejected_for_every_protocol_pair() {
    for source in PROTOCOLS {
        let response = decode(source, tool_fixture(source)).unwrap();
        let mut request = base_request();
        request.messages.push(response.message);
        for target in PROTOCOLS {
            if source != target {
                assert_eq!(
                    encode(target, &request),
                    Err(CodecError::CrossProtocolReplay)
                );
            }
        }
    }
}

#[test]
fn edited_messages_cannot_replay_stale_provider_data() {
    for protocol in PROTOCOLS {
        let mut response = decode(protocol, tool_fixture(protocol)).unwrap();
        response.message.content.push(ContentBlock::Text {
            text: "Changed".to_owned(),
        });
        let mut request = base_request();
        request.messages.push(response.message);
        assert_eq!(
            encode(protocol, &request),
            Err(CodecError::StaleProviderData)
        );
    }
}

#[test]
fn gemini_synthesizes_local_ids_without_sending_them_as_provider_ids() {
    let mut fixture = tool_fixture(Protocol::Gemini);
    fixture["candidates"][0]["content"]["parts"][0]["functionCall"]
        .as_object_mut()
        .unwrap()
        .remove("id");
    let response = decode(Protocol::Gemini, fixture).unwrap();
    let ContentBlock::ToolCall { id, .. } = &response.message.content[0] else {
        panic!("expected tool call")
    };
    let result = tool_result(id, false);
    let mut request = base_request();
    request.messages.push(response.message);
    request.messages.push(result);
    let body = encode(Protocol::Gemini, &request).unwrap();
    assert!(
        body["contents"][2]["parts"][0]["functionResponse"]
            .get("id")
            .is_none()
    );
    assert_eq!(
        body["contents"][2]["parts"][0]["functionResponse"]["name"],
        "get_weather"
    );
}

#[test]
fn gemini_does_not_expose_thought_text_as_the_answer() {
    let body = json!({"candidates": [{"content": {"role": "model", "parts": [
        {"text": "internal fixture", "thought": true, "thoughtSignature": "opaque"}, {"text": "Hello"}
    ]}, "finishReason": "STOP"}]});
    let response = decode(Protocol::Gemini, body).unwrap();
    assert_eq!(response.message.text_content(), "Hello");
    assert_eq!(
        response.message.provider_data.unwrap().data["parts"][0]["thoughtSignature"],
        "opaque"
    );
}

#[test]
fn blocked_gemini_response_is_not_an_empty_success() {
    let response = decode(
        Protocol::Gemini,
        json!({"promptFeedback": {"blockReason": "SAFETY"}}),
    )
    .unwrap();
    assert_eq!(response.finish_reason, FinishReason::ContentFilter);
    assert!(response.message.content.is_empty());
    assert!(
        decode(
            Protocol::Gemini,
            json!({"promptFeedback": {"blockReason": "BLOCK_REASON_UNSPECIFIED"}})
        )
        .is_err()
    );
}

#[test]
fn partial_responses_report_length_without_inventing_output() {
    let response = decode(
        Protocol::OpenAiResponses,
        json!({"status": "incomplete", "output": [],
        "incomplete_details": {"reason": "max_output_tokens"}, "usage": {"input_tokens": 7}}),
    )
    .unwrap();
    assert_eq!(response.finish_reason, FinishReason::Length);
    assert_eq!(
        response.usage,
        Usage {
            input_tokens: Some(7),
            output_tokens: None,
            total_tokens: None
        }
    );
}

#[test]
fn invalid_tool_arguments_are_not_silently_replaced() {
    let mut body = tool_fixture(Protocol::OpenAiChat);
    body["choices"][0]["message"]["tool_calls"][0]["function"]["arguments"] =
        json!("{invalid-secret-arguments}");
    let error = decode(Protocol::OpenAiChat, body).unwrap_err();
    assert!(!error.to_string().contains("secret"));
    let mut body = tool_fixture(Protocol::Anthropic);
    body["content"][1]["input"] = json!("not-an-object");
    assert!(decode(Protocol::Anthropic, body).is_err());
}

#[test]
fn anthropic_usage_includes_reported_cache_input_tokens() {
    let mut body = tool_fixture(Protocol::Anthropic);
    body["usage"]["cache_creation_input_tokens"] = json!(20);
    body["usage"]["cache_read_input_tokens"] = json!(30);
    let response = decode(Protocol::Anthropic, body).unwrap();
    assert_eq!(response.usage.input_tokens, Some(67));
    assert_eq!(response.usage.total_tokens, Some(75));
}

#[test]
fn anthropic_does_not_derive_totals_when_one_count_is_missing() {
    let mut body = text_fixture(Protocol::Anthropic);
    body["usage"] = json!({"input_tokens": 5});
    let response = decode(Protocol::Anthropic, body).unwrap();
    assert_eq!(response.usage.total_tokens, None);
    assert_eq!(response.usage.output_tokens, None);
}

#[test]
fn gemini_output_accounts_for_explicitly_reported_thought_tokens() {
    let mut body = text_fixture(Protocol::Gemini);
    body["usageMetadata"] = json!({"promptTokenCount": 5, "candidatesTokenCount": 3, "thoughtsTokenCount": 8, "totalTokenCount": 16});
    let response = decode(Protocol::Gemini, body.clone()).unwrap();
    assert_eq!(response.usage.output_tokens, Some(11));
    assert_eq!(response.usage.total_tokens, Some(16));
    body["usageMetadata"]
        .as_object_mut()
        .unwrap()
        .remove("candidatesTokenCount");
    let response = decode(Protocol::Gemini, body).unwrap();
    assert_eq!(response.usage.output_tokens, None);
}

#[test]
fn token_count_overflow_is_an_error() {
    let mut body = text_fixture(Protocol::Anthropic);
    body["usage"] = json!({"input_tokens": u64::MAX, "output_tokens": 1});
    assert!(decode(Protocol::Anthropic, body).is_err());
    let mut body = text_fixture(Protocol::Gemini);
    body["usageMetadata"] = json!({"candidatesTokenCount": u64::MAX, "thoughtsTokenCount": 1});
    assert!(decode(Protocol::Gemini, body).is_err());
}

#[test]
fn parallel_tool_results_share_one_user_turn() {
    for protocol in [Protocol::Anthropic, Protocol::Gemini, Protocol::Bedrock] {
        let mut request = base_request();
        request.messages.push(Message {
            role: Role::Assistant,
            provider_data: None,
            content: vec![
                ContentBlock::ToolCall {
                    id: "call_1".to_owned(),
                    name: "get_weather".to_owned(),
                    arguments: json!({"city": "Lima"}),
                },
                ContentBlock::ToolCall {
                    id: "call_2".to_owned(),
                    name: "get_weather".to_owned(),
                    arguments: json!({"city": "Cusco"}),
                },
            ],
        });
        request.messages.push(tool_result("call_1", false));
        request.messages.push(tool_result("call_2", true));
        let body = encode(protocol, &request).unwrap();
        let (key, content) = if protocol == Protocol::Gemini {
            ("contents", "parts")
        } else {
            ("messages", "content")
        };
        assert_eq!(body[key].as_array().unwrap().len(), 3);
        assert_eq!(body[key][2][content].as_array().unwrap().len(), 2);
        match protocol {
            Protocol::Anthropic => assert_eq!(body[key][2][content][1]["is_error"], true),
            Protocol::Gemini => assert_eq!(
                body[key][2][content][1]["functionResponse"]["response"]["error"],
                json!({"temperature": 22})
            ),
            Protocol::Bedrock => {
                assert_eq!(body[key][2][content][1]["toolResult"]["status"], "error")
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn invalid_input_is_rejected_before_a_request_is_built() {
    let mut request = base_request();
    request.max_output_tokens = 0;
    for protocol in PROTOCOLS {
        assert!(encode(protocol, &request).is_err());
    }
    request.max_output_tokens = 512;
    request.temperature = Some(f64::NAN);
    for protocol in PROTOCOLS {
        assert!(encode(protocol, &request).is_err());
    }
    request.temperature = None;
    request
        .messages
        .push(Message::text(Role::Tool, "unlinked result"));
    for protocol in PROTOCOLS {
        assert!(encode(protocol, &request).is_err());
    }
}

#[test]
fn metadata_on_user_messages_is_rejected() {
    let mut request = base_request();
    request.messages[1].provider_data = Some(ProviderData {
        protocol: Protocol::OpenAiChat,
        data: json!({}),
    });
    assert!(encode(Protocol::OpenAiChat, &request).is_err());
}

#[test]
fn responses_flex_is_explicit_and_opt_in() {
    let mut request =
        fenix_core::ChatRequest::new("gpt-6-astra", vec![fenix_core::Message::user("Hello")]);
    let normal = encode(Protocol::OpenAiResponses, &request).unwrap();
    assert!(normal["service_tier"].is_null());
    request.flex_processing = true;
    let flex = encode(Protocol::OpenAiResponses, &request).unwrap();
    assert_eq!(flex["service_tier"], "flex");
}
