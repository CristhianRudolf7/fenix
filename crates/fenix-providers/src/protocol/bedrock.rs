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
                .ok_or(CodecError::InvalidRequest("invalid Bedrock replay data"))?;
            validate_replay(message, decode_content(blocks)?)?;
            blocks.clone()
        } else {
            message
                .content
                .iter()
                .map(|block| match block {
                    ContentBlock::Text { text } => json!({"text": text}),
                    ContentBlock::ToolCall {
                        id,
                        name,
                        arguments,
                    } => json!({
                        "toolUse": {"toolUseId": id, "name": name, "input": arguments}
                    }),
                    ContentBlock::ToolResult {
                        tool_call_id,
                        content,
                        is_error,
                        ..
                    } => {
                        let result = match content.as_str() {
                            Some(text) => json!({"text": text}),
                            None => json!({"json": content}),
                        };
                        json!({"toolResult": {"toolUseId": tool_call_id, "content": [result],
                        "status": if *is_error { "error" } else { "success" }}})
                    }
                })
                .collect()
        };
        match message.role {
            Role::System => system.extend(content),
            Role::Assistant => push_message(&mut messages, "assistant", "content", content),
            Role::User | Role::Tool => push_message(&mut messages, "user", "content", content),
        }
    }
    let mut body =
        json!({"messages": messages, "inferenceConfig": {"maxTokens": request.max_output_tokens}});
    temperature(&mut body["inferenceConfig"], "temperature", request);
    if !system.is_empty() {
        body["system"] = Value::Array(system);
    }
    if !request.tools.is_empty() {
        body["toolConfig"] = json!({"tools": request.tools.iter().map(|tool| json!({
            "toolSpec": {"name": tool.name, "description": tool.description, "inputSchema": {"json": tool.parameters}}
        })).collect::<Vec<_>>()});
    }
    Ok(body)
}

fn decode_content(blocks: &[Value]) -> Result<Vec<ContentBlock>, CodecError> {
    let mut content = Vec::new();
    for block in blocks {
        if block.as_object().is_none_or(|object| object.len() != 1) {
            return Err(CodecError::InvalidResponse(
                "Bedrock content must have exactly one member",
            ));
        }
        if let Some(text) = block.get("text") {
            content.push(ContentBlock::Text {
                text: text
                    .as_str()
                    .ok_or(CodecError::InvalidResponse("invalid Bedrock text"))?
                    .to_owned(),
            });
        } else if let Some(call) = block.get("toolUse") {
            content.push(ContentBlock::ToolCall {
                id: nonempty_string(call, "toolUseId")?.to_owned(),
                name: nonempty_string(call, "name")?.to_owned(),
                arguments: object_arguments(&call["input"])?,
            });
        } else if let Some(reasoning) = block.get("reasoningContent") {
            if let Some(text) = reasoning.get("reasoningText") {
                string(text, "text")?;
                // Signed reasoning is replayed exactly, including its signature.
                optional_string(text, "signature")?;
            } else if !reasoning
                .get("redactedContent")
                .is_some_and(Value::is_string)
            {
                return Err(CodecError::InvalidResponse(
                    "invalid Bedrock reasoning content",
                ));
            }
        } else {
            return Err(CodecError::UnsupportedContent);
        }
    }
    Ok(content)
}

pub(super) fn decode(body: Value) -> Result<ChatResponse, CodecError> {
    let raw = &body["output"]["message"];
    if string(raw, "role")? != "assistant" {
        return Err(CodecError::InvalidResponse(
            "Bedrock output must have assistant role",
        ));
    }
    let blocks = array(raw, "content")?;
    let content = decode_content(blocks)?;
    let finish_reason = match string(&body, "stopReason")? {
        "end_turn" | "stop_sequence" => FinishReason::Stop,
        "tool_use" => FinishReason::ToolCalls,
        "max_tokens" | "model_context_window_exceeded" => FinishReason::Length,
        "content_filtered" | "guardrail_intervened" => FinishReason::ContentFilter,
        other => FinishReason::Other(other.to_owned()),
    };
    Ok(ChatResponse {
        id: optional_string(&body, "requestId")?,
        message: Message {
            role: Role::Assistant,
            content,
            provider_data: Some(ProviderData {
                protocol: Protocol::Bedrock,
                data: Value::Array(blocks.clone()),
            }),
        },
        usage: usage(
            body.get("usage"),
            "inputTokens",
            "outputTokens",
            "totalTokens",
        )?,
        finish_reason,
    })
}
