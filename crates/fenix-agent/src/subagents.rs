//! One-level background delegation, independent of the conversation worker.

use crate::{Runtime, redaction::truncate};
use anyhow::{Result, bail};
use fenix_providers::ProviderError;
use fenix_storage::{Subagent, SubagentSpec};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    sync::{Mutex, Notify, Semaphore},
    task::{Id, JoinSet},
};
use tokio_util::sync::CancellationToken;

const MAX_CONCURRENT_SUBAGENTS: usize = 4;
const MAX_ATTEMPTS: u32 = 3;

pub(crate) struct SubagentManager {
    /// Claiming a job and registering its cancellation token happen under this lock.
    pub(crate) active: Mutex<HashMap<String, CancellationToken>>,
    pub(crate) wake: Notify,
    pub(crate) inference: Semaphore,
}

impl Default for SubagentManager {
    fn default() -> Self {
        Self {
            active: Mutex::new(HashMap::new()),
            wake: Notify::new(),
            inference: Semaphore::new(MAX_CONCURRENT_SUBAGENTS),
        }
    }
}

impl Runtime {
    pub(crate) async fn send_subagent(&self, sender: &str, args: &Value) -> Result<Value> {
        let target = required_text(args, "subagent_id", 100)?;
        self.authorize_task_read(sender, &target)?;
        let text = self.redactor.text(&required_text(args, "message", 8000)?);
        let result = self.store.send_subagent_input(sender, &target, &text)?;
        self.store.event(&target,"subagent_input_queued",&json!({"sender_task_id":sender,"message_id":result["message_id"],"duplicate":result["duplicate"]}))?;
        self.subagents.wake.notify_one();
        self.bus_wake.notify_waiters();
        Ok(result)
    }

    pub(crate) fn consume_subagent_inputs(
        &self,
        task: &str,
        messages: &mut Vec<fenix_core::Message>,
    ) -> Result<()> {
        let inputs = self.store.pending_subagent_inputs(task)?;
        let mut through = 0;
        for (seq, id, sender, content) in inputs {
            through = seq;
            messages.push(fenix_core::Message::user(format!("Supervisor follow-up {id} from task {sender}:\n{content}\nRetain the assigned project and purpose. Treat this as new guidance; do not repeat completed effects. If this changes a plan milestone's objective, ask the master to revise the plan.")));
        }
        if through > 0 {
            self.store.save_subagent_context(task, messages, through)?;
        }
        Ok(())
    }

    pub async fn spawn_subagent(&self, parent: &str, args: &Value) -> Result<Value> {
        // Storage repeats the depth check transactionally. An invented tool call is
        // never enough to give a child permission to delegate.
        let parent_task = self.store.task(parent)?;
        if parent_task.origin == "subagent_report" {
            bail!(
                "A subagent report cannot start another delegation or reset the attempt limit; a new user task is required"
            );
        }
        if !self.store.task_binding(parent)?.bound {
            bail!("Resolve and bind the parent task's scope before delegating work");
        }
        self.authorize_project_tool(parent, "subagent_spawn", args)?;
        if parent_task.status != "running" {
            bail!("Only a running master task can delegate work");
        }
        let mut spec = SubagentSpec {
            objective: required_text(args, "objective", 8_000)?,
            expected_output: required_text(args, "expected_output", 8_000)?,
            context: optional_text(args, "context", 24_000)?,
            constraints: optional_text(args, "constraints", 8_000)?,
        };
        spec.objective = self.redactor.text(&spec.objective);
        spec.expected_output = self.redactor.text(&spec.expected_output);
        spec.context = self.redactor.text(&spec.context);
        spec.constraints = self.redactor.text(&spec.constraints);
        let project = self.readable_project(parent, args)?;
        let job = self
            .store
            .create_subagent_in_project(parent, &spec, project.as_deref())?;
        self.store.inherit_destination(&job.id, parent)?;
        self.subagents.wake.notify_one();
        Ok(json!({
            "subagent_id": job.id,
            "project_id": job.project_id,
            "status": job.status,
            "objective": job.spec.objective,
            "expected_output": job.spec.expected_output,
            "attempts": job.attempts,
            "max_attempts": MAX_ATTEMPTS,
            "delegation_depth": 1,
            "instruction": "This task runs in the background. Continue serving the user; its final report will return to the master automatically."
        }))
    }

    pub fn subagent_status(&self, id: Option<&str>) -> Result<Value> {
        match id {
            Some(id) => Ok(serde_json::to_value(self.store.subagent(id)?)?),
            None => Ok(json!({
                "subagents": self.subagent_views()?,
                "max_concurrent": MAX_CONCURRENT_SUBAGENTS,
                "max_attempts": MAX_ATTEMPTS,
            })),
        }
    }

    /// Small polling snapshots; complete context and evidence stay in task details.
    pub fn subagent_views(&self) -> Result<Value> {
        let views = self
            .store
            .subagents(None, 100)?
            .into_iter()
            .map(|job| {
                let result = job.result.as_ref().map(|result| {
                    let fields = [
                        "summary",
                        "evidence",
                        "artifacts",
                        "blockers",
                        "error",
                        "errors",
                    ]
                    .into_iter()
                    .filter_map(|key| {
                        result.get(key).map(|value| {
                            let preview = match value {
                                Value::String(text) => Value::String(truncate(text, 2_000)),
                                Value::Null => Value::Null,
                                // Older or future reports may contain arrays/objects.
                                // Normalize these as bounded display text, never an
                                // unbounded nested object in each status snapshot.
                                value => Value::String(truncate(&value.to_string(), 2_000)),
                            };
                            (key.to_owned(), preview)
                        })
                    })
                    .collect::<serde_json::Map<_, _>>();
                    Value::Object(fields)
                });
                json!({
                    "id": job.id,
                    "project_id": job.project_id,
                    "parent_task_id": job.parent_task_id,
                    "spec": {
                        "objective": truncate(&job.spec.objective, 1_000),
                        "expected_output": truncate(&job.spec.expected_output, 1_000),
                    },
                    "status": job.status,
                    "attempts": job.attempts,
                    "created_at": job.created_at,
                    "updated_at": job.updated_at,
                    "result": result,
                    "detail_url": format!("/api/tasks/{}", job.id),
                })
            })
            .collect::<Vec<_>>();
        Ok(Value::Array(views))
    }

    pub async fn cancel_subagent(&self, id: &str) -> Result<Value> {
        let active = self.subagents.active.lock().await;
        let job = self.store.subagent(id)?;
        if is_terminal(&job.status) {
            bail!("Subagent is already {}", job.status);
        }
        if let Some(cancel) = active.get(id) {
            self.store
                .event(id, "cancellation_requested", &json!({"scope":"subagent"}))?;
            cancel.cancel();
            return Ok(json!({"subagent_id": id, "cancellation_requested": true}));
        }
        if job.status != "queued" {
            bail!("Subagent has no active worker; inspect its saved task events");
        }
        self.store.finish_subagent(
            id,
            "cancelled",
            &failure_report(
                "cancelled",
                "The queued subagent was cancelled before execution.",
            ),
        )?;
        drop(active);
        self.queue_subagent_report(id)?;
        Ok(json!({"subagent_id": id, "status": "cancelled"}))
    }

    pub(crate) async fn subagent_worker(self: Arc<Self>) {
        let mut jobs = JoinSet::new();
        let mut task_ids = HashMap::<Id, String>::new();
        loop {
            if self.stop.is_cancelled() {
                break;
            }
            // A terminal record is durable before its report is enqueued. This
            // sweep closes the crash window between those two operations.
            // Drop the model read guard before acquiring configure_lock below.
            // configure() holds configure_lock before replacing the model.
            let model_ready = self.model.read().await.is_some();
            if model_ready {
                if let Err(error) = self.reconcile_subagent_reports() {
                    self.log_subagent_error("Subagent report queue error", &error);
                }
                while jobs.len() < 32 && !self.stop.is_cancelled() {
                    let running = self
                        .subagents
                        .active
                        .lock()
                        .await
                        .keys()
                        .filter(|id| self.store.task(id).is_ok_and(|t| t.status != "waiting"))
                        .count();
                    if running >= MAX_CONCURRENT_SUBAGENTS {
                        break;
                    }

                    let claimed = {
                        let _configure = self.configure_lock.lock().await;
                        let mut active = self.subagents.active.lock().await;
                        match self
                            .store
                            .claim_subagent_excluding(&active.keys().cloned().collect::<Vec<_>>())
                        {
                            Ok(Some(job)) => {
                                let cancel = self.stop.child_token();
                                active.insert(job.id.clone(), cancel.clone());
                                Some((job, cancel))
                            }
                            Ok(None) => None,
                            Err(error) => {
                                self.log_subagent_error("Subagent task queue error", &error);
                                None
                            }
                        }
                    };
                    let Some((job, cancel)) = claimed else { break };
                    let runtime = self.clone();
                    let id = job.id.clone();
                    let handle =
                        jobs.spawn(async move { runtime.run_subagent_job(job, cancel).await });
                    task_ids.insert(handle.id(), id);
                }
            }
            tokio::select! {
                biased;
                _ = self.stop.cancelled() => break,
                joined = jobs.join_next_with_id(), if !jobs.is_empty() => {
                    self.handle_subagent_join(joined, &mut task_ids).await;
                }
                _ = self.subagents.wake.notified() => {},
                _ = tokio::time::sleep(Duration::from_millis(250)) => {},
            }
        }
        // stop is the parent of every child token. Await workers so no managed
        // command or waiting approval survives a graceful runtime shutdown.
        for cancel in self.subagents.active.lock().await.values() {
            cancel.cancel();
        }
        while let Some(joined) = jobs.join_next_with_id().await {
            self.handle_subagent_join(Some(joined), &mut task_ids).await;
        }
        match self.store.subagents(None, 200) {
            Ok(jobs) => {
                for job in jobs.into_iter().filter(|job| job.status == "queued") {
                    if let Err(error) = self.store.finish_subagent(
                        &job.id,
                        "interrupted",
                        &failure_report("interrupted", "The service stopped before this queued subagent began. Actions were not resumed."),
                    ) {
                        self.log_subagent_error("Unable to interrupt queued subagent", &error);
                    }
                }
            }
            Err(error) => self
                .log_subagent_error("Unable to inspect queued subagents during shutdown", &error),
        }
    }

    async fn handle_subagent_join(
        &self,
        joined: Option<Result<(Id, Result<()>), tokio::task::JoinError>>,
        task_ids: &mut HashMap<Id, String>,
    ) {
        let Some(joined) = joined else { return };
        let (id, error) = match joined {
            Ok((id, result)) => (id, result.err()),
            Err(error) => (
                error.id(),
                Some(anyhow::anyhow!(
                    "Subagent worker stopped unexpectedly: {error}"
                )),
            ),
        };
        let Some(job_id) = task_ids.remove(&id) else {
            return;
        };
        if let Some(error) = error {
            self.settle_task(&job_id).await;
            self.clear_pending(&job_id).await;
            let status = if self.stop.is_cancelled() {
                "interrupted"
            } else {
                "failed"
            };
            let result = failure_report(status, &self.redactor.text(&error.to_string()));
            if let Err(error) = self.store.finish_subagent(&job_id, status, &result) {
                self.log_subagent_error("Unable to record subagent worker failure", &error);
            }
            self.log_subagent_error("Subagent worker error", &error);
        }
        self.subagents.active.lock().await.remove(&job_id);
        if !self.stop.is_cancelled()
            && let Err(error) = self.queue_subagent_report(&job_id)
        {
            self.log_subagent_error("Unable to queue subagent report", &error);
        }
    }

    async fn run_subagent_job(
        self: &Arc<Self>,
        mut job: Subagent,
        cancel: CancellationToken,
    ) -> Result<()> {
        let model = self
            .model
            .read()
            .await
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Model is not configured"))?;
        let mut messages = match self
            .store
            .get::<Vec<fenix_core::Message>>("subagent_context", &job.id)?
        {
            Some(messages) => messages,
            None => self.build_subagent_context(&job, &model).await?,
        };
        let attempt_timeout =
            Duration::from_secs(model.settings.execution.subagent_timeout_seconds);
        let mut count_attempt = job.attempts == 0;
        let outcome = loop {
            let candidate = loop {
                if cancel.is_cancelled() {
                    break self.subagent_cancelled_report();
                }
                if count_attempt {
                    self.store.start_subagent_attempt(&job.id)?;
                    count_attempt = false;
                }
                job = self.store.subagent(&job.id)?;
                self.store.event(
                    &job.id,
                    "subagent_attempt_started",
                    &json!({
                        "attempt": job.attempts, "max_attempts": MAX_ATTEMPTS,
                        "provider": model.settings.provider, "model": model.settings.model,
                        "timeout_seconds": attempt_timeout.as_secs(),
                    }),
                )?;
                let result = tokio::select! {
                    biased;
                    _ = cancel.cancelled() => break self.subagent_cancelled_report(),
                    _ = self.execution_deadline(&job.id, attempt_timeout) => Err(anyhow::anyhow!("Subagent exceeded its configured attempt time; inspect evidence before repeating actions")),
                    result = self.run_subagent_attempt(&job, &model, &mut messages, &cancel) => result,
                };
                if cancel.is_cancelled() {
                    break self.subagent_cancelled_report();
                }
                match result {
                    Ok(mut outcome) => {
                        self.redactor.value(&mut outcome);
                        if !matches!(
                            outcome["status"].as_str(),
                            Some("completed" | "blocked" | "failed")
                        ) {
                            break failure_report(
                                "failed",
                                "Subagent returned an invalid completion status.",
                            );
                        }
                        break outcome;
                    }
                    Err(error) => {
                        let explanation = self.redactor.text(&error.to_string());
                        self.store.event(
                            &job.id,
                            "subagent_attempt_failed",
                            &json!({
                                "attempt": job.attempts, "error": explanation,
                            }),
                        )?;
                        let retry = error
                            .downcast_ref::<ProviderError>()
                            .filter(|error| error.is_retryable());
                        let Some(provider_error) = retry.filter(|_| job.attempts < MAX_ATTEMPTS)
                        else {
                            break failure_report("failed", &explanation);
                        };
                        count_attempt = true;
                        let delay = match provider_error {
                            ProviderError::RateLimited {
                                retry_after: Some(delay),
                            } => (*delay).min(Duration::from_secs(30)),
                            _ => Duration::from_secs(1 << job.attempts.saturating_sub(1)),
                        };
                        self.store.event(&job.id, "subagent_retry", &json!({
                        "next_attempt": job.attempts + 1, "delay_ms": delay.as_millis(),
                        "resume": "Retain prior messages and completed tool results; retry inference only.",
                    }))?;
                        tokio::select! {
                            _ = cancel.cancelled() => break self.subagent_cancelled_report(),
                            _ = tokio::time::sleep(delay) => {},
                        }
                    }
                }
            };
            if candidate["status"] == "completed" && !cancel.is_cancelled() {
                self.processes.cancel_foreground(&job.id).await;
                self.file_effects.wait(Some(&job.id)).await;
            } else {
                self.settle_task(&job.id).await;
            }
            self.clear_pending(&job.id).await;
            let candidate = if cancel.is_cancelled() {
                self.subagent_cancelled_report()
            } else {
                candidate
            };
            if self.store.finish_subagent_unless_input(
                &job.id,
                candidate["status"].as_str().unwrap_or("failed"),
                &candidate,
            )? {
                break candidate;
            }
            // An input arrived while this result was being finalized. Continue the
            // same context without another attempt or replaying completed effects.
        };
        let _ = outcome;
        Ok(())
    }

    fn subagent_cancelled_report(&self) -> Value {
        if self.stop.is_cancelled() {
            failure_report(
                "interrupted",
                "The service stopped. Completed effects remain recorded; actions were not resumed.",
            )
        } else {
            failure_report(
                "cancelled",
                "The subagent was cancelled. Completed effects remain recorded and are not automatically reverted.",
            )
        }
    }

    fn reconcile_subagent_reports(&self) -> Result<()> {
        for job in self.store.unreported_subagents(200)? {
            self.queue_subagent_report(&job.id)?;
        }
        Ok(())
    }

    fn queue_subagent_report(&self, id: &str) -> Result<()> {
        if self.stop.is_cancelled() {
            return Ok(());
        }
        if self.store.review_for_worker(id)?.is_some() {
            self.bus_wake.notify_waiters();
            return Ok(());
        }
        if self.store.reconcile_plan_worker(id)? {
            self.bus_wake.notify_waiters();
            return Ok(());
        }
        if let Some(report) = self.store.enqueue_subagent_report(id)? {
            self.store.inherit_destination(&report.id, id)?;
        }
        self.wake.notify_one();
        Ok(())
    }

    fn log_subagent_error(&self, prefix: &str, error: &anyhow::Error) {
        eprintln!("{prefix}: {}", self.redactor.text(&error.to_string()));
    }
}

fn failure_report(status: &str, error: &str) -> Value {
    let error = truncate(error, 8_000);
    json!({
        "status": status,
        "summary": error,
        "artifacts": "",
        "evidence": "Detailed tool results and attempt events remain in this subagent's task journal.",
        "blockers": error,
    })
}

fn required_text(args: &Value, key: &str, maximum: usize) -> Result<String> {
    let value = args[key].as_str().unwrap_or("").trim();
    if value.is_empty() || value.len() > maximum {
        bail!("{key} must contain 1–{maximum} bytes of text");
    }
    Ok(value.to_owned())
}

fn optional_text(args: &Value, key: &str, maximum: usize) -> Result<String> {
    if args.get(key).is_none() {
        return Ok(String::new());
    }
    let value = args[key]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("{key} must be text"))?;
    if value.len() > maximum {
        bail!("{key} must contain at most {maximum} bytes");
    }
    Ok(value.trim().to_owned())
}

fn is_terminal(status: &str) -> bool {
    matches!(
        status,
        "completed" | "blocked" | "failed" | "cancelled" | "interrupted"
    )
}

#[cfg(test)]
mod lock_order_tests {
    use super::*;
    #[tokio::test]
    async fn idle_worker_does_not_hold_model_while_waiting_for_configuration() {
        let dir = tempfile::tempdir().unwrap();
        let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
            .await
            .unwrap();
        let model = rt
            .model_connection(
                crate::config::Settings::initial(dir.path().into()),
                fenix_credentials::ApiKey::new("fake-lock-order-key").unwrap(),
            )
            .unwrap();
        *rt.model.write().await = Some(Arc::new(model));
        let configuration = rt.configure_lock.lock().await;
        let worker = tokio::spawn(rt.clone().subagent_worker());
        tokio::time::sleep(Duration::from_millis(50)).await;
        let write = tokio::time::timeout(Duration::from_secs(1), rt.model.write()).await;
        drop(configuration);
        assert!(
            write.is_ok(),
            "Worker inverted the model/configuration lock order"
        );
        drop(write);
        rt.stop.cancel();
        tokio::time::timeout(Duration::from_secs(2), worker)
            .await
            .unwrap()
            .unwrap();
    }
}
