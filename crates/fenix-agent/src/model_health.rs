//! Cached, credential-bound model availability. Polling status never calls providers.
use crate::{Runtime, config::ModelConnection};
use anyhow::Result;
use fenix_core::Message;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
const VALID_SECONDS: i64 = 6 * 60 * 60;
#[derive(Clone, Serialize, Deserialize)]
struct Health {
    identity: String,
    status: String,
    reason: String,
    checked_at: i64,
    next_check: i64,
    failures: u32,
}
fn now() -> i64 {
    chrono::Utc::now().timestamp()
}
fn failure_reason(error: &fenix_providers::ProviderError) -> &'static str {
    use fenix_providers::ProviderError::*;
    match error {
        Authentication { .. } | Credential(_) => "authentication",
        RateLimited { .. } => "rate_limit",
        Http { status: 402 } => "quota",
        Http { status: 404 } | InvalidConfiguration(_) | InvalidRequest(_) => "configuration",
        Timeout | Transport => "connection",
        _ => "provider",
    }
}
impl Runtime {
    async fn health_connection(&self, role: &str) -> Option<Arc<ModelConnection>> {
        if role == "intelligent" {
            self.model.read().await.clone()
        } else {
            self.fast_model().await.ok()
        }
    }
    fn health_identity(&self, _role: &str, model: &ModelConnection) -> Result<String> {
        Ok(model.health_identity.clone())
    }
    pub(crate) fn record_model_health(
        &self,
        role: &str,
        model: &ModelConnection,
        error: Option<&fenix_providers::ProviderError>,
    ) -> Result<()> {
        let identity = self.health_identity(role, model)?;
        let previous = self
            .store
            .get::<Health>("model_health", role)?
            .filter(|h| h.identity == identity);
        let failures = if error.is_some() {
            previous.map_or(1, |h| h.failures.saturating_add(1))
        } else {
            0
        };
        let reason = error.map(failure_reason).unwrap_or("");
        let delay = if error.is_none() {
            VALID_SECONDS
        } else if matches!(reason, "authentication" | "configuration" | "quota") {
            15 * 60
        } else {
            (30_i64.saturating_mul(1_i64 << failures.min(5))).min(15 * 60)
        };
        let at = now();
        self.store.put(
            "model_health",
            role,
            &Health {
                identity,
                status: if error.is_none() {
                    "available"
                } else {
                    "unavailable"
                }
                .into(),
                reason: reason.into(),
                checked_at: at,
                next_check: at + delay,
                failures,
            },
        )
    }
    pub async fn model_health_status(&self) -> Result<Value> {
        let mut roles = serde_json::Map::new();
        let mut ready = true;
        for role in ["intelligent", "fast"] {
            let value = if let Some(model) = self.health_connection(role).await {
                let identity = self.health_identity(role, &model)?;
                match self
                    .store
                    .get::<Health>("model_health", role)?
                    .filter(|h| h.identity == identity && h.checked_at <= now() + 5)
                {
                    Some(h) => {
                        let status = if h.status == "available" && h.next_check <= now()
                            || h.status == "checking" && h.next_check <= now()
                        {
                            "unchecked"
                        } else {
                            &h.status
                        };
                        json!({"status":status,"reason":h.reason,"checked_at":h.checked_at,"next_check":h.next_check})
                    }
                    None => json!({"status":"unchecked","reason":"","checked_at":null}),
                }
            } else {
                json!({"status":"unconfigured","reason":"configuration","checked_at":null})
            };
            ready &= value["status"] == "available";
            roles.insert(role.into(), value);
        }
        Ok(json!({"can_send":ready,"roles":roles}))
    }
    pub async fn check_model_health(&self, force: bool) -> Result<Value> {
        // Single-flight across startup, automatic recovery and manual retries.
        let Ok(_guard) = self.health_check.try_lock() else {
            return self.model_health_status().await;
        };
        for role in ["intelligent", "fast"] {
            let Some(model) = self.health_connection(role).await else {
                continue;
            };
            let identity = self.health_identity(role, &model)?;
            let previous = self
                .store
                .get::<Health>("model_health", role)?
                .filter(|h| h.identity == identity && h.checked_at <= now() + 5);
            if previous.as_ref().is_some_and(|h| {
                h.status != "checking"
                    && h.next_check > now()
                    && (!force || now() - h.checked_at < 30)
            }) {
                continue;
            }
            let probe_seconds = if model.settings.flex_processing {
                180
            } else {
                30
            };
            let at = now();
            let failures = previous.map_or(0, |h| h.failures);
            self.store.put(
                "model_health",
                role,
                &Health {
                    identity: identity.clone(),
                    status: "checking".into(),
                    reason: String::new(),
                    checked_at: at,
                    next_check: at + probe_seconds as i64 + 30,
                    failures,
                },
            )?;
            let mut request = model
                .settings
                .request(vec![Message::user("Reply with OK.")]);
            request.max_output_tokens = 1024;
            let config = model.settings.provider_config();
            let result = tokio::select! {_=self.stop.cancelled()=>return self.model_health_status().await,result=tokio::time::timeout(Duration::from_secs(probe_seconds),model.client.complete(&config,model.settings.project.as_deref(),&request))=>result};
            // A configuration edited during the probe must never inherit its old result.
            let Some(current) = self.health_connection(role).await else {
                continue;
            };
            if self.health_identity(role, &current)? != identity {
                continue;
            }
            match result {
                Ok(Ok(response)) if !response.message.text_content().trim().is_empty() => {
                    self.record_model_health(role, &model, None)?;
                    self.store.put("model_health_usage",&fenix_storage::id(),&json!({"role":role,"model":model.settings.model,"usage":response.usage,"checked_at":now()}))?;
                }
                Ok(Err(error)) => self.record_model_health(role, &model, Some(&error))?,
                Err(_) => self.record_model_health(
                    role,
                    &model,
                    Some(&fenix_providers::ProviderError::Timeout),
                )?,
                _ => self.record_model_health(
                    role,
                    &model,
                    Some(&fenix_providers::ProviderError::MalformedResponse),
                )?,
            }
        }
        self.model_health_status().await
    }
    pub(crate) async fn model_health_worker(self: Arc<Self>) {
        loop {
            let _ = self.check_model_health(false).await;
            tokio::select! {_=self.stop.cancelled()=>break,_=tokio::time::sleep(Duration::from_secs(30))=>{}}
        }
    }
}
