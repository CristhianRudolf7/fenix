//! Provider usage anchors are valid only for the same model, tools and message prefix.
use crate::{Runtime, config::ModelConnection};
use anyhow::Result;
use fenix_core::{Message, ToolDefinition};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize)]
struct Anchor {
    identity: String,
    prefix: String,
    count: usize,
    input_tokens: usize,
}
fn digest<T: Serialize>(value: &T) -> String {
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(value).unwrap_or_default())
    )
}
fn identity(model: &ModelConnection, tools: &[ToolDefinition]) -> String {
    digest(&(
        &model.settings.provider,
        &model.settings.model,
        &model.settings.base_url,
        tools,
    ))
}
impl Runtime {
    pub(crate) fn context_tokens(
        &self,
        task: &str,
        model: &ModelConnection,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<usize> {
        if let Some(anchor) = self.store.get::<Anchor>("usage_anchors", task)?
            && anchor.identity == identity(model, tools)
            && anchor.count <= messages.len()
            && anchor.prefix == digest(&&messages[..anchor.count])
        {
            return Ok(anchor.input_tokens.saturating_add(
                crate::active_context::estimated_tokens(&messages[anchor.count..], &[]),
            ));
        }
        Ok(crate::active_context::estimated_tokens(messages, tools))
    }
    pub(crate) fn anchor_usage(
        &self,
        task: &str,
        model: &ModelConnection,
        messages: &[Message],
        tools: &[ToolDefinition],
        tokens: Option<u64>,
    ) -> Result<()> {
        // Provider usage also includes transient operational data; retaining that charge is conservative.
        if let Some(tokens) = tokens.filter(|n| *n > 0) {
            self.store.put(
                "usage_anchors",
                task,
                &Anchor {
                    identity: identity(model, tools),
                    prefix: digest(&messages),
                    count: messages.len(),
                    input_tokens: usize::try_from(tokens).unwrap_or(usize::MAX),
                },
            )?;
        } else {
            self.store.delete("usage_anchors", task)?;
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fingerprint_covers_earlier_messages_and_tool_schemas() {
        let first = vec![Message::user("original"), Message::user("same tail")];
        let second = vec![Message::user("edited"), Message::user("same tail")];
        assert_ne!(digest(&first), digest(&second));
        assert_ne!(
            digest(&serde_json::json!({"type":"string"})),
            digest(&serde_json::json!({"type":"integer"}))
        );
    }
}
