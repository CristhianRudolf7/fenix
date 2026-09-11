use crate::{ProviderConfig, catalog::descriptor};
use fenix_core::ProviderId;
use fenix_credentials::{Credential, CredentialError, CredentialStore};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Configuration contains references to secrets, never their values.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvidersConfig {
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
    #[serde(default)]
    pub credentials: Vec<CredentialConfig>,
    #[serde(default)]
    pub projects: Vec<ProjectConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialConfig {
    pub provider: ProviderId,
    pub api_key_env: String,
    #[serde(default)]
    pub provider_project: Option<String>,
    #[serde(default)]
    pub organization: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub id: String,
    #[serde(default)]
    pub credentials: Vec<CredentialConfig>,
    /// Optional project-specific endpoints (for example an Azure resource).
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
}

impl ProvidersConfig {
    pub fn from_toml(input: &str) -> Result<Self, ConfigurationError> {
        // Do not return a TOML parser error with a snippet that might contain an
        // accidentally pasted credential. Unknown fields (including api_key) fail.
        let config: Self = toml::from_str(input).map_err(|_| ConfigurationError::InvalidToml)?;
        config.credential_store()?;
        Ok(config)
    }

    pub fn credential_store(&self) -> Result<CredentialStore, ConfigurationError> {
        validate_providers(&self.providers)?;
        let mut store = CredentialStore::default();
        let mut seen = HashSet::new();
        for credential in &self.credentials {
            if !seen.insert(credential.provider) {
                return Err(ConfigurationError::DuplicateCredential);
            }
            store.set_default(credential.provider, credential.to_credential()?);
        }
        let mut projects = HashSet::new();
        for project in &self.projects {
            if !projects.insert(&project.id) {
                return Err(ConfigurationError::DuplicateProject);
            }
            if project.id.is_empty()
                || project.id.len() > 128
                || !project
                    .id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
            {
                return Err(CredentialError::InvalidProject.into());
            }
            validate_providers(&project.providers)?;
            let mut seen = HashSet::new();
            for credential in &project.credentials {
                if !seen.insert(credential.provider) {
                    return Err(ConfigurationError::DuplicateCredential);
                }
                store.set_project(
                    &project.id,
                    credential.provider,
                    credential.to_credential()?,
                )?;
            }
        }
        Ok(store)
    }

    /// Resolve endpoints independently from credentials, without losing scope.
    pub fn provider(
        &self,
        provider: ProviderId,
        project: Option<&str>,
    ) -> Result<ProviderConfig, ConfigurationError> {
        if let Some(project) = project {
            let project = self
                .projects
                .iter()
                .find(|entry| entry.id == project)
                .ok_or(ConfigurationError::UnknownProject)?;
            if let Some(config) = project
                .providers
                .iter()
                .find(|entry| entry.provider == provider)
            {
                return Ok(config.clone());
            }
        }
        Ok(self
            .providers
            .iter()
            .find(|entry| entry.provider == provider)
            .cloned()
            .unwrap_or_else(|| ProviderConfig::new(provider)))
    }
}

impl CredentialConfig {
    fn to_credential(&self) -> Result<Credential, ConfigurationError> {
        let provider = descriptor(self.provider);
        if self.provider_project.is_some() && provider.project_header.is_none()
            || self.organization.is_some() && provider.organization_header.is_none()
        {
            return Err(ConfigurationError::UnsupportedProviderScope);
        }
        let mut credential = Credential::environment(&self.api_key_env)?;
        credential
            .provider_project
            .clone_from(&self.provider_project);
        credential.organization.clone_from(&self.organization);
        Ok(credential)
    }
}

fn validate_providers(providers: &[ProviderConfig]) -> Result<(), ConfigurationError> {
    let mut seen = HashSet::new();
    for provider in providers {
        if !seen.insert(provider.provider) {
            return Err(ConfigurationError::DuplicateProvider);
        }
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error(
        "invalid providers TOML; check field names and types (inline secrets are not supported)"
    )]
    InvalidToml,
    #[error("duplicate provider endpoint configuration")]
    DuplicateProvider,
    #[error("duplicate credentials for the same provider and scope")]
    DuplicateCredential,
    #[error("duplicate Fénix project ID")]
    DuplicateProject,
    #[error("unknown Fénix project")]
    UnknownProject,
    #[error("provider does not support the configured project or organization header")]
    UnsupportedProviderScope,
    #[error(transparent)]
    Credential(#[from] CredentialError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_example_is_valid_without_credentials_in_environment() {
        ProvidersConfig::from_toml(include_str!("../../../config/providers.example.toml")).unwrap();
    }

    #[test]
    fn config_contains_references_and_keeps_project_endpoints() {
        let config = ProvidersConfig::from_toml(
            r#"
            [[providers]]
            provider = "azure"
            base_url = "https://default.example/openai/v1"
            [[credentials]]
            provider = "azure"
            api_key_env = "AZURE_API_KEY"
            [[projects]]
            id = "work"
            [[projects.providers]]
            provider = "azure"
            base_url = "https://work.example/openai/v1"
            [[projects.credentials]]
            provider = "azure"
            api_key_env = "WORK_AZURE_KEY"
        "#,
        )
        .unwrap();
        assert_eq!(
            config
                .provider(ProviderId::Azure, Some("work"))
                .unwrap()
                .base_url
                .as_deref(),
            Some("https://work.example/openai/v1")
        );
        assert!(matches!(
            config.provider(ProviderId::Azure, Some("missing")),
            Err(ConfigurationError::UnknownProject)
        ));
        // Loading the file must not require that referenced secrets already exist.
        assert!(config.credential_store().is_ok());
    }

    #[test]
    fn rejects_ambiguous_configuration() {
        assert!(matches!(
            ProvidersConfig::from_toml(
                r#"
            [[credentials]]
            provider="openai"
            api_key_env="KEY_A"
            [[credentials]]
            provider="openai"
            api_key_env="KEY_B"
        "#
            ),
            Err(ConfigurationError::DuplicateCredential)
        ));
        assert!(matches!(
            ProvidersConfig::from_toml(
                r#"
            [[projects]]
            id="work"
            [[projects]]
            id="work"
        "#
            ),
            Err(ConfigurationError::DuplicateProject)
        ));
    }

    #[test]
    fn invalid_toml_does_not_leak_pasted_keys() {
        let error = ProvidersConfig::from_toml(
            r#"
            [[credentials]]
            provider="openai"
            api_key="accidentally-pasted-secret"
        "#,
        )
        .unwrap_err();
        assert!(!format!("{error:?}: {error}").contains("accidentally-pasted-secret"));
    }

    #[test]
    fn rejects_misleading_native_scope_headers() {
        assert!(matches!(
            ProvidersConfig::from_toml(
                r#"
            [[credentials]]
            provider="deepseek"
            api_key_env="KEY"
            provider_project="not-a-real-provider-scope"
        "#
            ),
            Err(ConfigurationError::UnsupportedProviderScope)
        ));
    }
}
