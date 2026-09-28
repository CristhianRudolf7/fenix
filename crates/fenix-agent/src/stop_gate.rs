//! Ground non-success outcomes in scoped observations, without another model.
use crate::Runtime;
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

#[derive(Default, Serialize, Deserialize)]
struct StopState {
    observations: Vec<Observation>,
    rejected_stops: usize,
    progress_at_rejection: usize,
    progress: usize,
    repair_epoch: usize,
    repairs_seen: HashSet<String>,
}
#[derive(Clone, Serialize, Deserialize)]
struct Observation {
    call_id: String,
    probe: String,
    attempt: String,
    epoch: usize,
    #[serde(default)]
    failure_key: String,
    category: String,
    diagnostic: String,
    resolved: bool,
}
fn digest(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
fn category(output: &Value, error: bool) -> Option<&'static str> {
    if output["denied"] == true || output["approved"] == false {
        return Some("permission_denied");
    }
    if !error
        && !output["exit_code"].as_i64().is_some_and(|n| n != 0)
        && !output["status_code"].as_u64().is_some_and(|n| n >= 400)
    {
        return None;
    }
    let text = format!(
        "{} {}",
        output["error"].as_str().unwrap_or(""),
        output["output"].as_str().unwrap_or("")
    )
    .to_lowercase();
    if text.contains("noninteractive_input_unavailable:") {
        return Some("owner_input_unavailable");
    }
    if crate::execution_state::missing_dependency_diagnostic(&text) {
        return Some("missing_dependency");
    }
    if [
        "no module named",
        "modulenotfounderror",
        "command not found",
        "not found in path",
    ]
    .iter()
    .any(|s| text.contains(s))
    {
        return Some("missing_dependency");
    }
    if [
        "permission denied",
        "operation not permitted",
        "policy denied",
    ]
    .iter()
    .any(|s| text.contains(s))
    {
        return Some("permission_denied");
    }
    if [401, 403, 429, 502, 503, 504]
        .iter()
        .any(|n| output["status_code"] == *n)
        || [
            "could not resolve host",
            "failed to connect to",
            "connection refused",
            "authentication failed",
            "http error 401",
            "http error 403",
            "http error 503",
        ]
        .iter()
        .any(|s| text.contains(s))
    {
        return Some("external_dependency");
    }
    if output["execution_state"] == "unknown" {
        return Some("uncertain_execution");
    }
    Some("repairable_failure")
}
fn guidance(kind: &str) -> &'static str {
    match kind {
        "missing_dependency" => {
            "Identify the missing dependency and install a compatible version within existing authorization; rerun the failed check. Missing test runners are not terminal blockers."
        }
        "permission_denied" => {
            "Do not retry or bypass the denied operation. Continue independent authorized work, or report this observed denial as the blocker if it prevents the objective."
        }
        "owner_input_unavailable" => {
            "No interactive owner is available. State the missing indispensable requirement or permission; do not invent an answer or expand authorization."
        }
        "external_dependency" => {
            "Inspect connectivity/authentication/service evidence. Do not change credentials or permissions implicitly. Respect Retry-After; ask the owner for a missing prerequisite when needed."
        }
        "uncertain_execution" => {
            "Inspect existing processes and side effects before retrying. Do not replay an uncertain write."
        }
        _ => {
            "Read the concrete diagnostic, make a targeted authorized repair, then rerun that check. Repeating an unchanged command is not a new recovery strategy."
        }
    }
}
impl Runtime {
    pub(crate) fn record_stop_observation(
        &self,
        task: &str,
        call: &str,
        tool: &str,
        args: &Value,
        output: &Value,
        error: bool,
    ) -> Result<()> {
        if !error && crate::execution_state::unchanged_file_edit(tool, output) {
            return Ok(());
        }
        // Failed closing attempts, reads and invented annotations cannot create blocker evidence.
        if matches!(
            tool,
            "task_finish" | "subagent_finish" | "task_contract" | "task_inspect"
        ) {
            return Ok(());
        }
        let mut state = self
            .store
            .get::<StopState>("task_stop_state", task)?
            .unwrap_or_default();
        let process = output["process_id"]
            .as_str()
            .or(args["process_id"].as_str());
        let process_key = process.map(|id| format!("{task}:{id}"));
        if tool == "command_run" {
            if let Some(key) = &process_key {
                self.store
                    .put("stop_process_epoch", key, &state.repair_epoch)?;
            }
        }
        let observation = process_key
            .as_ref()
            .map(|key| self.store.get::<Value>("completion_processes", key))
            .transpose()?
            .flatten();
        if let Some(key) = &process_key {
            if self
                .store
                .get::<bool>("stop_seen_processes", key)?
                .unwrap_or(false)
            {
                return Ok(());
            }
            if output["exit_code"].is_number() {
                self.store.put("stop_seen_processes", key, &true)?;
            } else if !error && output["denied"] != true {
                return Ok(());
            }
        }
        let identity = if let Some(origin) = &observation {
            json!(["command", origin["command"], origin["cwd"]])
        } else {
            json!([tool, args])
        };
        let probe = digest(&identity.to_string());
        let epoch = process_key
            .as_ref()
            .map(|key| self.store.get::<usize>("stop_process_epoch", key))
            .transpose()?
            .flatten()
            .unwrap_or(state.repair_epoch);
        let attempt = digest(&json!([identity, epoch]).to_string());
        let kind = category(output, error);
        if let Some(kind) = kind {
            let diagnostic = crate::tool_receipt::head_tail(
                &format!(
                    "{} {}",
                    output["error"].as_str().unwrap_or(""),
                    output["output"].as_str().unwrap_or("")
                ),
                800,
            );
            // Patch comments/context changes are not progress when the same target
            // still rejects the edit. Keep repair attempts separate from stall identity.
            let target = if tool == "file_patch" {
                let mut paths: Vec<_> = args["patch"]
                    .as_str()
                    .unwrap_or("")
                    .lines()
                    .filter_map(|line| {
                        ["*** Update File: ", "*** Add File: ", "*** Delete File: "]
                            .iter()
                            .find_map(|prefix| line.strip_prefix(prefix))
                    })
                    .collect();
                paths.sort_unstable();
                paths.dedup();
                if paths.is_empty() {
                    json!(probe)
                } else {
                    json!([tool, paths])
                }
            } else {
                json!(probe)
            };
            let failure_key = digest(&json!([target, epoch, kind, diagnostic]).to_string());
            if !state
                .observations
                .iter()
                .any(|o| !o.resolved && o.failure_key == failure_key)
            {
                state.progress += 1;
            }
            state.observations.push(Observation {
                call_id: call.into(),
                probe,
                attempt,
                epoch,
                failure_key,
                category: kind.into(),
                diagnostic,
                resolved: false,
            });
            if state.observations.len() > 64 {
                state.observations.remove(0);
            }
        } else if !error && output["executed"] != false {
            let completed = output["exit_code"] == 0
                || matches!(
                    tool,
                    "file_write"
                        | "file_edit"
                        | "file_patch"
                        | "document_edit"
                        | "connector_call"
                        | "integration_call"
                );
            if completed {
                for previous in &mut state.observations {
                    if previous.probe == probe {
                        previous.resolved = true;
                    }
                }
                state.progress += 1;
                let command = observation
                    .as_ref()
                    .and_then(|o| o["command"].as_str())
                    .or(args["command"].as_str())
                    .unwrap_or("");
                let mutation = matches!(tool, "file_write" | "file_edit" | "file_patch")
                    || [
                        "pip install",
                        "pip uninstall",
                        "build_ext",
                        "cargo build",
                        "npm install",
                        "pnpm install",
                        ".write_text(",
                        ".write_bytes(",
                        ".write(",
                        "sed -i",
                        "perl -i",
                        "patch ",
                    ]
                    .iter()
                    .any(|s| command.contains(s));
                if mutation && state.repairs_seen.insert(probe) {
                    state.repair_epoch += 1;
                }
            }
        }
        self.store.put("task_stop_state", task, &state)?;
        Ok(())
    }
    pub(crate) fn stop_context(&self, task: &str) -> Result<Value> {
        let state = self
            .store
            .get::<StopState>("task_stop_state", task)?
            .unwrap_or_default();
        let mut context = json!({"rejected_stops":state.rejected_stops,"unresolved":state.observations.iter().rev().filter(|o|!o.resolved).take(6).map(|o|json!({"call_id":o.call_id,"category":o.category,"diagnostic":o.diagnostic,"next_action":guidance(&o.category)})).collect::<Vec<_>>(),"instruction":"A failed/blocked close needs stop_reason and blocker_call_id from observed evidence. Repeated commands alone do not prove recovery."});
        if let Some(last) = state.observations.iter().rev().find(|o| !o.resolved)
            && last.category == "repairable_failure"
        {
            let contexts = state
                .observations
                .iter()
                .filter(|o| !o.resolved && o.probe == last.probe && o.category == last.category)
                .map(|o| o.epoch)
                .collect::<HashSet<_>>()
                .len();
            if contexts >= 2 {
                context["recovery_hint"] = json!(
                    "This check still fails after an observed repair. Before another speculative edit, isolate a tiny reproducer and test one falsifiable cause against the first incorrect value or state. Compare a working case and preserve known-good behavior. Run the focused case before scaling to the required checks and deliverables. A slow check needs phase/timing evidence, not a guessed diagnosis or repeated unchanged polling. This advice does not waive permissions or mandatory checks."
                );
            }
        }
        Ok(context)
    }
    pub(crate) fn repeated_failure_pause(&self, task: &str) -> Result<Option<Value>> {
        let state = self
            .store
            .get::<StopState>("task_stop_state", task)?
            .unwrap_or_default();
        let Some(last) = state.observations.iter().rev().find(|o| !o.resolved) else {
            return Ok(None);
        };
        if matches!(
            last.category.as_str(),
            "permission_denied" | "external_dependency" | "owner_input_unavailable"
        ) {
            return Ok(None);
        }
        let repeats = state
            .observations
            .iter()
            .filter(|o| {
                !o.resolved && o.failure_key == last.failure_key && !o.failure_key.is_empty()
            })
            .count();
        if repeats < 3 || last.epoch != state.repair_epoch {
            return Ok(None);
        }
        let result = json!({"status":"blocked","summary":"Execution paused after the same check failed three times without a new observed repair. The task remains unfinished.","blockers":"Repeated unchanged failure requires owner/supervisor intervention; it does not prove that the task is impossible.","evidence":{"call_id":last.call_id,"unchanged_failures":repeats},"stop_reason":"runtime_stagnation"});
        self.store.event(task, "execution_stalled", &result)?;
        Ok(Some(result))
    }
    /// None accepts a grounded stop. Some is a runtime-generated stagnation pause.
    pub(crate) fn validate_task_stop(&self, task: &str, args: &Value) -> Result<Option<Value>> {
        let mut state = self
            .store
            .get::<StopState>("task_stop_state", task)?
            .unwrap_or_default();
        if args["stop_reason"] == "owner_input_unavailable"
            && crate::autonomy::declares_question_optional(&format!(
                "{} {}",
                args["summary"].as_str().unwrap_or(""),
                args["blockers"].as_str().unwrap_or("")
            ))
        {
            bail!(
                "Blocker contradicts your own evidence: you say the question is unnecessary or authorized work can continue. Inspect the available inputs and proceed. Do not convert optional clarification into an owner dependency."
            );
        }
        let reference = args["blocker_call_id"].as_str().unwrap_or("");
        let reason = args["stop_reason"].as_str().unwrap_or("");
        let observed = state
            .observations
            .iter()
            .rev()
            .find(|o| o.call_id == reference && !o.resolved);
        if let Some(observed) = observed {
            let grounded = match reason {
                "permission_denied" | "external_dependency" | "owner_input_unavailable" => {
                    observed.category == reason
                }
                "recovery_exhausted" => {
                    if observed.epoch != state.repair_epoch {
                        false
                    } else {
                        // Three distinct attempted contexts for this still-failing check. A
                        // changed command alone is not proof that the original check failed again.
                        state
                            .observations
                            .iter()
                            .filter(|o| {
                                !o.resolved
                                    && o.probe == observed.probe
                                    && o.category == observed.category
                            })
                            .map(|o| &o.attempt)
                            .collect::<HashSet<_>>()
                            .len()
                            >= 3
                    }
                }
                _ => false,
            };
            if grounded {
                self.store.event(task,"stop_validated",&json!({"reason":reason,"call_id":reference,"status":args["status"],"classification":"observed_diagnostic_not_semantic_proof"}))?;
                return Ok(None);
            }
        }
        if state.progress != state.progress_at_rejection {
            state.rejected_stops = 0;
        }
        state.progress_at_rejection = state.progress;
        state.rejected_stops += 1;
        let last = state.observations.iter().rev().find(|o| !o.resolved);
        let next=last.map(|o|guidance(&o.category)).unwrap_or("No failed operation is recorded. Continue the next unperformed step of the original request; a successful clone or read is preparation, not completion.");
        let repeats = last
            .filter(|o| o.epoch == state.repair_epoch)
            .map(|o| {
                state
                    .observations
                    .iter()
                    .filter(|p| {
                        !p.resolved && p.failure_key == o.failure_key && !p.failure_key.is_empty()
                    })
                    .count()
            })
            .unwrap_or(0);
        let stalled = state.rejected_stops >= 3 || repeats >= 3;
        let detail = json!({"requested_status":args["status"],"blocker_call_id":reference,"rejected_stops":state.rejected_stops,"unchanged_failures":repeats,"next_action":next,"published":false});
        self.store.event(task, "abandonment_rejected", &detail)?;
        self.store.put("task_stop_state", task, &state)?;
        if stalled {
            let result = json!({"status":"blocked","summary":"Execution paused because the agent made no progress after repeated rejected closures or unchanged failures. The requested work remains unfinished.","blockers":"Runtime detected stagnation; owner/supervisor intervention is required. This is not proof that the task is impossible.","evidence":detail,"stop_reason":"runtime_stagnation"});
            self.store.event(task, "execution_stalled", &result)?;
            return Ok(Some(result));
        }
        bail!(
            "Abandonment rejected: no supported terminal blocker. {next} Use task_contract get for pending checks and recovery evidence. Continue authorized work; do not repeat completed effects. For an actual denial or unavailable external prerequisite cite stop_reason and blocker_call_id. Use ask_user if a decision is genuinely missing. Repeating this finish request without progress will pause the task as runtime stagnation, not validate the claimed blocker."
        )
    }
}
#[cfg(test)]
mod tests;
