use super::*;
use fenix_core::ProviderData;

pub(super) fn encode(request: &ChatRequest) -> Result<Value, CodecError> {
    let mut system = Vec::new();
    let mut messages = Vec::new();
    for message in &request.messages {
        let content = if let Some(metadata) = &message.provider_data {
            let blocks = metadata
                .data
                .as_array()
                .ok_or(CodecError::InvalidRequest("invalid Anthropic replay data"))?;
            validate_replay(message, decode_content(blocks)?)?;
            blocks.clone()
        } else {
            message
                .content
                .iter()
                .map(|block| match block {
                    ContentBlock::Text { text } => json!({"type": "text", "text": text}),
                    ContentBlock::ToolCall {
                        id,
                        name,
                        arguments,
                    } => json!({"type": "tool_use", "id": id, "name": name, "input": arguments}),
                    ContentBlock::ToolResult {
                        tool_call_id,
                        content,
                        is_error,
                        ..
                    } => json!({
                        "type": "tool_result", "tool_use_id": tool_call_id,
                        "content": tool_result_text(content, false), "is_error": is_error
                    }),
                })
                .collect()
        };
        match message.role {
            Role::System => system.extend(content),
            Role::Assistant => push_message(&mut messages, "assistant", "content", content),
            Role::User | Role::Tool => push_message(&mut messages, "user", "content", content),
        }
    }
    let mut body = json!({"model": request.model, "messages": messages,
        "max_tokens": request.max_output_tokens, "stream": false});
    if !system.is_empty() {
        body["system"] = Value::Array(system);
    }
    if !request.tools.is_empty() {
        body["tools"] = Value::Array(request.tools.iter().map(|tool| json!({
            "name": tool.name, "description": tool.description, "input_schema": tool.parameters
        })).collect());
    }
    temperature(&mut body, "temperature", request);
    Ok(body)
}

fn decode_content(blocks: &[Value]) -> Result<Vec<ContentBlock>, CodecError> {
    let mut content = Vec::new();
    for block in blocks {
        match string(block, "type")? {
            "text" => content.push(ContentBlock::Text {
                text: string(block, "text")?.to_owned(),
            }),
            "tool_use" => content.push(ContentBlock::ToolCall {
                id: nonempty_string(block, "id")?.to_owned(),
                name: nonempty_string(block, "name")?.to_owned(),
                arguments: object_arguments(&block["input"])?,
            }),
            "thinking" => {
                string(block, "thinking")?;
                nonempty_string(block, "signature")?;
            }
            "redacted_thinking" => {
                nonempty_string(block, "data")?;
            }
            _ => return Err(CodecError::UnsupportedContent),
        }
    }
    Ok(content)
}

pub(super) fn decode(body: Value) -> Result<ChatResponse, CodecError> {
    if string(&body, "role")? != "assistant" {
        return Err(CodecError::InvalidResponse(
            "response message must have assistant role",
        ));
    }
    let blocks = array(&body, "content")?;
    let content = decode_content(blocks)?;
    let finish_reason = match string(&body, "stop_reason")? {
        "end_turn" | "stop_sequence" => FinishReason::Stop,
        "tool_use" => FinishReason::ToolCalls,
        "max_tokens" | "model_context_window_exceeded" => FinishReason::Length,
        "refusal" => FinishReason::ContentFilter,
        other => FinishReason::Other(other.to_owned()),
    };
    // Anthropic's input_tokens is the uncached input count. Cache creation and
    // cache reads are distinct input tokens, so include them when supplied.
    let mut usage = usage(
        body.get("usage"),
        "input_tokens",
        "output_tokens",
        "total_tokens",
    )?;
    if let Some(input) = usage.input_tokens {
        let details = &body["usage"];
        let cache_creation = optional_tokens(details, "cache_creation_input_tokens")?.unwrap_or(0);
        let cache_read = optional_tokens(details, "cache_read_input_tokens")?.unwrap_or(0);
        usage.input_tokens = Some(
            input
                .checked_add(cache_creation)
                .and_then(|value| value.checked_add(cache_read))
                .ok_or(CodecError::InvalidResponse("input token count overflow"))?,
        );
    }
    if usage.total_tokens.is_none()
        && let (Some(input), Some(output)) = (usage.input_tokens, usage.output_tokens)
    {
        usage.total_tokens = Some(
            input
                .checked_add(output)
                .ok_or(CodecError::InvalidResponse("total token count overflow"))?,
        );
    }
    Ok(ChatResponse {
        id: optional_string(&body, "id")?,
        message: Message {
            role: Role::Assistant,
            content,
            provider_data: Some(ProviderData {
                protocol: Protocol::Anthropic,
                data: Value::Array(blocks.clone()),
            }),
        },
        usage,
        finish_reason,
    })
}
