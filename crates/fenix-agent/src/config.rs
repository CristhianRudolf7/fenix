use crate::Runtime;
use anyhow::{Result, bail};
use fenix_core::{ChatRequest, Message, ProviderId, ReasoningEffort};
use fenix_credentials::{ApiKey, Credential, CredentialStore};
use fenix_providers::{ProviderClient, ProviderConfig, catalog::descriptor};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{path::PathBuf, sync::Arc};

pub const DEFAULT_MODEL: &str = "gpt-5.6-luna";

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub provider: ProviderId,
    pub model: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub provider_project: Option<String>,
    #[serde(default)]
    pub organization: Option<String>,
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub reasoning_effort: Option<ReasoningEffort>,
    #[serde(default)]
    pub flex_processing: bool,
    pub workspace: PathBuf,
    #[serde(default = "default_commands")]
    pub command_policy: ActionPolicy,
    #[serde(default = "default_approval")]
    pub file_write_policy: ActionPolicy,
    #[serde(default)]
    pub coding: CodingSettings,
    #[serde(default)]
    pub execution: ExecutionSettings,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ExecutionSettings {
    /// Opt-in external supervision: no model-step, task, or HTTP request deadline.
    pub externally_supervised: bool,
    pub max_model_steps: usize,
    pub task_timeout_seconds: u64,
    pub subagent_timeout_seconds: u64,
    pub command_timeout_seconds: u64,
    /// Initial foreground wait; recent short commands may extend it up to 10 seconds.
    pub command_yield_ms: u64,
    pub service_timeout_seconds: u64,
    pub max_output_tokens: u32,
}
impl Default for ExecutionSettings {
    fn default() -> Self {
        Self {
            externally_supervised: false,
            max_model_steps: 128,
            task_timeout_seconds: 14_400,
            subagent_timeout_seconds: 3_600,
            command_timeout_seconds: 1_800,
            command_yield_ms: 1_000,
            service_timeout_seconds: 86_400,
            max_output_tokens: 16_384,
        }
    }
}
impl ExecutionSettings {
    pub fn validate(&self) -> Result<()> {
        if !(8..=4096).contains(&self.max_model_steps)
            || !(60..=604_800).contains(&self.task_timeout_seconds)
            || !(60..=604_800).contains(&self.subagent_timeout_seconds)
            || self.command_yield_ms > 30_000
            || !(1..=604_800).contains(&self.command_timeout_seconds)
            || !(1..=604_800).contains(&self.service_timeout_seconds)
            || !(1024..=1_048_576).contains(&self.max_output_tokens)
        {
            bail!("Invalid execution limits; timeouts must not exceed seven days");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CodingSettings {
    pub language_servers: Vec<crate::coding::lsp::ServerConfig>,
    /// Operator-supplied model context capacity; conservative estimates reserve headroom.
    pub context_window_tokens: usize,
    pub compact_at_tokens: usize,
    pub keep_recent_tokens: usize,
}
impl Default for CodingSettings {
    fn default() -> Self {
        Self {
            language_servers: Vec::new(),
            context_window_tokens: 200_000,
            compact_at_tokens: 160_000,
            keep_recent_tokens: 6_000,
        }
    }
}
impl CodingSettings {
    pub fn validate(&self) -> Result<()> {
        if self.context_window_tokens < 16_384
            || self.context_window_tokens > 2_000_000
            || self.compact_at_tokens < 8_000
            || self.compact_at_tokens > self.context_window_tokens.saturating_sub(8_192)
            || self.keep_recent_tokens < 1_000
            || self.keep_recent_tokens > self.compact_at_tokens / 2
            || self.language_servers.len() > 16
        {
            bail!("Invalid coding context budget or too many language servers");
        }
        let mut extensions = std::collections::HashSet::new();
        for server in &self.language_servers {
            server.validate()?;
            for extension in &server.extensions {
                if !extensions.insert(extension) {
                    bail!("Each extension must have only one configured language server");
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionPolicy {
    Ask,
    Allow,
}
fn default_commands() -> ActionPolicy {
    ActionPolicy::Allow
}
fn default_approval() -> ActionPolicy {
    ActionPolicy::Ask
}

impl Settings {
    pub fn initial(workspace: PathBuf) -> Self {
        Self {
            provider: ProviderId::OpenAi,
            model: DEFAULT_MODEL.into(),
            base_url: None,
            provider_project: None,
            organization: None,
            project: None,
            reasoning_effort: Some(ReasoningEffort::Low),
            flex_processing: false,
            workspace,
            command_policy: ActionPolicy::Allow,
            file_write_policy: ActionPolicy::Ask,
            coding: CodingSettings::default(),
            execution: ExecutionSettings::default(),
        }
    }
    pub fn request(&self, messages: Vec<Message>) -> ChatRequest {
        let mut request = ChatRequest::new(&self.model, messages);
        request.max_output_tokens = self.execution.max_output_tokens;
        if self.provider == ProviderId::OpenAi {
            request.reasoning_effort = self.reasoning_effort;
            request.flex_processing = self.flex_processing;
        }
        request
    }
    pub fn provider_config(&self) -> ProviderConfig {
        ProviderConfig {
            provider: self.provider,
            base_url: self.base_url.clone(),
        }
    }
    pub fn credential_id(&self) -> String {
        match &self.project {
            Some(project) => format!("{}:project:{project}", self.provider),
            None => format!("{}:default", self.provider),
        }
    }
}

pub struct ModelConnection {
    pub(crate) health_identity: String,
    pub settings: Settings,
    pub client: ProviderClient,
}

impl Runtime {
    pub fn settings(&self) -> Result<Option<Settings>> {
        self.store.get("settings", "active")
    }

    pub fn model_connection(&self, settings: Settings, key: ApiKey) -> Result<ModelConnection> {
        let health_identity = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&(
                settings.provider_config(),
                &settings.model,
                settings.reasoning_effort,
                settings.flex_processing,
                &settings.provider_project,
                &settings.organization,
                &settings.project,
                key.expose_secret(),
            ))?)
        );
        let mut credential = Credential::inline(key);
        credential.provider_project = settings.provider_project.clone();
        credential.organization = settings.organization.clone();
        let mut store = CredentialStore::default();
        if let Some(project) = &settings.project {
            store.set_project(project, settings.provider, credential)?;
        } else {
            store.set_default(settings.provider, credential);
        }
        Ok(ModelConnection {
            health_identity,
            client: ProviderClient::with_options(
                store,
                fenix_providers::ClientOptions {
                    disable_request_timeout: settings.execution.externally_supervised,
                    ..Default::default()
                },
            )?
            .with_redactor(self.redactor.clone()),
            settings,
        })
    }

    pub async fn configure(&self, mut settings: Settings, key: Option<String>) -> Result<Value> {
        let _guard = self.configure_lock.lock().await;
        if !self.active.lock().await.is_empty()
            || self
                .store
                .tasks(200)?
                .iter()
                .any(|t| matches!(t.status.as_str(), "queued" | "running" | "waiting"))
            || !self.subagents.active.lock().await.is_empty()
        {
            bail!("Wait for queued or active tasks before changing model configuration");
        }
        settings.coding.validate()?;
        settings.execution.validate()?;
        if settings.execution.max_output_tokens as usize >= settings.coding.context_window_tokens {
            bail!("Output limit must be smaller than model context capacity");
        }
        settings.model = settings.model.trim().to_owned();
        if settings.model.is_empty() || settings.model.len() > 512 {
            bail!("Enter a model identifier");
        }
        settings.workspace = std::fs::canonicalize(&settings.workspace)?;
        self.validate_user_workspace(&settings.workspace)?;
        if !settings.workspace.is_dir() {
            bail!("Workspace must be an existing directory");
        }
        if settings.provider != ProviderId::OpenAi {
            settings.reasoning_effort = None;
        }
        for item in [
            &mut settings.base_url,
            &mut settings.provider_project,
            &mut settings.organization,
            &mut settings.project,
        ] {
            if item.as_ref().is_some_and(|s| s.trim().is_empty()) {
                *item = None;
            }
        }
        let key = match key.filter(|s| !s.is_empty()) {
            Some(key) => ApiKey::new(key)?,
            None => ApiKey::new(
                self.store
                    .get::<String>("credentials", &settings.credential_id())?
                    .ok_or_else(|| anyhow::anyhow!("An API key is required to complete setup"))?,
            )?,
        };
        let connection = self.model_connection(settings.clone(), key.clone())?;
        let mut request = settings.request(vec![Message::user("Reply with OK.")]);
        request.max_output_tokens = 1024;
        let response = connection
            .client
            .complete(
                &settings.provider_config(),
                settings.project.as_deref(),
                &request,
            )
            .await?;
        if response.message.text_content().trim().is_empty() {
            bail!("The model did not return text. Choose a compatible text model and try again");
        }
        self.redactor.add(key.expose_secret());
        self.store.put(
            "credentials",
            &settings.credential_id(),
            &key.expose_secret(),
        )?;
        self.store.put("settings", "active", &settings)?;
        // Signed replay data is protocol-specific. Preserve it on disk; only clear it
        // when building context for a different protocol (see context.rs).
        self.language_servers.shutdown().await;
        self.record_model_health("intelligent", &connection, None)?;
        *self.model.write().await = Some(Arc::new(connection));
        Ok(json!({"connected":true,"model":settings.model,"usage":response.usage}))
    }
}

pub fn provider_catalog() -> Value {
    json!(ProviderId::ALL.iter().map(|id|{
        let d=descriptor(*id);
        json!({"id":id,"base_url":d.default_base_url,"requires_base_url":d.default_base_url.is_none(),"project_header":d.project_header,"organization_header":d.organization_header,"default_model":if *id==ProviderId::OpenAi {Some(DEFAULT_MODEL)} else {None}})
    }).collect::<Vec<_>>())
}

#[cfg(test)]
mod local_command_defaults_tests {
    use super::*;
    #[test]
    fn new_installations_use_risk_checks_and_existing_ask_preferences_survive() {
        let settings = Settings::initial(PathBuf::from("/tmp/workspace"));
        assert!(settings.command_policy == ActionPolicy::Allow);
        let mut value = serde_json::to_value(settings).unwrap();
        value.as_object_mut().unwrap().remove("command_policy");
        let defaults: Settings = serde_json::from_value(value.clone()).unwrap();
        assert!(defaults.command_policy == ActionPolicy::Allow);
        value["command_policy"] = json!("ask");
        let existing: Settings = serde_json::from_value(value).unwrap();
        assert!(existing.command_policy == ActionPolicy::Ask);
    }
}
