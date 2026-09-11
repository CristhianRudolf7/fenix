use super::*;
use fenix_core::ProviderData;

pub(super) fn encode_responses(request: &ChatRequest) -> Result<Value, CodecError> {
    let mut input = Vec::new();
    for message in &request.messages {
        if let Some(metadata) = &message.provider_data {
            let items = metadata
                .data
                .as_array()
                .ok_or(CodecError::InvalidRequest("invalid Responses replay data"))?;
            validate_replay(message, responses_content(items)?.0)?;
            input.extend(items.iter().cloned());
            continue;
        }
        for block in &message.content {
            match block {
                ContentBlock::Text { text } => {
                    let role = match message.role {
                        Role::System => "system",
                        Role::User => "user",
                        Role::Assistant => "assistant",
                        Role::Tool => return Err(CodecError::InvalidRequest("invalid tool text")),
                    };
                    input.push(json!({"role": role, "content": text}));
                }
                ContentBlock::ToolCall { id, name, arguments } => input.push(json!({
                    "type": "function_call", "call_id": id, "name": name, "arguments": arguments.to_string()
                })),
                ContentBlock::ToolResult { tool_call_id, content, is_error, .. } => input.push(json!({
                    "type": "function_call_output", "call_id": tool_call_id,
                    "output": tool_result_text(content, *is_error)
                })),
            }
        }
    }
    let mut body = json!({
        "model": request.model,
        "input": input,
        "max_output_tokens": request.max_output_tokens,
        "store": false,
        "stream": false,
        "include": ["reasoning.encrypted_content"]
    });
    if !request.tools.is_empty() {
        body["tools"] = Value::Array(
            request
                .tools
                .iter()
                .map(|tool| {
                    json!({
                        "type": "function", "name": tool.name, "description": tool.description,
                        "parameters": tool.parameters, "strict": false
                    })
                })
                .collect(),
        );
    }
    temperature(&mut body, "temperature", request);
    Ok(body)
}

fn responses_content(items: &[Value]) -> Result<(Vec<ContentBlock>, bool), CodecError> {
    let mut content = Vec::new();
    let mut refused = false;
    for item in items {
        match string(item, "type")? {
            "message" => {
                if string(item, "role")? != "assistant" {
                    return Err(CodecError::InvalidResponse(
                        "response message must have assistant role",
                    ));
                }
                for block in array(item, "content")? {
                    match string(block, "type")? {
                        "output_text" => content.push(ContentBlock::Text {
                            text: string(block, "text")?.to_owned(),
                        }),
                        "refusal" => {
                            refused = true;
                            content.push(ContentBlock::Text {
                                text: string(block, "refusal")?.to_owned(),
                            });
                        }
                        _ => return Err(CodecError::UnsupportedContent),
                    }
                }
            }
            "function_call" => content.push(ContentBlock::ToolCall {
                id: nonempty_string(item, "call_id")?.to_owned(),
                name: nonempty_string(item, "name")?.to_owned(),
                arguments: json_arguments(item, "arguments")?,
            }),
            "reasoning" => {
                array(item, "summary")?;
            }
            _ => return Err(CodecError::UnsupportedContent),
        }
    }
    Ok((content, refused))
}

pub(super) fn decode_responses(body: Value) -> Result<ChatResponse, CodecError> {
    let output = array(&body, "output")?;
    let (content, refused) = responses_content(output)?;
    let finish_reason = match string(&body, "status")? {
        "completed" if refused => FinishReason::ContentFilter,
        "completed" if has_tool_calls(&content) => FinishReason::ToolCalls,
        "completed" => FinishReason::Stop,
        "incomplete" => match string(&body["incomplete_details"], "reason")? {
            "max_output_tokens" => FinishReason::Length,
            "content_filter" => FinishReason::ContentFilter,
            reason => FinishReason::Other(reason.to_owned()),
        },
        _ => {
            return Err(CodecError::InvalidResponse(
                "response is not completed or incomplete",
            ));
        }
    };
    Ok(ChatResponse {
        id: optional_string(&body, "id")?,
        message: Message {
            role: Role::Assistant,
            content,
            provider_data: Some(ProviderData {
                protocol: Protocol::OpenAiResponses,
                data: Value::Array(output.clone()),
            }),
        },
        usage: usage(
            body.get("usage"),
            "input_tokens",
            "output_tokens",
            "total_tokens",
        )?,
        finish_reason,
    })
}

pub(super) fn encode_chat(request: &ChatRequest) -> Result<Value, CodecError> {
    let mut messages = Vec::new();
    for message in &request.messages {
        if let Some(metadata) = &message.provider_data {
            validate_replay(message, chat_content(&metadata.data)?.0)?;
            messages.push(chat_replay(&metadata.data));
            continue;
        }
        if message.role == Role::Tool {
            for block in &message.content {
                if let ContentBlock::ToolResult {
                    tool_call_id,
                    content,
                    is_error,
                    ..
                } = block
                {
                    messages.push(json!({"role": "tool", "tool_call_id": tool_call_id,
                        "content": tool_result_text(content, *is_error)}));
                }
            }
            continue;
        }
        let role = match message.role {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Tool => unreachable!(),
        };
        let text = text_content(message);
        let calls = message.content.iter().filter_map(|block| match block {
            ContentBlock::ToolCall { id, name, arguments } => Some(json!({
                "id": id, "type": "function", "function": {"name": name, "arguments": arguments.to_string()}
            })),
            _ => None,
        }).collect::<Vec<_>>();
        let mut item = json!({"role": role, "content": text});
        if !calls.is_empty() {
            if text.is_empty() {
                item["content"] = Value::Null;
            }
            item["tool_calls"] = Value::Array(calls);
        }
        messages.push(item);
    }
    let mut body = json!({"model": request.model, "messages": messages,
        "max_tokens": request.max_output_tokens, "stream": false});
    if !request.tools.is_empty() {
        body["tools"] = Value::Array(request.tools.iter().map(|tool| json!({
            "type": "function", "function": {"name": tool.name, "description": tool.description,
                "parameters": tool.parameters}
        })).collect());
    }
    temperature(&mut body, "temperature", request);
    Ok(body)
}

fn chat_replay(message: &Value) -> Value {
    let mut replay = json!({"role": "assistant"});
    // Keep reasoning extensions used by DeepSeek and OpenRouter, but do not copy
    // response-only fields such as log probabilities into a subsequent request.
    for key in [
        "content",
        "tool_calls",
        "refusal",
        "reasoning_content",
        "reasoning",
        "reasoning_details",
    ] {
        if let Some(value) = message.get(key) {
            replay[key] = value.clone();
        }
    }
    replay
}

fn chat_content(message: &Value) -> Result<(Vec<ContentBlock>, bool), CodecError> {
    if string(message, "role")? != "assistant" {
        return Err(CodecError::InvalidResponse(
            "response message must have assistant role",
        ));
    }
    let mut content = Vec::new();
    match message.get("content") {
        Some(Value::String(text)) if !text.is_empty() => {
            content.push(ContentBlock::Text { text: text.clone() })
        }
        Some(Value::Array(parts)) => {
            for part in parts {
                if string(part, "type")? != "text" {
                    return Err(CodecError::UnsupportedContent);
                }
                content.push(ContentBlock::Text {
                    text: string(part, "text")?.to_owned(),
                });
            }
        }
        None | Some(Value::Null) | Some(Value::String(_)) => {}
        _ => return Err(CodecError::InvalidResponse("invalid assistant content")),
    }
    if let Some(calls) = message.get("tool_calls").filter(|value| !value.is_null()) {
        for call in calls
            .as_array()
            .ok_or(CodecError::InvalidResponse("tool calls must be an array"))?
        {
            if string(call, "type")? != "function" {
                return Err(CodecError::UnsupportedContent);
            }
            content.push(ContentBlock::ToolCall {
                id: nonempty_string(call, "id")?.to_owned(),
                name: nonempty_string(&call["function"], "name")?.to_owned(),
                arguments: json_arguments(&call["function"], "arguments")?,
            });
        }
    }
    let refused = optional_string(message, "refusal")?;
    if let Some(refusal) = &refused {
        content.push(ContentBlock::Text {
            text: refusal.clone(),
        });
    }
    Ok((content, refused.is_some()))
}

pub(super) fn decode_chat(body: Value) -> Result<ChatResponse, CodecError> {
    let choice = array(&body, "choices")?
        .first()
        .ok_or(CodecError::InvalidResponse("no response choice"))?;
    let message = &choice["message"];
    let (content, refused) = chat_content(message)?;
    let finish_reason = match string(choice, "finish_reason")? {
        "content_filter" => FinishReason::ContentFilter,
        "length" => FinishReason::Length,
        _ if refused => FinishReason::ContentFilter,
        "tool_calls" => FinishReason::ToolCalls,
        "stop" if has_tool_calls(&content) => FinishReason::ToolCalls,
        "stop" => FinishReason::Stop,
        other => FinishReason::Other(other.to_owned()),
    };
    Ok(ChatResponse {
        id: optional_string(&body, "id")?,
        message: Message {
            role: Role::Assistant,
            content,
            provider_data: Some(ProviderData {
                protocol: Protocol::OpenAiChat,
                data: chat_replay(message),
            }),
        },
        usage: usage(
            body.get("usage"),
            "prompt_tokens",
            "completion_tokens",
            "total_tokens",
        )?,
        finish_reason,
    })
}
