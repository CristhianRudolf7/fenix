//! Read current ownership on every authorization decision, including quiet SSE.
use super::{PasswordHash, Registry};
use anyhow::{Result, ensure};
use std::{collections::HashSet, io::Read, os::unix::fs::MetadataExt, path::PathBuf};

pub(super) enum RegistrySource {
    #[cfg(test)]
    Static(Registry),
    File(PathBuf),
}
impl RegistrySource {
    pub fn load(&self) -> Result<Registry> {
        match self {
            #[cfg(test)]
            Self::Static(registry) => Ok(registry.clone()),
            Self::File(path) => {
                let file = fenix_execution::fs::open(path)?;
                let metadata = file.metadata()?;
                ensure!(
                    metadata.is_file()
                        && metadata.mode() & 0o077 == 0
                        && metadata.len() <= 4 * 1024 * 1024,
                    "Registry must be a private regular file no larger than 4 MiB"
                );
                let mut bytes = vec![];
                file.take(4 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
                ensure!(
                    bytes.len() <= 4 * 1024 * 1024,
                    "Registry grew beyond its limit"
                );
                let registry: Registry = serde_json::from_slice(&bytes)?;
                validate(&registry)?;
                Ok(registry)
            }
        }
    }
}

pub(super) fn validate(registry: &Registry) -> Result<()> {
    ensure!(registry.accounts.len() <= 10000, "Account limit exceeded");
    let (mut ids, mut emails, mut ports, mut credentials) = (
        HashSet::new(),
        HashSet::new(),
        HashSet::new(),
        HashSet::new(),
    );
    for account in &registry.accounts {
        ensure!(
            uuid::Uuid::parse_str(&account.id)?.to_string() == account.id,
            "Invalid account identity"
        );
        ensure!(
            account.email == account.email.trim().to_lowercase()
                && account.email.contains('@')
                && account.email.len() <= 254,
            "Invalid account email"
        );
        let url = reqwest::Url::parse(&account.upstream)?;
        ensure!(
            url.scheme() == "http"
                && url.host_str() == Some("127.0.0.1")
                && url.port().is_some_and(|port| port > 0)
                && url.path() == "/"
                && url.query().is_none()
                && url.fragment().is_none()
                && url.username().is_empty()
                && url.password().is_none()
                && account.runtime_port > 0,
            "Invalid loopback upstream"
        );
        ensure!(
            (32..=256).contains(&account.upstream_token.len())
                && account
                    .upstream_token
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-'),
            "Invalid upstream credential"
        );
        ensure!(
            PasswordHash::new(&account.password_hash).is_ok(),
            "Invalid password hash"
        );
        ensure!(
            ids.insert(&account.id)
                && emails.insert(&account.email)
                && ports.insert(url.port())
                && credentials.insert(&account.upstream_token),
            "Duplicate account identity, email, upstream port or credential"
        );
    }
    Ok(())
}
