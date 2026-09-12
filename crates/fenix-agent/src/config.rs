use crate::Runtime;
use anyhow::{Result, bail};
use fenix_core::{ChatRequest, Message, ProviderId, ReasoningEffort};
use fenix_credentials::{ApiKey, Credential, CredentialStore};
use fenix_providers::{ProviderClient, ProviderConfig, catalog::descriptor};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
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
    pub workspace: PathBuf,
    #[serde(default = "default_approval")]
    pub command_policy: ActionPolicy,
    #[serde(default = "default_approval")]
    pub file_write_policy: ActionPolicy,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionPolicy {
    Ask,
    Allow,
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
            workspace,
            command_policy: ActionPolicy::Ask,
            file_write_policy: ActionPolicy::Ask,
        }
    }
    pub fn request(&self, messages: Vec<Message>) -> ChatRequest {
        let mut request = ChatRequest::new(&self.model, messages);
        request.max_output_tokens = 4096;
        if self.provider == ProviderId::OpenAi {
            request.reasoning_effort = self.reasoning_effort;
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
    pub settings: Settings,
    pub client: ProviderClient,
}

impl Runtime {
    pub fn settings(&self) -> Result<Option<Settings>> {
        self.store.get("settings", "active")
    }

    pub fn model_connection(&self, settings: Settings, key: ApiKey) -> Result<ModelConnection> {
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
            settings,
            client: ProviderClient::new(store)?,
        })
    }

    pub async fn configure(&self, mut settings: Settings, key: Option<String>) -> Result<Value> {
        let _guard = self.configure_lock.lock().await;
        if self.active.lock().await.is_some()
            || self.store.tasks(200)?.iter().any(|t| t.status == "queued")
        {
            bail!("Wait for queued or active tasks before changing model configuration");
        }
        settings.model = settings.model.trim().to_owned();
        if settings.model.is_empty() || settings.model.len() > 512 {
            bail!("Enter a model identifier");
        }
        settings.workspace = std::fs::canonicalize(&settings.workspace)?;
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
