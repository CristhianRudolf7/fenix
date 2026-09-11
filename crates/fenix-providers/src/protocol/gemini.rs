use super::*;
use fenix_core::ProviderData;
use std::collections::HashMap;

pub(super) fn encode(request: &ChatRequest) -> Result<Value, CodecError> {
    let mut contents = Vec::new();
    let mut system = Vec::new();
    let mut call_names: HashMap<String, (String, Option<String>)> = HashMap::new();
    for message in &request.messages {
        if let Some(metadata) = &message.provider_data {
            validate_replay(message, decode_content(&metadata.data)?)?;
            remember_calls(message, Some(&metadata.data), &mut call_names)?;
            contents.push(metadata.data.clone());
            continue;
        }
        remember_calls(message, None, &mut call_names)?;
        let mut parts = Vec::new();
        for block in &message.content {
            parts.push(match block {
                ContentBlock::Text { text } => json!({"text": text}),
                ContentBlock::ToolCall {
                    id,
                    name,
                    arguments,
                } => json!({
                    "functionCall": {"id": id, "name": name, "args": arguments}
                }),
                ContentBlock::ToolResult {
                    tool_call_id,
                    name,
                    content,
                    is_error,
                } => {
                    let resolved = call_names.get(tool_call_id);
                    let call_name = match (name.as_deref(), resolved) {
                        (Some(name), Some((expected, _))) if name != expected => {
                            return Err(CodecError::InvalidRequest(
                                "tool result name does not match its call",
                            ));
                        }
                        (Some(name), _) => name,
                        (None, Some((name, _))) => name,
                        (None, None) => {
                            return Err(CodecError::InvalidRequest(
                                "Gemini tool results require a name or matching call",
                            ));
                        }
                    };
                    let result = if *is_error {
                        json!({"error": content})
                    } else if content.is_object() {
                        content.clone()
                    } else {
                        json!({"result": content})
                    };
                    let mut response = json!({"name": call_name, "response": result});
                    if let Some((_, Some(id))) = resolved {
                        response["id"] = json!(id);
                    }
                    json!({"functionResponse": response})
                }
            });
        }
        match message.role {
            Role::System => system.extend(parts),
            Role::Assistant => push_message(&mut contents, "model", "parts", parts),
            Role::User | Role::Tool => push_message(&mut contents, "user", "parts", parts),
        }
    }
    let mut body = json!({"contents": contents,
        "generationConfig": {"maxOutputTokens": request.max_output_tokens}});
    temperature(&mut body["generationConfig"], "temperature", request);
    if !system.is_empty() {
        body["systemInstruction"] = json!({"parts": system});
    }
    if !request.tools.is_empty() {
        body["tools"] = json!([{"functionDeclarations": request.tools.iter().map(|tool| json!({
            "name": tool.name, "description": tool.description, "parametersJsonSchema": tool.parameters
        })).collect::<Vec<_>>()}]);
    }
    Ok(body)
}

fn remember_calls(
    message: &Message,
    raw: Option<&Value>,
    calls: &mut HashMap<String, (String, Option<String>)>,
) -> Result<(), CodecError> {
    let raw_calls = match raw {
        Some(content) => array(content, "parts")?
            .iter()
            .filter_map(|part| part.get("functionCall"))
            .collect::<Vec<_>>(),
        None => Vec::new(),
    };
    for (index, block) in message
        .content
        .iter()
        .filter(|block| matches!(block, ContentBlock::ToolCall { .. }))
        .enumerate()
    {
        if let ContentBlock::ToolCall { id, name, .. } = block {
            let wire_id = match raw {
                Some(_) => raw_calls
                    .get(index)
                    .and_then(|call| call.get("id"))
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                None => Some(id.clone()),
            };
            calls.insert(id.clone(), (name.clone(), wire_id));
        }
    }
    Ok(())
}

fn decode_content(raw: &Value) -> Result<Vec<ContentBlock>, CodecError> {
    if string(raw, "role")? != "model" {
        return Err(CodecError::InvalidResponse(
            "Gemini output must have model role",
        ));
    }
    let mut content = Vec::new();
    for (index, part) in array(raw, "parts")?.iter().enumerate() {
        if let Some(call) = part.get("functionCall") {
            let name = nonempty_string(call, "name")?.to_owned();
            let id = match optional_string(call, "id")? {
                Some(id) if id.is_empty() => {
                    return Err(CodecError::InvalidResponse("required identifier is empty"));
                }
                Some(id) => id,
                None => format!("gemini_call_{index}_{name}"),
            };
            let arguments = match call.get("args") {
                None => json!({}),
                Some(arguments) => object_arguments(arguments)?,
            };
            content.push(ContentBlock::ToolCall {
                id,
                name,
                arguments,
            });
        } else if let Some(text) = part.get("text") {
            let text = text
                .as_str()
                .ok_or(CodecError::InvalidResponse("invalid Gemini text part"))?;
            if part.get("thought").and_then(Value::as_bool) != Some(true) {
                content.push(ContentBlock::Text {
                    text: text.to_owned(),
                });
            }
        } else {
            return Err(CodecError::UnsupportedContent);
        }
        if let Some(signature) = part.get("thoughtSignature")
            && !signature.is_string()
        {
            return Err(CodecError::InvalidResponse(
                "invalid Gemini thought signature",
            ));
        }
    }
    Ok(content)
}

pub(super) fn decode(body: Value) -> Result<ChatResponse, CodecError> {
    let mut usage = usage(
        body.get("usageMetadata"),
        "promptTokenCount",
        "candidatesTokenCount",
        "totalTokenCount",
    )?;
    if let Some(output) = usage.output_tokens
        && let Some(thoughts) = optional_tokens(&body["usageMetadata"], "thoughtsTokenCount")?
    {
        usage.output_tokens = Some(
            output
                .checked_add(thoughts)
                .ok_or(CodecError::InvalidResponse("output token count overflow"))?,
        );
    }
    let id = optional_string(&body, "responseId")?;
    let candidate = body
        .get("candidates")
        .and_then(Value::as_array)
        .and_then(|candidates| candidates.first());
    if candidate.is_none()
        && let Some(reason) = body
            .get("promptFeedback")
            .and_then(|feedback| feedback.get("blockReason"))
            .and_then(Value::as_str)
        && reason != "BLOCK_REASON_UNSPECIFIED"
    {
        return Ok(ChatResponse {
            id,
            message: Message {
                role: Role::Assistant,
                content: Vec::new(),
                provider_data: None,
            },
            usage,
            finish_reason: FinishReason::ContentFilter,
        });
    }
    let candidate = candidate.ok_or(CodecError::InvalidResponse("no Gemini candidate"))?;
    let finish_reason = match string(candidate, "finishReason")? {
        "STOP" => FinishReason::Stop,
        "MAX_TOKENS" => FinishReason::Length,
        "SAFETY" | "RECITATION" | "BLOCKLIST" | "PROHIBITED_CONTENT" | "SPII" | "IMAGE_SAFETY" => {
            FinishReason::ContentFilter
        }
        other => FinishReason::Other(other.to_owned()),
    };
    let raw = candidate.get("content");
    let (content, provider_data) = match raw {
        Some(raw) => (
            decode_content(raw)?,
            Some(ProviderData {
                protocol: Protocol::Gemini,
                data: raw.clone(),
            }),
        ),
        None if matches!(
            finish_reason,
            FinishReason::ContentFilter | FinishReason::Length
        ) =>
        {
            (Vec::new(), None)
        }
        None => {
            return Err(CodecError::InvalidResponse(
                "Gemini candidate has no content",
            ));
        }
    };
    let finish_reason = if finish_reason == FinishReason::Stop && has_tool_calls(&content) {
        FinishReason::ToolCalls
    } else {
        finish_reason
    };
    Ok(ChatResponse {
        id,
        message: Message {
            role: Role::Assistant,
            content,
            provider_data,
        },
        usage,
        finish_reason,
    })
}
