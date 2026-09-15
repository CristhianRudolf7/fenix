//! Keyed coordination without retaining page payloads or locks after callers leave.
use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};
use tokio::sync::Mutex as AsyncMutex;

#[derive(Default)]
pub(crate) struct Entries<T> {
    entries: Mutex<HashMap<String, Weak<AsyncMutex<T>>>>,
}

impl<T: Default> Entries<T> {
    pub(crate) fn get(&self, key: &str) -> Result<Arc<AsyncMutex<T>>> {
        let mut entries = self
            .entries
            .lock()
            .ok()
            .context("Web coordination lock poisoned")?;
        entries.retain(|_, entry| entry.strong_count() > 0);
        if let Some(entry) = entries.get(key).and_then(Weak::upgrade) {
            return Ok(entry);
        }
        let entry = Arc::new(AsyncMutex::new(T::default()));
        entries.insert(key.to_owned(), Arc::downgrade(&entry));
        Ok(entry)
    }
}
