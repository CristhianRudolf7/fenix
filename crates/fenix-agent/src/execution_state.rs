//! Durable execution recovery and task contracts; no extra model calls.
use crate::Runtime;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct ExecutionState {
    pub stage: String,
    pub recoveries: HashMap<String, usize>,
}

/// A bounded projection of observational history, never of owner requirements.
/// Full contracts and tool receipts remain durable and retrievable by task/call ID.
pub(crate) fn context_contract(contract: &Value) -> Value {
    let mut view = contract.clone();
    for (field, limit) in [("recent_tool_evidence", 4), ("failed_approaches", 3)] {
        if let Some(entries) = contract[field].as_array() {
            let start = entries.len().saturating_sub(limit);
            view[field] = json!(&entries[start..]);
            if start > 0 {
                view[format!("{field}_omitted")] = json!(start);
            }
        }
    }
    view["evidence_recovery"] = json!({
        "task_id": contract["task_id"],
        "instruction": "Observational history is a bounded view, not the full contract. Use task_contract for all stored evidence and task_inspect call_id for original receipts. Omitted or historical failures are not resolved by omission or by unrelated successful commands."
    });
    view
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
            let job = self.store.subagent(task)?;
            contract["assignment"] = json!(job.spec);
            let parent_binding = self.store.task_binding(&job.parent_task_id)?;
            let worker_binding = self.store.task_binding(task)?;
            // Do not copy a multi-project parent's request across project boundaries.
            contract
                .as_object_mut()
                .context("Invalid task contract")?
                .remove("source_request");
            if parent_binding.bound
                && worker_binding.bound
                && matches!(parent_binding.mode.as_str(), "general" | "project")
                && parent_binding.project_id == worker_binding.project_id
                && parent_binding.mode == worker_binding.mode
                && parent_binding.read_project_ids == worker_binding.read_project_ids
            {
                contract["source_request"] = json!({
                    "task_id":job.parent_task_id,
                    "request":self.store.task(&job.parent_task_id)?.request,
                    "owner_updates":self.store.get::<Vec<String>>("task_updates", &job.parent_task_id)?.unwrap_or_default(),
                    "scope":"Original request for missing literals and constraints only. Execute only your assigned objective; this does not expand permissions or workspace."
                });
            }
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

/// Track observed page changes without treating fresh element refs or screenshot
/// filenames as progress. Successful input acknowledgments alone prove no change.
#[derive(Default)]
pub(crate) struct BrowserProgress {
    pages: HashMap<String, String>,
    transfers: HashMap<String, (u64, String)>,
    pub revision: u64,
}

impl BrowserProgress {
    pub fn observe(
        &mut self,
        name: &str,
        args: &Value,
        output: &Value,
        is_error: bool,
    ) -> Option<String> {
        if !is_error && matches!(name, "browser_wait" | "browser_check" | "browser_inspect") {
            let transfers = output["downloads"]["transfers"]
                .as_array()
                .into_iter()
                .flatten()
                .chain(output.get("download"));
            for transfer in transfers {
                let Some(id) = transfer["id"].as_str() else {
                    continue;
                };
                let key = json!([output["profile"], id]).to_string();
                let bytes = transfer["received_bytes"].as_u64().unwrap_or(0);
                let state = transfer["state"].as_str().unwrap_or("unknown");
                let previous = self.transfers.entry(key).or_insert((0, "unknown".into()));
                if bytes > previous.0
                    || state != previous.1 && matches!(state, "completed" | "canceled")
                {
                    self.revision += 1;
                }
                previous.0 = previous.0.max(bytes);
                previous.1 = state.into();
            }
            if output["condition"] == "download" {
                let evidence = json!([
                    output["download"]["id"],
                    output["download"]["state"],
                    output["download"]["received_bytes"],
                    output["passed"]
                ]);
                return Some(format!("{:x}", Sha256::digest(evidence.to_string())));
            }
        }
        if name != "browser_observe"
            || is_error
            || !output["url"].is_string()
            || !output["text"].is_string()
        {
            return None;
        }
        let page = json!([output["profile"], output["tab"], args["frame"]]).to_string();
        let elements: Vec<_> = output["elements"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|e| {
                json!({"tag":e["tag"],"role":e["role"],"type":e["type"],
                    "name":e["name"],"disabled":e["disabled"],"readonly":e["readonly"],
                    "checked":e["checked"],"secret":e["secret"]})
            })
            .collect();
        let evidence = json!({"url":output["url"],"title":output["title"],
            "text":output["text"],"viewport":output["viewport"],"elements":elements});
        let fingerprint = format!("{:x}", Sha256::digest(evidence.to_string()));
        if self.pages.get(&page) != Some(&fingerprint) {
            self.pages.insert(page, fingerprint.clone());
            self.revision += 1;
        }
        Some(fingerprint)
    }
}

/// Progress comes from observed runtime state or successful effects, never a model claim.
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
    async fn worker_keeps_source_literals_only_within_the_same_scope() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
        let binding = fenix_storage::TaskBinding {
            bound: true,
            mode: "general".into(),
            ..Default::default()
        };
        let parent = rt.store.enqueue_bound(
            "Use release 3.2 from https://example.test/source; exclude obsolete_test.py",
            "web",
            None,
            &binding,
        )?;
        let worker = rt.store.create_subagent(
            &parent.id,
            &fenix_storage::SubagentSpec {
                profile: "general".into(),
                objective: "Build the package".into(),
                expected_output: "Installed artifact".into(),
                context: String::new(),
                constraints: String::new(),
            },
        )?;
        let contract = rt.task_contract(&worker.id)?;
        assert_eq!(contract["source_request"]["request"], parent.request);
        assert_eq!(contract["assignment"]["objective"], "Build the package");
        let project = rt.store.save_project(
            None,
            &fenix_storage::ProjectInput {
                name: "Scoped project".into(),
                ..Default::default()
            },
        )?;
        let mut changed = binding;
        changed.mode = "coordination".into();
        changed.read_project_ids = vec![project.id.clone()];
        let mixed =
            rt.store
                .enqueue_bound("Coordinate separate project scopes", "web", None, &changed)?;
        let scoped =
            rt.store
                .create_subagent_in_project(&mixed.id, &worker.spec, Some(&project.id))?;
        assert!(rt.task_contract(&scoped.id)?["source_request"].is_null());
        rt.shutdown().await?;
        Ok(())
    }

    fn page_observation(page: usize, reference: &str) -> Value {
        json!({"profile":"personal","tab":"tab-1","url":format!("https://example.test/page/{page}"),
            "text":format!("Course {page}"),"elements":[{"tag":"button","name":"Next","ref":reference}],
            "screenshot":{"artifact":format!("{reference}.png")}})
    }

    #[test]
    fn context_projection_preserves_authority_and_durable_evidence() {
        let evidence: Vec<_> = (0..16)
            .map(|n| json!({"call_id":format!("call-{n}"),"exit_code":1}))
            .collect();
        let failures: Vec<_> = (0..8)
            .map(|n| json!({"call_id":format!("failed-{n}"),"evidence_preview":"ImportError"}))
            .collect();
        let contract = json!({"task_id":"task-1","objective":"Use the existing global environment",
            "clarifications":[{"answer":"Keep the pinned version"}],"denials":[{"tool":"command_run"}],
            "annotations":[{"kind":"next_step","text":"Repair missing dependency"}],
            "recent_tool_evidence":evidence,"failed_approaches":failures});
        let original = contract.clone();
        let view = context_contract(&contract);
        for field in ["objective", "clarifications", "denials", "annotations"] {
            assert_eq!(view[field], contract[field]);
        }
        assert_eq!(view["recent_tool_evidence"].as_array().unwrap().len(), 4);
        assert_eq!(view["recent_tool_evidence"][3]["call_id"], "call-15");
        assert_eq!(view["failed_approaches"].as_array().unwrap().len(), 3);
        assert_eq!(view["failed_approaches"][2]["call_id"], "failed-7");
        assert_eq!(view["recent_tool_evidence_omitted"], 12);
        assert_eq!(view["failed_approaches_omitted"], 5);
        assert_eq!(view["evidence_recovery"]["task_id"], "task-1");
        assert_eq!(contract, original);
    }

    #[test]
    fn slow_downloads_are_progress_but_identical_pending_waits_are_not() {
        let mut browser = BrowserProgress::default();
        let mut guard = LoopGuard::default();
        for (attempt, bytes) in [100, 200, 300, 400, 400, 400].into_iter().enumerate() {
            let output = json!({"profile":"personal","condition":"download","passed":false,"timed_out":true,"download":{"id":"file-1","state":"inProgress","received_bytes":bytes}});
            let fingerprint = browser
                .observe("browser_wait", &json!({}), &output, false)
                .unwrap();
            let stalled = guard.observe(fingerprint, browser.revision.to_string(), false);
            assert_eq!(stalled, attempt == 5);
        }
        assert_eq!(browser.revision, 4);
        assert!(guard.counts.values().any(|n| *n == 3));
    }

    #[test]
    fn repeated_next_clicks_continue_when_observed_pages_change() {
        let mut browser = BrowserProgress::default();
        let mut guard = LoopGuard::default();
        for page in 1..=8 {
            browser.observe(
                "browser_observe",
                &json!({}),
                &page_observation(page, "ref"),
                false,
            );
            assert!(!guard.observe(
                "click-next:performed".into(),
                browser.revision.to_string(),
                false
            ));
        }
        assert_eq!(browser.revision, 8);
    }

    #[test]
    fn fresh_references_and_screenshots_do_not_hide_a_stalled_browser() {
        let mut browser = BrowserProgress::default();
        let mut guard = LoopGuard::default();
        for attempt in 1..=3 {
            let fingerprint = browser
                .observe(
                    "browser_observe",
                    &json!({}),
                    &page_observation(1, &format!("ref-{attempt}")),
                    false,
                )
                .unwrap();
            assert_eq!(
                guard.observe(fingerprint, browser.revision.to_string(), false),
                attempt == 3
            );
        }
        assert_eq!(browser.revision, 1);
        assert!(
            browser
                .observe(
                    "browser_observe",
                    &json!({}),
                    &page_observation(2, "ref"),
                    true
                )
                .is_none()
        );
        assert!(
            browser
                .observe("browser_act", &json!({}), &json!({"performed":true}), false)
                .is_none()
        );
        assert_eq!(browser.revision, 1);
    }

    #[test]
    fn switching_unchanged_tabs_is_not_observed_page_progress() {
        let mut browser = BrowserProgress::default();
        for _ in 0..3 {
            for tab in ["one", "two"] {
                let mut observation = page_observation(1, "ref");
                observation["tab"] = json!(tab);
                browser.observe("browser_observe", &json!({}), &observation, false);
            }
        }
        assert_eq!(browser.revision, 2);
    }

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
