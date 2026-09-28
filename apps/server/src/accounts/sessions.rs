//! Durable gateway sessions. The file stays beside the host-only account registry.
use super::{Account, private_write};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub(super) const SESSION_SECONDS: u64 = 7 * 24 * 60 * 60;
const SESSION_MILLIS: u64 = SESSION_SECONDS * 1000;

pub(super) fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Session {
    // Never persist the registry's array index: account order can change on restart.
    pub account_id: String,
    pub password_hash: String,
    #[serde(default)]
    pub auth_epoch: u64,
    pub issued_at_ms: u64,
    pub expires_at_ms: u64,
}
impl Session {
    pub fn new(account: &Account, now: u64) -> Self {
        Self {
            account_id: account.id.clone(),
            password_hash: account.password_hash.clone(),
            auth_epoch: account.auth_epoch,
            issued_at_ms: now,
            expires_at_ms: now + SESSION_MILLIS,
        }
    }
    pub fn valid_at(&self, now: u64) -> bool {
        self.expires_at_ms.checked_sub(self.issued_at_ms) == Some(SESSION_MILLIS)
            && now < self.expires_at_ms
    }
}
#[derive(Serialize, Deserialize)]
struct File {
    version: u8,
    sessions: HashMap<String, Session>,
}
pub(super) struct Sessions {
    path: PathBuf,
    entries: HashMap<String, Session>,
}
impl Sessions {
    pub fn open(path: PathBuf) -> Result<Self> {
        let entries = match std::fs::read(&path) {
            Ok(bytes) => {
                let file: File = serde_json::from_slice(&bytes).context("Read gateway sessions")?;
                ensure!(file.version == 1, "Unsupported gateway session format");
                file.sessions
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => HashMap::new(),
            Err(error) => return Err(error).context("Read gateway sessions"),
        };
        Ok(Self { path, entries })
    }
    pub fn get(&self, key: &str, now: u64) -> Option<&Session> {
        self.entries
            .get(key)
            .filter(|session| session.valid_at(now))
    }
    pub fn full(&self, now: u64) -> bool {
        self.entries.values().filter(|s| s.valid_at(now)).count() >= 10_000
    }
    pub fn insert(&mut self, key: String, session: Session, now: u64) -> Result<()> {
        let mut entries = self.entries.clone();
        entries.retain(|_, session| session.valid_at(now));
        entries.insert(key, session);
        self.commit(entries)
    }
    pub fn remove(&mut self, key: &str) -> Result<()> {
        if !self.entries.contains_key(key) {
            return Ok(());
        }
        let mut entries = self.entries.clone();
        entries.remove(key);
        self.commit(entries)
    }
    fn commit(&mut self, entries: HashMap<String, Session>) -> Result<()> {
        let file = File {
            version: 1,
            sessions: entries,
        };
        // Persist before acknowledging login/logout; failed writes leave memory unchanged.
        private_write(&self.path, &serde_json::to_vec(&file)?)?;
        self.entries = file.sessions;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    fn account() -> Account {
        Account {
            id: "alice".into(),
            email: "alice@example.test".into(),
            display_name: String::new(),
            password_hash: "test-hash".into(),
            upstream: "http://127.0.0.1:9".into(),
            upstream_token: "test-only".into(),
            runtime_port: 4173,
            disabled: false,
            auth_epoch: 0,
        }
    }
    #[test]
    fn fixed_week_survives_reopening_without_sliding_expiration() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("sessions.json");
        let start = 1_800_000_000_123;
        let mut store = Sessions::open(path.clone()).unwrap();
        store
            .insert("test-token".into(), Session::new(&account(), start), start)
            .unwrap();
        let original = std::fs::read(&path).unwrap();
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        drop(store);
        let store = Sessions::open(path.clone()).unwrap();
        let deadline = start + 604_800_000;
        for at in [start, start + 86_400_000, deadline - 1] {
            assert_eq!(store.get("test-token", at).unwrap().expires_at_ms, deadline);
        }
        assert!(store.get("test-token", deadline).is_none());
        assert!(store.get("test-token", deadline + 1).is_none());
        assert_eq!(std::fs::read(path).unwrap(), original);
    }
    #[test]
    fn writes_must_succeed_before_login_or_logout_is_acknowledged() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("sessions.json");
        let mut store = Sessions::open(path.clone()).unwrap();
        std::fs::create_dir(&path).unwrap();
        assert!(
            store
                .insert("test-token".into(), Session::new(&account(), 1000), 1000)
                .is_err()
        );
        assert!(store.get("test-token", 1000).is_none());
        std::fs::remove_dir(&path).unwrap();
        store
            .insert("test-token".into(), Session::new(&account(), 1000), 1000)
            .unwrap();
        let saved = directory.path().join("saved.json");
        std::fs::rename(&path, &saved).unwrap();
        std::fs::create_dir(&path).unwrap();
        assert!(store.remove("test-token").is_err());
        assert!(store.get("test-token", 1000).is_some());
        std::fs::remove_dir(&path).unwrap();
        std::fs::rename(saved, &path).unwrap();
        store.remove("test-token").unwrap();
        assert!(
            Sessions::open(path)
                .unwrap()
                .get("test-token", 1000)
                .is_none()
        );
    }
    #[test]
    fn corrupt_session_files_are_not_silently_discarded() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("sessions.json");
        std::fs::write(&path, b"broken json").unwrap();
        assert!(Sessions::open(path).is_err());
    }
}
