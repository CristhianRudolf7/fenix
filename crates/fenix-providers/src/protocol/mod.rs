//! Request and response codecs. These modules do not handle credentials or I/O.

mod anthropic;
mod bedrock;
mod gemini;
mod openai;

use fenix_core::{
    ChatRequest, ChatResponse, ContentBlock, FinishReason, Message, Protocol, Role, Usage,
};
use serde_json::{Value, json};
use thiserror::Error;

/// Conversion failures deliberately omit provider payloads and conversation text.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CodecError {
    #[error("invalid request: {0}")]
    InvalidRequest(&'static str),
    #[error("invalid provider response: {0}")]
    InvalidResponse(&'static str),
    #[error("provider metadata cannot be replayed with another protocol")]
    CrossProtocolReplay,
    #[error("assistant content was changed without clearing its provider metadata")]
    StaleProviderData,
    #[error("the provider returned an unsupported content type")]
    UnsupportedContent,
}

pub fn encode(protocol: Protocol, request: &ChatRequest) -> Result<Value, CodecError> {
    validate_request(protocol, request)?;
    match protocol {
        Protocol::OpenAiResponses => openai::encode_responses(request),
        Protocol::OpenAiChat => openai::encode_chat(request),
        Protocol::Anthropic => anthropic::encode(request),
        Protocol::Gemini => gemini::encode(request),
        Protocol::Bedrock => bedrock::encode(request),
    }
}

pub fn decode(protocol: Protocol, body: Value) -> Result<ChatResponse, CodecError> {
    if !body.is_object() || body.get("error").is_some_and(|value| !value.is_null()) {
        return Err(CodecError::InvalidResponse("expected a success object"));
    }
    let response = match protocol {
        Protocol::OpenAiResponses => openai::decode_responses(body),
        Protocol::OpenAiChat => openai::decode_chat(body),
        Protocol::Anthropic => anthropic::decode(body),
        Protocol::Gemini => gemini::decode(body),
        Protocol::Bedrock => bedrock::decode(body),
    }?;
    // A chat generation may end after plain reasoning without text or calls; the
    // caller decides how to recover instead of failing the whole response.
    let reasoning_only = response
        .message
        .provider_data
        .as_ref()
        .is_some_and(|replay| {
            replay.protocol == Protocol::OpenAiChat
                && ["reasoning", "reasoning_content"].iter().any(|key| {
                    replay.data[key]
                        .as_str()
                        .is_some_and(|s| !s.trim().is_empty())
                })
        });
    if response.message.content.is_empty()
        && !reasoning_only
        && !matches!(
            response.finish_reason,
            FinishReason::Length | FinishReason::ContentFilter
        )
    {
        return Err(CodecError::InvalidResponse(
            "response contains no supported output",
        ));
    }
    if response.finish_reason == FinishReason::ToolCalls
        && !has_tool_calls(&response.message.content)
    {
        return Err(CodecError::InvalidResponse(
            "tool-call finish reason has no tool calls",
        ));
    }
    Ok(response)
}

fn validate_request(protocol: Protocol, request: &ChatRequest) -> Result<(), CodecError> {
    if request.model.trim().is_empty()
        || request.messages.is_empty()
        || request.max_output_tokens == 0
    {
        return Err(CodecError::InvalidRequest(
            "model, messages and output token budget are required",
        ));
    }
    if request
        .temperature
        .is_some_and(|value| !value.is_finite() || !(0.0..=2.0).contains(&value))
    {
        return Err(CodecError::InvalidRequest(
            "temperature must be finite and between zero and two",
        ));
    }
    if matches!(protocol, Protocol::Anthropic | Protocol::Bedrock)
        && request.temperature.is_some_and(|value| value > 1.0)
    {
        return Err(CodecError::InvalidRequest(
            "this protocol requires temperature between zero and one",
        ));
    }
    let mut seen_non_system = false;
    for message in &request.messages {
        if let Some(metadata) = &message.provider_data {
            if metadata.protocol != protocol {
                return Err(CodecError::CrossProtocolReplay);
            }
            if message.role != Role::Assistant {
                return Err(CodecError::InvalidRequest(
                    "provider metadata belongs to assistant messages",
                ));
            }
        }
        if message.role == Role::System {
            if seen_non_system {
                return Err(CodecError::InvalidRequest(
                    "system messages must precede conversation messages",
                ));
            }
        } else {
            seen_non_system = true;
        }
        if message.content.is_empty() && message.provider_data.is_none() {
            return Err(CodecError::InvalidRequest("message content is empty"));
        }
        for block in &message.content {
            match block {
                ContentBlock::Text { .. } if message.role == Role::Tool => {
                    return Err(CodecError::InvalidRequest(
                        "tool messages require tool result blocks",
                    ));
                }
                ContentBlock::ToolCall {
                    id,
                    name,
                    arguments,
                } => {
                    if message.role != Role::Assistant
                        || id.is_empty()
                        || name.is_empty()
                        || !arguments.is_object()
                    {
                        return Err(CodecError::InvalidRequest(
                            "tool calls require an assistant role, ID, name and object arguments",
                        ));
                    }
                }
                ContentBlock::ToolResult { tool_call_id, .. }
                    if message.role != Role::Tool || tool_call_id.is_empty() =>
                {
                    return Err(CodecError::InvalidRequest(
                        "tool results require a tool role and call ID",
                    ));
                }
                _ => {}
            }
        }
    }
    if !seen_non_system {
        return Err(CodecError::InvalidRequest(
            "at least one conversation message is required",
        ));
    }
    for tool in &request.tools {
        if tool.name.is_empty() || !tool.parameters.is_object() {
            return Err(CodecError::InvalidRequest(
                "tools require names and JSON Schema objects",
            ));
        }
    }
    Ok(())
}

fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str, CodecError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or(CodecError::InvalidResponse(
            "missing or invalid string field",
        ))
}

fn nonempty_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, CodecError> {
    let value = string(value, key)?;
    if value.is_empty() {
        return Err(CodecError::InvalidResponse("required identifier is empty"));
    }
    Ok(value)
}

fn array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, CodecError> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or(CodecError::InvalidResponse(
            "missing or invalid array field",
        ))
}

fn object_arguments(value: &Value) -> Result<Value, CodecError> {
    if value.is_object() {
        Ok(value.clone())
    } else {
        Err(CodecError::InvalidResponse(
            "tool arguments must be a JSON object",
        ))
    }
}

fn json_arguments(value: &Value, key: &str) -> Result<Value, CodecError> {
    let decoded: Value = serde_json::from_str(string(value, key)?)
        .map_err(|_| CodecError::InvalidResponse("tool arguments are not valid JSON"))?;
    object_arguments(&decoded)
}

fn optional_string(value: &Value, key: &str) -> Result<Option<String>, CodecError> {
    match value.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        _ => Err(CodecError::InvalidResponse("invalid optional string field")),
    }
}

fn optional_tokens(value: &Value, key: &str) -> Result<Option<u64>, CodecError> {
    match value.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_u64()
            .map(Some)
            .ok_or(CodecError::InvalidResponse("invalid token count")),
    }
}

fn usage(
    value: Option<&Value>,
    input: &str,
    output: &str,
    total: &str,
) -> Result<Usage, CodecError> {
    let Some(value) = value.filter(|value| !value.is_null()) else {
        return Ok(Usage::default());
    };
    if !value.is_object() {
        return Err(CodecError::InvalidResponse("usage must be an object"));
    }
    Ok(Usage {
        input_tokens: optional_tokens(value, input)?,
        output_tokens: optional_tokens(value, output)?,
        total_tokens: optional_tokens(value, total)?,
    })
}

fn has_tool_calls(content: &[ContentBlock]) -> bool {
    content
        .iter()
        .any(|block| matches!(block, ContentBlock::ToolCall { .. }))
}

fn text_content(message: &Message) -> String {
    message
        .content
        .iter()
        .filter_map(|block| match block {
            ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn tool_result_text(content: &Value, is_error: bool) -> String {
    if is_error {
        json!({"error": content}).to_string()
    } else {
        content
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| content.to_string())
    }
}

fn temperature(body: &mut Value, key: &str, request: &ChatRequest) {
    if let Some(value) = request.temperature {
        body[key] = json!(value);
    }
}

fn validate_replay(message: &Message, decoded: Vec<ContentBlock>) -> Result<(), CodecError> {
    if message.content != decoded {
        return Err(CodecError::StaleProviderData);
    }
    Ok(())
}

/// Group consecutive user/tool messages so parallel tool results share one turn.
fn push_message(messages: &mut Vec<Value>, role: &str, content_key: &str, content: Vec<Value>) {
    if role != "assistant"
        && role != "model"
        && let Some(last) = messages.last_mut()
        && last.get("role").and_then(Value::as_str) == Some(role)
        && let Some(previous) = last.get_mut(content_key).and_then(Value::as_array_mut)
    {
        previous.extend(content);
        return;
    }
    messages.push(json!({"role": role, content_key: content}));
}

#[cfg(test)]
mod tests;
