//! Bounded transfer state, independent of the diagnostic event ring.
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

#[derive(Default)]
pub(crate) struct Downloads {
    entries: VecDeque<Transfer>,
}
struct Transfer {
    id: String,
    state: String,
    received: u64,
    total: u64,
    progressed: Instant,
}
impl Transfer {
    fn value(&self) -> Value {
        json!({"id":self.id,"state":self.state,"received_bytes":self.received,"total_bytes":self.total})
    }
}
impl Downloads {
    pub fn observe(&mut self, event: &Value) {
        let method = event["method"].as_str().unwrap_or_default();
        if !matches!(
            method,
            "Browser.downloadWillBegin" | "Browser.downloadProgress"
        ) {
            return;
        }
        let params = &event["params"];
        let Some(id) = params["guid"]
            .as_str()
            .filter(|id| uuid::Uuid::parse_str(id).is_ok())
        else {
            return;
        };
        let index = self
            .entries
            .iter()
            .position(|entry| entry.id == id)
            .unwrap_or_else(|| {
                if self.entries.len() == 128 {
                    // Prefer retaining active transfers; memory remains bounded even if a site
                    // starts more transfers than we can track. Completed files remain on disk.
                    let victim = self
                        .entries
                        .iter()
                        .position(|entry| entry.state != "inProgress")
                        .unwrap_or(0);
                    self.entries.remove(victim);
                }
                self.entries.push_back(Transfer {
                    id: id.into(),
                    state: "inProgress".into(),
                    received: 0,
                    total: 0,
                    progressed: Instant::now(),
                });
                self.entries.len() - 1
            });
        let entry = &mut self.entries[index];
        if entry.state != "inProgress" {
            return;
        }
        let received = params["receivedBytes"]
            .as_f64()
            .filter(|n| n.is_finite() && *n >= 0.)
            .map(|n| n as u64)
            .unwrap_or(entry.received);
        let state = params["state"]
            .as_str()
            .filter(|s| matches!(*s, "inProgress" | "completed" | "canceled"))
            .unwrap_or("inProgress");
        if received > entry.received || state != entry.state {
            entry.progressed = Instant::now();
        }
        entry.received = entry.received.max(received);
        entry.total = params["totalBytes"]
            .as_f64()
            .filter(|n| n.is_finite() && *n >= 0.)
            .map(|n| n as u64)
            .unwrap_or(entry.total);
        entry.state = state.into();
    }
    pub fn get(&self, id: &str) -> Option<Value> {
        self.entries
            .iter()
            .find(|entry| entry.id == id)
            .map(Transfer::value)
    }
    pub fn values(&self) -> Vec<Value> {
        self.entries.iter().rev().map(Transfer::value).collect()
    }
    pub fn active_recently(&self, idle: Duration) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.state == "inProgress" && entry.progressed.elapsed() < idle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(id: &str, state: &str, bytes: u64) -> Value {
        json!({"method":"Browser.downloadProgress","params":{"guid":id,"state":state,"receivedBytes":bytes,"totalBytes":1000}})
    }
    #[test]
    fn unrelated_events_cannot_erase_completion_or_keep_a_stall_alive() {
        let mut ledger = Downloads::default();
        let id = uuid::Uuid::new_v4().to_string();
        ledger.observe(&event(&id, "inProgress", 100));
        ledger.entries[0].progressed = Instant::now() - Duration::from_secs(100);
        ledger.observe(&event(&id, "inProgress", 100));
        assert!(!ledger.active_recently(Duration::from_secs(60)));
        ledger.observe(&event(&id, "inProgress", 101));
        assert!(ledger.active_recently(Duration::from_secs(60)));
        ledger.observe(&event(&id, "completed", 1000));
        for _ in 0..200 {
            ledger.observe(&json!({"method":"Network.loadingFailed"}));
        }
        ledger.observe(&event(&id, "inProgress", 100));
        assert_eq!(ledger.get(&id).unwrap()["state"], "completed");
        assert!(!ledger.active_recently(Duration::from_secs(60)));
    }
    #[test]
    fn bounded_ledger_preserves_active_transfers_before_terminal_entries() {
        let mut ledger = Downloads::default();
        let active = uuid::Uuid::new_v4().to_string();
        ledger.observe(&event(&active, "inProgress", 50));
        for _ in 0..200 {
            ledger.observe(&event(&uuid::Uuid::new_v4().to_string(), "canceled", 0));
        }
        assert_eq!(ledger.values().len(), 128);
        assert!(ledger.get(&active).is_some());
        assert_eq!(ledger.values()[0]["state"], "canceled");
    }
}
