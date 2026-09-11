//! Credentials kept separate from provider protocols and conversation context.

use fenix_core::ProviderId;
use std::{collections::HashMap, fmt};
use thiserror::Error;
use zeroize::Zeroizing;

/// An API key that is redacted in Debug output and zeroized when dropped.
/// Intentionally does not implement Serialize or Display.
#[derive(Clone)]
pub struct ApiKey(Zeroizing<String>);

impl ApiKey {
    pub fn new(value: impl Into<String>) -> Result<Self, CredentialError> {
        let value = Zeroizing::new(value.into());
        if value.is_empty() || value.len() > 65_536 || !value.bytes().all(|b| b.is_ascii_graphic())
        {
            return Err(CredentialError::InvalidKey);
        }
        Ok(Self(value))
    }

    pub fn expose_secret(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey([REDACTED])")
    }
}

#[derive(Clone, Debug)]
pub enum CredentialSource {
    Inline(ApiKey),
    /// Resolve on each call so environment credentials need not be copied into config.
    Environment(String),
}

#[derive(Clone, Debug)]
pub struct Credential {
    pub source: CredentialSource,
    /// Native provider project/workspace ID, distinct from a Fénix project scope.
    pub provider_project: Option<String>,
    pub organization: Option<String>,
}

impl Credential {
    pub fn inline(key: ApiKey) -> Self {
        Self {
            source: CredentialSource::Inline(key),
            provider_project: None,
            organization: None,
        }
    }

    pub fn environment(name: impl Into<String>) -> Result<Self, CredentialError> {
        let name = name.into();
        validate_env_name(&name)?;
        Ok(Self {
            source: CredentialSource::Environment(name),
            provider_project: None,
            organization: None,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ResolvedCredential {
    pub key: ApiKey,
    pub provider_project: Option<String>,
    pub organization: Option<String>,
}

/// Exact project lookup: a project-scoped call never borrows a default key.
#[derive(Clone, Debug, Default)]
pub struct CredentialStore {
    defaults: HashMap<ProviderId, Credential>,
    projects: HashMap<String, HashMap<ProviderId, Credential>>,
}

impl CredentialStore {
    pub fn set_default(&mut self, provider: ProviderId, credential: Credential) {
        self.defaults.insert(provider, credential);
    }

    pub fn set_project(
        &mut self,
        project: impl Into<String>,
        provider: ProviderId,
        credential: Credential,
    ) -> Result<(), CredentialError> {
        let project = project.into();
        validate_project(&project)?;
        self.projects
            .entry(project)
            .or_default()
            .insert(provider, credential);
        Ok(())
    }

    /// Returns None only when no default is configured and no project was requested.
    /// This permits an explicitly local provider to run without credentials.
    pub fn resolve_optional(
        &self,
        provider: ProviderId,
        project: Option<&str>,
    ) -> Result<Option<ResolvedCredential>, CredentialError> {
        let credential = match project {
            Some(project) => {
                validate_project(project)?;
                Some(
                    self.projects
                        .get(project)
                        .and_then(|keys| keys.get(&provider))
                        .ok_or(CredentialError::MissingProjectCredential { provider })?,
                )
            }
            None => self.defaults.get(&provider),
        };
        credential
            .map(|entry| {
                let key = match &entry.source {
                    CredentialSource::Inline(key) => key.clone(),
                    CredentialSource::Environment(name) => {
                        validate_env_name(name)?;
                        let value =
                            std::env::var(name).map_err(|_| CredentialError::MissingEnvironment)?;
                        ApiKey::new(value)?
                    }
                };
                Ok(ResolvedCredential {
                    key,
                    provider_project: entry.provider_project.clone(),
                    organization: entry.organization.clone(),
                })
            })
            .transpose()
    }

    pub fn resolve(
        &self,
        provider: ProviderId,
        project: Option<&str>,
    ) -> Result<ResolvedCredential, CredentialError> {
        self.resolve_optional(provider, project)?
            .ok_or(CredentialError::MissingCredential { provider })
    }
}

fn validate_env_name(name: &str) -> Result<(), CredentialError> {
    let mut chars = name.chars();
    if !chars
        .next()
        .is_some_and(|c| c == '_' || c.is_ascii_alphabetic())
        || !chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
    {
        return Err(CredentialError::InvalidEnvironmentName);
    }
    Ok(())
}

fn validate_project(project: &str) -> Result<(), CredentialError> {
    if project.is_empty()
        || project.len() > 128
        || !project
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
    {
        return Err(CredentialError::InvalidProject);
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum CredentialError {
    #[error("API key must be a nonempty ASCII token without whitespace")]
    InvalidKey,
    #[error("invalid credential environment variable name")]
    InvalidEnvironmentName,
    #[error("configured credential environment variable is missing or invalid")]
    MissingEnvironment,
    #[error("project ID must contain 1–128 ASCII letters, digits, dots, underscores, or hyphens")]
    InvalidProject,
    #[error("no default API key configured for {provider}")]
    MissingCredential { provider: ProviderId },
    #[error("no API key configured for this project and {provider}; default fallback is disabled")]
    MissingProjectCredential { provider: ProviderId },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inline(value: &str) -> Credential {
        Credential::inline(ApiKey::new(value).unwrap())
    }

    #[test]
    fn secrets_are_redacted_in_nested_debug() {
        let mut store = CredentialStore::default();
        store.set_default(ProviderId::OpenAi, inline("fake-sensitive-value"));
        store
            .set_project(
                "client",
                ProviderId::OpenAi,
                inline("other-sensitive-value"),
            )
            .unwrap();
        let output = format!(
            "{store:?} {:?}",
            store.resolve(ProviderId::OpenAi, None).unwrap()
        );
        assert!(!output.contains("fake-sensitive-value"));
        assert!(!output.contains("other-sensitive-value"));
        assert!(output.contains("[REDACTED]"));
    }

    #[test]
    fn projects_are_isolated_and_do_not_fall_back() {
        let mut store = CredentialStore::default();
        store.set_default(ProviderId::OpenAi, inline("default-key"));
        store
            .set_project("alpha", ProviderId::OpenAi, inline("alpha-key"))
            .unwrap();
        store
            .set_project("beta", ProviderId::OpenAi, inline("beta-key"))
            .unwrap();
        assert_eq!(
            store
                .resolve(ProviderId::OpenAi, Some("alpha"))
                .unwrap()
                .key
                .expose_secret(),
            "alpha-key"
        );
        assert_eq!(
            store
                .resolve(ProviderId::OpenAi, Some("beta"))
                .unwrap()
                .key
                .expose_secret(),
            "beta-key"
        );
        assert!(matches!(
            store.resolve(ProviderId::OpenAi, Some("missing")),
            Err(CredentialError::MissingProjectCredential { .. })
        ));
        assert!(matches!(
            store.resolve(ProviderId::Gemini, Some("alpha")),
            Err(CredentialError::MissingProjectCredential { .. })
        ));
    }

    #[test]
    fn a_broken_project_environment_does_not_fall_back() {
        let mut store = CredentialStore::default();
        store.set_default(ProviderId::OpenAi, inline("default-key"));
        store
            .set_project(
                "alpha",
                ProviderId::OpenAi,
                Credential::environment("FENIX_TEST_ABSENT_KEY_79EC45A1").unwrap(),
            )
            .unwrap();
        assert!(matches!(
            store.resolve(ProviderId::OpenAi, Some("alpha")),
            Err(CredentialError::MissingEnvironment)
        ));
    }

    #[test]
    fn header_injection_and_invalid_scopes_are_rejected() {
        for key in ["", " ", "secret\r\nx-header: bad", "secret value", "é"] {
            assert!(ApiKey::new(key).is_err());
        }
        for name in ["", "123KEY", "KEY=secret", "KEY\n"] {
            assert!(Credential::environment(name).is_err());
        }
        let mut store = CredentialStore::default();
        for project in ["", " alpha", "../alpha/key", "α"] {
            assert!(
                store
                    .set_project(project, ProviderId::OpenAi, inline("key"))
                    .is_err()
            );
        }
    }
}
