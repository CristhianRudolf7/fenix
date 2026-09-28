//! Model roles reuse the existing account-scoped connections and credentials.
use crate::{Runtime, config::ModelConnection};
use anyhow::Result;
use serde_json::{Value, json};
use std::sync::Arc;
impl Runtime {
    /// Shared fast role: conversation and inexpensive presentation tasks.
    /// Legacy /live/settings remains an alias to this same configuration.
    pub(crate) async fn fast_model(&self) -> Result<Arc<ModelConnection>> {
        self.ensure_interlocutor().await?;
        self.interlocutor_connection().await
    }
    pub async fn model_roles(&self) -> Result<Value> {
        let intelligent = self.model.read().await;
        Ok(json!({"fast":self.live_status()?, "intelligent":{
            "configured":intelligent.is_some(),
            "settings":intelligent.as_ref().map(|m| &m.settings)
        }}))
    }
}
