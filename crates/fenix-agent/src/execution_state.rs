//! Durable execution recovery and task contracts; no extra model calls.
use crate::Runtime;
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct ExecutionState {
    pub stage: String,
    pub recoveries: HashMap<String, usize>,
}

impl Runtime {
    pub(crate) fn execution_stage(&self, task: &str, stage: &str) -> Result<()> {
        let mut state = self
            .store
            .get::<ExecutionState>("task_execution_state", task)?
            .unwrap_or_default();
        state.stage = stage.into();
        self.store.put("task_execution_state", task, &state)?;
        Ok(())
    }
    pub(crate) fn recover_execution(
        &self,
        task: &str,
        reason: &str,
        evidence: Value,
    ) -> Result<()> {
        let mut state = self
            .store
            .get::<ExecutionState>("task_execution_state", task)?
            .unwrap_or_default();
        let total: usize = state.recoveries.values().sum();
        let attempts = state.recoveries.entry(reason.into()).or_default();
        if *attempts >= 3 || total >= 8 {
            bail!("Recovery budget exhausted for {reason}; inspect task events before continuing");
        }
        *attempts += 1;
        let attempt = *attempts;
        state.stage = "recovering".into();
        self.store.put("task_execution_state", task, &state)?;
        let data = json!({"reason":reason,"attempt":attempt,"evidence":evidence,"session_id":self.store.task(task)?.session_id});
        self.store.event(task, "task_recovery", &data)?;
        self.store
            .publish_event(Some(task), "task.recovering", &data)?;
        self.bus_wake.notify_waiters();
        Ok(())
    }
    pub(crate) fn task_contract(&self, task: &str) -> Result<Value> {
        let current = self.store.task(task)?;
        let mut contract = self.store.get::<Value>("task_contracts", task)?.unwrap_or_else(|| json!({
            "task_id":task,"session_id":current.session_id,"objective":current.request,
            "requirements_source":"Original user request; do not invent additional requirements.",
            "clarifications":[],"assumptions":[],"completion_evidence":[]
        }));
        contract["binding"] = json!(self.store.task_binding(task)?);
        contract["owner_updates"] = json!(
            self.store
                .get::<Vec<String>>("task_updates", task)?
                .unwrap_or_default()
        );
        contract["grants"] = json!(
            self.store
                .get::<Vec<Value>>("task_grants", task)?
                .unwrap_or_default()
        );
        contract["denials"] = json!(
            self.store
                .get::<Vec<Value>>("task_denials", task)?
                .unwrap_or_default()
        );
        if let Some(settings) = self.settings()? {
            contract["runtime_policy"] = json!({"commands":settings.command_policy,"file_writes":settings.file_write_policy});
        }
        if current.origin == "subagent" {
            contract["assignment"] = json!(self.store.subagent(task)?.spec);
        }
        self.store.put("task_contracts", task, &contract)?;
        Ok(contract)
    }
    pub(crate) fn contract_answer(&self, task: &str, question: &str, answer: &str) -> Result<()> {
        let mut contract = self.task_contract(task)?;
        let updates = contract["owner_updates"].clone();
        let entries = contract["clarifications"]
            .as_array_mut()
            .ok_or_else(|| anyhow::anyhow!("Invalid persisted contract clarifications"))?;
        entries.push(json!({"question":question,"answer":answer,"source":"explicit_user_reply","owner_updates":updates}));
        self.store.put("task_contracts", task, &contract)?;
        Ok(())
    }
    pub(crate) fn answered_question(&self, task: &str, question: &str) -> Result<Option<String>> {
        let contract = self.task_contract(task)?;
        Ok(contract["clarifications"]
            .as_array()
            .into_iter()
            .flatten()
            .rev()
            .find(|e| {
                e["owner_updates"] == contract["owner_updates"]
                    && normalized(e["question"].as_str().unwrap_or("")) == normalized(question)
            })
            .and_then(|e| e["answer"].as_str())
            .map(str::to_owned))
    }
}
fn normalized(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Progress comes from observed process state or successful effects, never a model claim.
#[derive(Default)]
pub(crate) struct LoopGuard {
    counts: HashMap<String, usize>,
    progress: String,
}
impl LoopGuard {
    pub fn observe(
        &mut self,
        fingerprint: String,
        progress: String,
        effect_completed: bool,
    ) -> bool {
        if self.progress != progress || effect_completed {
            self.counts.clear();
            self.progress = progress;
        }
        let count = self.counts.entry(fingerprint).or_default();
        *count += 1;
        *count >= 3
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn explicit_answers_survive_reopening_without_becoming_grants() {
        let dir = tempfile::tempdir().unwrap();
        let data = dir.path().join("data");
        let skills = dir.path().join("skills");
        let rt = Runtime::open(&data, Some(&skills)).await.unwrap();
        let task = rt
            .store
            .enqueue("Repair login without changing the API", "web", None)
            .unwrap();
        rt.contract_answer(&task.id, "Which branch?", "main")
            .unwrap();
        rt.shutdown().await.unwrap();
        drop(rt);
        let rt = Runtime::open(&data, Some(&skills)).await.unwrap();
        assert_eq!(
            rt.answered_question(&task.id, " Which   branch? ")
                .unwrap()
                .as_deref(),
            Some("main")
        );
        assert!(
            rt.answered_question(&task.id, "May I deploy?")
                .unwrap()
                .is_none()
        );
        let contract = rt.task_contract(&task.id).unwrap();
        assert_eq!(contract["objective"], task.request);
        assert_eq!(contract["grants"], json!([]));
        rt.store
            .put(
                "task_updates",
                &task.id,
                &vec!["Use the release branch now"],
            )
            .unwrap();
        assert!(
            rt.answered_question(&task.id, "Which branch?")
                .unwrap()
                .is_none()
        );
        rt.shutdown().await.unwrap();
    }
    #[test]
    fn progressing_installation_is_not_a_loop_but_stagnation_is() {
        let mut guard = LoopGuard::default();
        for cursor in [7107, 8000, 9707] {
            assert!(!guard.observe("busy".into(), format!("install:{cursor}"), false));
        }
        assert!(!guard.observe("busy".into(), "install:9707".into(), false));
        assert!(guard.observe("busy".into(), "install:9707".into(), false));
        assert!(!guard.observe("busy".into(), "new-process:0".into(), false));
    }
}
