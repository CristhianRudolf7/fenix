use crate::{
    Runtime,
    config::{ActionPolicy, ModelConnection},
    permissions::ActiveTask,
    progress::{Phase, Progress},
    tools,
};
use anyhow::{Context, Result, bail};
use fenix_core::{ContentBlock, FinishReason, Message, Role};
use fenix_storage::Task;
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio_util::sync::CancellationToken;

#[cfg(test)]
mod operational_tests {
    use super::*;
    use fenix_credentials::ApiKey;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[tokio::test]
    async fn saved_output_search_uses_redacted_journal_and_preserves_worker_scope() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
        let task = rt.store.enqueue_bound(
            "Investigate",
            "web",
            None,
            &fenix_storage::TaskBinding {
                bound: true,
                ..Default::default()
            },
        )?;
        rt.redactor.add("privatecredentialfixture");
        rt.record_tool_outcome(
            &task.id,
            "build-result",
            "command_run",
            &json!({}),
            Ok(json!({"exit_code":1,"output":"diagnosticmarker privatecredentialfixture"})),
        )?;
        let settings = crate::config::Settings::initial(dir.path().into());
        let cancel = CancellationToken::new();
        let page = rt
            .execute_tool(
                &task.id,
                "page",
                &settings,
                "task_inspect",
                &json!({"after":0,"limit":1}),
                &cancel,
            )
            .await?;
        assert!(page.get("contract").is_none());
        assert_eq!(page["events"].as_array().unwrap().len(), 1);
        assert_eq!(page["next_after"], page["events"][0]["seq"]);
        let next = rt
            .execute_tool(
                &task.id,
                "next",
                &settings,
                "task_inspect",
                &json!({"after":page["next_after"],"limit":1}),
                &cancel,
            )
            .await?;
        assert!(
            next["events"]
                .as_array()
                .unwrap()
                .iter()
                .all(|event| event["seq"].as_i64().unwrap() > page["next_after"].as_i64().unwrap())
        );
        let found = rt
            .execute_tool(
                &task.id,
                "search",
                &settings,
                "task_inspect",
                &json!({"query":"diagnosticmarker"}),
                &cancel,
            )
            .await?;
        assert_eq!(found["results"][0]["call_id"], "build-result");
        assert!(!found.to_string().contains("privatecredentialfixture"));
        let absent = rt
            .execute_tool(
                &task.id,
                "search",
                &settings,
                "task_inspect",
                &json!({"query":"privatecredentialfixture"}),
                &cancel,
            )
            .await?;
        assert_eq!(absent["results"], json!([]));
        let worker = rt.store.create_subagent(
            &task.id,
            &fenix_storage::SubagentSpec {
                profile: "general".into(),
                objective: "Independent review".into(),
                expected_output: "Report".into(),
                context: String::new(),
                constraints: String::new(),
            },
        )?;
        assert!(
            rt.execute_tool(
                &worker.id,
                "search",
                &settings,
                "task_inspect",
                &json!({"task_id":task.id,"query":"diagnosticmarker"}),
                &cancel
            )
            .await
            .is_err()
        );
        assert!(
            rt.execute_tool(
                &task.id,
                "search",
                &settings,
                "task_inspect",
                &json!({"query":"diagnosticmarker","checkpoint_id":"ambiguous"}),
                &cancel
            )
            .await
            .is_err()
        );
        rt.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn execution_cannot_complete_with_a_progress_only_final_answer() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
        let server = MockServer::start().await;
        let count = AtomicUsize::new(0);
        Mock::given(method("POST")).and(path("/responses")).respond_with(move |_: &wiremock::Request| {
            let output = match count.fetch_add(1, Ordering::SeqCst) {
                0 => json!([{"type":"function_call","call_id":"write","name":"file_write","arguments":json!({"path":"artifact.txt","content":"partial work"}).to_string()}]),
                1 => json!([{"type":"message","role":"assistant","content":[{"type":"output_text","text":"I have not finished the requested verification."}]}]),
                _ => json!([{"type":"function_call","call_id":"finish","name":"task_finish","arguments":json!({"status":"blocked","summary":"Partial artifact saved; external fixture unavailable.","blockers":"Required fixture is unavailable"}).to_string()}]),
            };
            ResponseTemplate::new(200).set_body_json(json!({"id":"response","status":"completed","output":output,"usage":{"input_tokens":10,"output_tokens":10,"total_tokens":20}}))
        }).mount(&server).await;
        let mut settings = crate::config::Settings::initial(dir.path().into());
        settings.base_url = Some(server.uri());
        settings.file_write_policy = ActionPolicy::Allow;
        settings.coding.context_window_tokens = 65536;
        settings.coding.compact_at_tokens = 30000;
        let model = rt.model_connection(settings, ApiKey::new("test-completion-key")?)?;
        let task = rt.store.enqueue_bound(
            "Create and verify artifact.txt",
            "web",
            None,
            &fenix_storage::TaskBinding {
                bound: true,
                ..Default::default()
            },
        )?;
        rt.store.set_task_status(&task.id, "running", None)?;
        let mut messages = vec![Message::user(task.request.clone())];
        let result = rt
            .run_loop(&task, &model, &mut messages, &CancellationToken::new())
            .await?;
        assert_eq!(result["status"], "blocked");
        assert_eq!(
            std::fs::read_to_string(dir.path().join("artifact.txt"))?,
            "partial work"
        );
        assert_eq!(server.received_requests().await.unwrap().len(), 3);
        let events = rt.store.events(&task.id, 0, 100)?;
        assert!(
            events
                .iter()
                .any(|e| e["kind"] == "premature_completion" && e["data"]["published"] == false)
        );
        assert!(events.iter().any(|e| e["kind"] == "tool_result"
            && e["data"]["tool"] == "task_finish"
            && e["data"]["result"]["status"] == "blocked"));
        rt.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn master_can_release_turn_only_for_its_own_active_delegation() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
        let binding = fenix_storage::TaskBinding {
            bound: true,
            ..Default::default()
        };
        let task = rt
            .store
            .enqueue_bound("Build and verify", "web", None, &binding)?;
        let other = rt
            .store
            .enqueue_bound("Unrelated work", "web", None, &binding)?;
        let spec = fenix_storage::SubagentSpec {
            profile: "general".into(),
            objective: "Verify build".into(),
            expected_output: "Evidence".into(),
            context: String::new(),
            constraints: String::new(),
        };
        rt.store.create_subagent(&other.id, &spec)?;
        assert!(!rt.store.has_active_delegation(&task.id)?);
        let worker = rt.store.create_subagent(&task.id, &spec)?;
        assert!(rt.store.has_active_delegation(&task.id)?);
        rt.store
            .put("explicit_execution_outcome", &task.id, &true)?;
        let server = MockServer::start().await;
        Mock::given(method("POST")).and(path("/responses")).respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "id":"handoff","status":"completed","output":[{"type":"message","role":"assistant",
                "content":[{"type":"output_text","text":"Verification is running; I will report its result."}]}],
                "usage":{"input_tokens":10,"output_tokens":10,"total_tokens":20}
            }))
        ).mount(&server).await;
        let mut settings = crate::config::Settings::initial(dir.path().into());
        settings.base_url = Some(server.uri());
        let model = rt.model_connection(settings, ApiKey::new("test-handoff-key")?)?;
        let result = rt
            .run_loop(
                &task,
                &model,
                &mut vec![Message::user(task.request.clone())],
                &CancellationToken::new(),
            )
            .await?;
        assert_eq!(result["status"], "delegated");
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
        assert_eq!(rt.store.task(&worker.id)?.status, "queued");
        assert!(
            !rt.store
                .events(&task.id, 0, 100)?
                .iter()
                .any(|e| e["kind"] == "premature_completion")
        );
        rt.store.set_task_status(&worker.id, "completed", None)?;
        assert!(!rt.store.has_active_delegation(&task.id)?);
        rt.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn failed_tool_reaches_next_inference_without_duplicate_history() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
        let server = MockServer::start().await;
        let count = AtomicUsize::new(0);
        Mock::given(method("POST")).and(path("/responses")).respond_with(move |_: &wiremock::Request| {
            let output=if count.fetch_add(1,Ordering::SeqCst)==0 {
                json!([{"type":"function_call","call_id":"missing-file","name":"file_read","arguments":"{\"path\":\"missing.txt\"}"}])
            } else {
                json!([{"type":"message","role":"assistant","content":[{"type":"output_text","text":"The requested file is absent."}]}])
            };
            ResponseTemplate::new(200).set_body_json(json!({"id":"response","status":"completed","output":output,"usage":{"input_tokens":10,"output_tokens":10,"total_tokens":20}}))
        }).mount(&server).await;
        let mut settings = crate::config::Settings::initial(dir.path().into());
        settings.base_url = Some(server.uri());
        settings.coding.context_window_tokens = 65536;
        settings.coding.compact_at_tokens = 30000;
        let model = rt.model_connection(settings, ApiKey::new("test-operational-key")?)?;
        let task = rt.store.enqueue_bound(
            "Read missing.txt",
            "web",
            None,
            &fenix_storage::TaskBinding {
                bound: true,
                ..Default::default()
            },
        )?;
        rt.store.set_task_status(&task.id, "running", None)?;
        let mut messages = vec![Message::user("Read missing.txt")];
        rt.run_loop(&task, &model, &mut messages, &CancellationToken::new())
            .await?;
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 2);
        let first = String::from_utf8_lossy(&requests[0].body);
        let second = String::from_utf8_lossy(&requests[1].body);
        assert!(!first.contains("Runtime operational snapshot"));
        assert_eq!(second.matches("Runtime operational snapshot").count(), 1);
        let second_body: Value = serde_json::from_slice(&requests[1].body)?;
        assert_eq!(second_body["input"][0]["role"], "user");
        assert!(second.contains("failed_approaches"));
        assert!(second.contains("missing-file"));
        assert!(
            !messages
                .iter()
                .any(|m| m.text_content().contains("Runtime operational snapshot"))
        );
        assert_eq!(
            rt.task_contract(&task.id)?["failed_approaches"][0]["call_id"],
            "missing-file"
        );
        // A completed command with a nonzero exit is also a failure, even when
        // the tool transport itself succeeded.
        rt.record_tool_outcome(
            &task.id,
            "bad-exit",
            "command_run",
            &json!({"command":"build"}),
            Ok(json!({"exit_code":2,"status":"completed"})),
        )?;
        assert_eq!(
            rt.task_contract(&task.id)?["failed_approaches"][1]["call_id"],
            "bad-exit"
        );
        rt.shutdown().await?;
        Ok(())
    }
}

impl Runtime {
    pub(crate) fn record_tool_outcome(
        &self,
        task: &str,
        call_id: &str,
        name: &str,
        args: &Value,
        result: Result<Value>,
    ) -> Result<(Value, bool)> {
        let (mut output, is_error) = match result {
            Err(error)
                if error
                    .downcast_ref::<crate::processes::BusyProcess>()
                    .is_some() =>
            {
                (
                    serde_json::to_value(
                        error
                            .downcast_ref::<crate::processes::BusyProcess>()
                            .unwrap(),
                    )?,
                    true,
                )
            }
            Err(error) if error.downcast_ref::<tools::ArgumentFailure>().is_some() => (
                serde_json::to_value(error.downcast_ref::<tools::ArgumentFailure>().unwrap())?,
                true,
            ),
            Err(error) if tools::process_reference_failure(&error).is_some() => {
                (tools::process_reference_failure(&error).unwrap(), true)
            }
            Err(error) if name == "command_run" => (
                json!({"code":"command_execution_error","category":"execution_control","error":self.redactor.text(&error.to_string()),"execution_state":"unknown","permissions_relaxed":false,"instruction":"Execution setup or policy failed. Inspect existing processes/evidence before retrying; never broaden permissions automatically."}),
                true,
            ),
            Ok(value) => (value, false),
            Err(error) => (
                json!({"error":self.redactor.text(&error.to_string())}),
                true,
            ),
        };
        self.redactor.value(&mut output);
        let uncertain_write = (is_error
            || output["exit_code"].as_i64().is_some_and(|code| code != 0))
            && (matches!(
                name,
                "command_run"
                    | "command_input"
                    | "file_write"
                    | "file_edit"
                    | "file_patch"
                    | "document_create"
                    | "document_edit"
                    | "library_export"
                    | "connector_call"
                    | "integration_call"
                    | "file_history"
            ) || name.starts_with("browser_"));
        if uncertain_write {
            self.store.put("restart_uncertain_effects", task, &true)?;
        }
        self.store.event(
            task,
            "tool_result",
            &json!({"call_id":call_id,"tool":name,"result":output,"is_error":is_error}),
        )?;
        if matches!(
            name,
            "command_run" | "command_input" | "file_write" | "file_edit" | "file_patch"
        ) {
            self.store.put("explicit_execution_outcome", task, &true)?;
        }
        if name != "task_contract" {
            let mut contract = self.task_contract(task)?;
            if !contract["recent_tool_evidence"].is_array() {
                contract["recent_tool_evidence"] = json!([]);
            }
            let evidence = contract["recent_tool_evidence"]
                .as_array_mut()
                .context("Invalid task evidence")?;
            evidence.push(json!({"call_id":call_id,"tool":name,"is_error":is_error,"executed":output.get("executed"),"process_id":output.get("process_id"),"status":output.get("status"),"exit_code":output.get("exit_code"),"code":output.get("code"),"error":output.get("error"),"artifact":output.get("path"),"source":"task_inspect call_id; tool success alone is not task success"}));
            if evidence.len() > 16 {
                evidence.remove(0);
            }
            self.remember_failed_approach(
                task,
                call_id,
                name,
                args,
                &output,
                is_error,
                &mut contract,
            )?;
            self.store.put("task_contracts", task, &contract)?;
        }
        Ok((output, is_error))
    }

    pub(crate) async fn worker(self: Arc<Self>) {
        let mut jobs = tokio::task::JoinSet::new();
        let mut ids = HashMap::new();
        while !self.stop.is_cancelled() {
            // Waiting questions release execution capacity. Keep a bounded total
            // of live turns; control replies and edits never pass through this queue.
            let running = self
                .active
                .lock()
                .await
                .keys()
                .filter(|id| self.store.task(id).is_ok_and(|t| t.status != "waiting"))
                .count();
            if jobs.len() < 65 {
                let _config = self.configure_lock.lock().await;
                let mut active = self.active.lock().await;
                match self
                    .store
                    .claim_next_control(running >= 8 || jobs.len() >= 64)
                {
                    Ok(Some(task)) => {
                        let cancel = self.stop.child_token();
                        active.insert(
                            task.id.clone(),
                            ActiveTask {
                                cancel: cancel.clone(),
                                revision: CancellationToken::new(),
                            },
                        );
                        let runtime = self.clone();
                        let id = task.id.clone();
                        let handle =
                            jobs.spawn(async move { runtime.process_task(task, cancel).await });
                        ids.insert(handle.id(), id);
                        continue;
                    }
                    Ok(None) => {}
                    Err(error) => {
                        eprintln!("Task queue: {}", self.redactor.text(&error.to_string()))
                    }
                }
            }
            tokio::select! {
                _ = self.stop.cancelled() => break,
                joined = jobs.join_next_with_id(), if !jobs.is_empty() => { self.finish_master_join(joined, &mut ids).await; },
                _ = self.wake.notified() => {},
                _ = tokio::time::sleep(Duration::from_millis(100)) => {},
            }
        }
        while let Some(joined) = jobs.join_next_with_id().await {
            self.finish_master_join(Some(joined), &mut ids).await;
        }
    }

    async fn finish_master_join(
        &self,
        joined: Option<Result<(tokio::task::Id, ()), tokio::task::JoinError>>,
        ids: &mut HashMap<tokio::task::Id, String>,
    ) {
        let Some(joined) = joined else { return };
        let (id, error) = match joined {
            Ok((id, ())) => (id, None),
            Err(error) => (error.id(), Some(error.to_string())),
        };
        if let Some(task) = ids.remove(&id) {
            if let Some(error) = error {
                self.settle_task(&task).await;
                self.clear_pending(&task).await;
                let _ = self.store.discard_context(&task);
                let _ =
                    self.store
                        .set_task_status(&task, "failed", Some(&self.redactor.text(&error)));
                let _ = self.notify(
                    Some(&task),
                    "La ejecución se interrumpió. El historial de acciones sigue disponible.",
                );
            }
            self.active.lock().await.remove(&task);
        }
    }

    async fn process_task(self: &Arc<Self>, task: Task, cancel: CancellationToken) {
        let limits = self
            .settings()
            .ok()
            .flatten()
            .map(|s| s.execution)
            .unwrap_or_default();
        let deadline = async {
            if limits.externally_supervised {
                std::future::pending::<()>().await;
            } else {
                self.execution_deadline(&task.id, Duration::from_secs(limits.task_timeout_seconds))
                    .await;
            }
        };
        tokio::pin!(deadline);
        let result = loop {
            let revision = {
                let mut active = self.active.lock().await;
                let Some(active) = active.get_mut(&task.id) else {
                    return;
                };
                if active.revision.is_cancelled() {
                    active.revision = CancellationToken::new();
                }
                active.revision.clone()
            };
            let execution = cancel.child_token();
            let result = tokio::select! {
                biased;
                _ = cancel.cancelled() => Some(Err(anyhow::anyhow!("Task cancelled"))),
                _ = revision.cancelled() => None,
                _ = &mut deadline => Some(Err(anyhow::anyhow!("Task exceeded its configured active execution time"))),
                result = self.run_task(&task, &execution) => Some(result),
            };
            execution.cancel();
            if result.as_ref().is_some_and(Result::is_ok) {
                self.processes.cancel_foreground(&task.id).await;
                self.file_effects.wait(Some(&task.id)).await;
            } else {
                self.settle_task(&task.id).await;
            }
            self.clear_pending(&task.id).await;
            if let Some(result) = result {
                break result;
            }
            let _ = self.store.discard_context(&task.id);
            let _ = self.store.set_task_status(&task.id, "running", None);
            let _ = self.store.publish_event(
                Some(&task.id),
                "task.replanning",
                &json!({"task_id":task.id}),
            );
        };
        let (status, error) = match result {
            Ok(outcome) => match outcome["status"].as_str() {
                Some("blocked") => (
                    "blocked",
                    Some(
                        outcome["blockers"]
                            .as_str()
                            .unwrap_or("Execution blocked")
                            .to_owned(),
                    ),
                ),
                Some("failed") => (
                    "failed",
                    Some(
                        outcome["blockers"]
                            .as_str()
                            .unwrap_or("Execution failed")
                            .to_owned(),
                    ),
                ),
                _ => ("completed", None),
            },
            Err(error) => {
                let status = if self.stop.is_cancelled() {
                    "interrupted"
                } else if cancel.is_cancelled() {
                    "cancelled"
                } else {
                    "failed"
                };
                let error = self.redactor.text(&error.to_string());
                let _ = self.store.discard_context(&task.id);
                let _ = self.notify(
                    Some(&task.id),
                    &format!("Tarea {} · {status}\n{error}", task.id),
                );
                let _ = self.store.append(Some(&task.id), &Message::user(format!("Previous request: {}\nOutcome: {status}: {error}. Inspect saved task events before repeating effects.",task.request)),false,true,"runtime");
                (status, Some(error))
            }
        };
        let _ = self.execution_stage(&task.id, status);
        let _ = self
            .store
            .finish_master_context(&task.id, status, error.as_deref());
        let _ = self.store.event(
            &task.id,
            "task_finished",
            &json!({"status":status,"error":error}),
        );
        let _ = self.store.publish_event(
            Some(&task.id),
            "task.finished",
            &json!({"status":status,"error":error}),
        );
        if status == "completed"
            && !self
                .store
                .get::<bool>("execution_handoff", &task.id)
                .ok()
                .flatten()
                .unwrap_or(false)
        {
            let _ = self.queue_skill_learning(&task.id);
        }
        self.active.lock().await.remove(&task.id);
        self.progress.send_if_modified(|progress| {
            if progress.task_id.as_deref() == Some(&task.id) {
                *progress = Progress::default();
                true
            } else {
                false
            }
        });
        self.bus_wake.notify_waiters();
    }

    /// Human response time does not consume the task's execution deadline.
    pub(crate) async fn execution_deadline(&self, task: &str, budget: Duration) {
        let mut remaining = budget;
        loop {
            let waiting = self.store.task(task).is_ok_and(|t| t.status == "waiting");
            let started = tokio::time::Instant::now();
            tokio::time::sleep(Duration::from_millis(100)).await;
            if !waiting {
                remaining = remaining.saturating_sub(started.elapsed());
            }
            if remaining.is_zero() {
                return;
            }
        }
    }

    async fn run_task(self: &Arc<Self>, task: &Task, cancel: &CancellationToken) -> Result<Value> {
        self.report_progress(&task.id, Phase::Preparing, None);
        let model = self
            .model
            .read()
            .await
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Model is not configured"))?;
        self.route_task(task, &model, cancel).await?;
        self.queue_memory_capture(task, None)?;
        let binding = self.store.task_binding(&task.id)?;
        self.store.event(&task.id,"task_started",&json!({"session_id":task.session_id,"objective":task.request,"provider":model.settings.provider,"model":model.settings.model,"reasoning_effort":model.settings.reasoning_effort,"workspace":self.task_workspace(&task.id,&model.settings.workspace)?,"binding":binding}))?;
        self.store
            .put("repository_rules_epoch", &task.id, &fenix_storage::id())?;
        let mut messages = match self.recovered_messages(&task.id)? {
            Some(messages) => messages,
            None => self.build_context(task, &model).await?,
        };
        self.run_loop(task, &model, &mut messages, cancel).await
    }

    pub(crate) async fn run_subagent_attempt(
        self: &Arc<Self>,
        job: &fenix_storage::Subagent,
        model: &ModelConnection,
        messages: &mut Vec<Message>,
        cancel: &CancellationToken,
    ) -> Result<Value> {
        let task = self.store.task(&job.id)?;
        let result = self.run_loop(&task, model, messages, cancel).await;
        let stage = result
            .as_ref()
            .ok()
            .and_then(|v| v["status"].as_str())
            .unwrap_or("failed");
        self.execution_stage(&task.id, stage)?;
        result
    }

    async fn run_loop(
        self: &Arc<Self>,
        task: &Task,
        model: &ModelConnection,
        messages: &mut Vec<Message>,
        cancel: &CancellationToken,
    ) -> Result<Value> {
        let delegated = task.origin == "subagent";
        let source = if delegated {
            task.id.clone()
        } else {
            self.store
                .task_user_message(&task.id)?
                .map(|m| m.id)
                .unwrap_or_else(|| task.id.clone())
        };
        let mut repetition = crate::execution_state::LoopGuard::default();
        let mut browser_progress = crate::execution_state::BrowserProgress::default();
        messages.push(Message::user(format!(
            "Persistent task contract (bounded observational history): {}",
            crate::execution_state::context_contract(&self.task_contract(&task.id)?)
        )));
        let mut grants = self
            .store
            .get::<Vec<Value>>("task_grants", &task.id)?
            .unwrap_or_default();
        let mut denials = self
            .store
            .get::<Vec<Value>>("task_denials", &task.id)?
            .unwrap_or_default();
        let step_limit = if model.settings.execution.externally_supervised {
            usize::MAX
        } else {
            model.settings.execution.max_model_steps
        };
        for _ in 0..step_limit {
            let step = self
                .store
                .get::<usize>("execution_steps", &task.id)?
                .unwrap_or(0);
            if step >= step_limit {
                break;
            }
            if delegated {
                self.consume_subagent_inputs(&task.id, messages)?;
            }
            self.store.put("execution_steps", &task.id, &(step + 1))?;
            self.check_plan_editor(&task.id)?;
            self.store.check_plan_worker(&task.id)?;
            if cancel.is_cancelled() {
                bail!("Task cancelled");
            }
            let catalog = self.exposed_tools(&task.id)?;
            let operational = self.operational_context(task)?;
            let reserved = operational.as_ref().map_or(0, |message| {
                crate::active_context::estimated_tokens(std::slice::from_ref(message), &[])
            });
            self.compact_active_context(task, model, messages, &catalog, reserved, cancel)
                .await?;
            let recoverable = !self
                .processes
                .list()
                .await
                .iter()
                .any(|p| p.owner == task.id && p.status == "running");
            self.recovery_boundary(&task.id, messages, recoverable)?;
            self.execution_stage(&task.id, "inferring")?;
            let inference_started = std::time::Instant::now();
            let mut input = messages.clone();
            if let Some(operational) = operational {
                // Keep a stable request prefix, with only one transient snapshot.
                // Never append repeated copies to durable history or checkpoints.
                input.push(operational);
            }
            let input_tokens = self
                .context_tokens(&task.id, model, messages, &catalog)?
                .saturating_add(reserved);
            let mut request = model.settings.request(input);
            let budget = crate::context_budget::ContextBudget::new(&model.settings);
            let available = budget.output_available(input_tokens);
            if available < 1024 {
                bail!("Context leaves insufficient output space; adjust coding context capacity");
            }
            request.max_output_tokens = request.max_output_tokens.min(available as u32);
            request.tools = catalog;
            let response = if delegated {
                let _permit = tokio::select! { biased; _=cancel.cancelled()=>bail!("Task cancelled"), permit=self.subagents.inference.acquire()=>permit? };

                // Task attempts own the retry budget; never multiply it by HTTP retries.
                let config = model.settings.provider_config();
                tokio::select! {
                    _ = cancel.cancelled() => bail!("Task cancelled"),
                    response = model.client.complete(&config, model.settings.project.as_deref(), &request) => response?,
                }
            } else {
                self.infer_with_retry(task, model, &request, cancel).await?
            };
            if cancel.is_cancelled() {
                bail!("Task cancelled");
            }
            self.check_plan_editor(&task.id)?;
            self.store.check_plan_worker(&task.id)?;
            self.acknowledge_repository_rules(&task.id, messages)?;
            self.anchor_usage(
                &task.id,
                model,
                messages,
                &request.tools,
                response.usage.input_tokens,
            )?;
            self.store.event(&task.id,"model_response",&json!({"turn_id":fenix_storage::id(),"step":step+1,"usage":response.usage,"finish_reason":response.finish_reason,"response_id":response.id,"elapsed_ms":inference_started.elapsed().as_millis()}))?;
            let mut encoded = serde_json::to_value(response.message)?;
            self.redactor.value(&mut encoded);
            let message: Message = serde_json::from_value(encoded)?;
            let text = message.text_content();
            let calls = message
                .content
                .iter()
                .filter_map(|b| match b {
                    ContentBlock::ToolCall {
                        id,
                        name,
                        arguments,
                    } => Some((id.clone(), name.clone(), arguments.clone())),
                    _ => None,
                })
                .collect::<Vec<_>>();
            if response.finish_reason == FinishReason::ContentFilter {
                bail!("The provider filtered the response; no proposed tool calls were executed");
            }
            if response.finish_reason == FinishReason::Length {
                self.store.event(
                    &task.id,
                    "truncated_response",
                    &json!({"message":message,"executed":false}),
                )?;
                self.recover_execution(
                    &task.id,
                    "output_length",
                    json!({"step":step+1,"output_ceiling":request.max_output_tokens}),
                )?;
                // Never execute partial calls or leave unmatched pairs in active context.
                let oversized_action = calls
                    .iter()
                    .any(|(_, _, args)| args.to_string().len() > 8000);
                let strategy = if oversized_action {
                    "An action payload was large. Split it into small edits; send only the next complete tool call."
                } else {
                    "No large action payload was observed; reasoning or response generation exhausted the output allowance. Stop speculative analysis. Choose one concrete executable probe or small implementation, then test its observed behavior before further reasoning."
                };
                self.store.event(&task.id, "recovery_strategy", &json!({"reason":"output_length","strategy":strategy,"classification":"observed_payload_heuristic_not_provider_reasoning_count"}))?;
                messages.push(Message::user(format!("The previous generation was NOT executed. {strategy} Preserve completed work and inspect existing evidence.")));
                continue;
            }
            if calls.is_empty() && self.processes.has_running(&task.id).await {
                self.store.event(
                    &task.id,
                    "premature_completion",
                    &json!({"message":message,"published":false}),
                )?;
                self.recover_execution(&task.id, "completion_with_running_process", json!({"processes":self.processes.list().await.into_iter().filter(|p| p.owner == task.id).collect::<Vec<_>>()}))?;
                if let Err(error) = self.processes.wait_foreground(&task.id, cancel).await
                    && error
                        .downcast_ref::<crate::processes::BusyProcess>()
                        .is_none()
                {
                    return Err(error);
                }
                messages.push(Message::user(format!("A completion was withheld because a foreground command was still running. No action was repeated. Inspect its output and status before finishing. Processes: {}", json!(self.processes.list().await.into_iter().filter(|p| p.owner == task.id).collect::<Vec<_>>()))));
                continue;
            }
            let handoff =
                calls.is_empty() && !delegated && self.store.has_active_delegation(&task.id)?;
            if calls.is_empty()
                && !delegated
                && !handoff
                && self
                    .store
                    .get::<bool>("explicit_execution_outcome", &task.id)?
                    .unwrap_or(false)
            {
                self.store.event(&task.id, "premature_completion", &json!({"message":message,"published":false,"reason":"missing_explicit_outcome"}))?;
                self.recover_execution(&task.id, "completion_without_outcome", json!({"instruction":"An execution task needs an explicit outcome, not a progress-only final answer."}))?;
                messages.push(Message::user("Your final response did not finish the execution task. Continue the remaining authorized work: inspect errors, repair actionable failures and run the requested checks. Do not merely restate pending work. When genuinely finished, call task_finish alone with completed and observed evidence; if no authorized path remains, use blocked/failed with a concrete blocker. Ask the owner only for a genuinely missing decision or authorization. Do not repeat completed effects."));
                continue;
            }
            self.recovery_boundary(&task.id, messages, false)?;
            if delegated {
                self.store
                    .event(&task.id, "subagent_message", &json!({"message":message}))?;
            } else {
                let message_id =
                    self.store
                        .publish_message(Some(&task.id), &message, true, "agent")?;
                self.progress.send_modify(|progress| {
                    if progress.task_id.as_deref() != Some(&task.id) {
                        return;
                    }
                    progress.phase = Phase::Saved;
                    progress.message_id = Some(message_id);
                    progress.preview.clear();
                });
                self.bus_wake.notify_waiters();
            }
            messages.push(message);
            if calls.is_empty() {
                if delegated {
                    self.store.save_subagent_context(&task.id, messages, 0)?;
                }
                if delegated && !self.store.pending_subagent_inputs(&task.id)?.is_empty() {
                    continue;
                }
                if text.trim().is_empty() {
                    bail!("The model returned neither text nor a tool action");
                }
                if self.processes.has_running(&task.id).await {
                    bail!(
                        "The model ended before verifying a running command; remaining managed processes were cancelled"
                    );
                }
                if delegated {
                    return Ok(
                        json!({"status":"blocked","summary":text,"blockers":"The subagent ended without an explicit subagent_finish outcome.","artifacts":"","evidence":format!("task_inspect task_id={}",task.id)}),
                    );
                }
                if handoff {
                    // Complete the conversational turn, not its independently tracked work.
                    self.store.put("execution_handoff", &task.id, &true)?;
                    self.store.publish_event(
                        Some(&task.id),
                        "task.delegated",
                        &json!({"task_id":task.id,"work_status":"in_progress"}),
                    )?;
                    return Ok(json!({"status":"delegated"}));
                }
                return Ok(json!({"status":"completed"}));
            }
            if calls.len() > 16 {
                bail!("Too many tool calls in one response");
            }
            let call_count = calls.len();
            let tool_byte_budget = budget.tool_bytes(call_count);
            self.execution_stage(&task.id, "executing_tools")?;
            let mut finished = None;
            // Independent, explicitly read-only calls can run as a batch.
            // Mixed/action responses retain their existing ordered execution.
            let read_batch =
                calls.len() > 1 && calls.iter().all(|(_, name, _)| tools::parallel_read(name));
            let mut read_results = if read_batch {
                for (call_id, name, args) in &calls {
                    self.check_plan_editor(&task.id)?;
                    self.store.check_plan_worker(&task.id)?;
                    if cancel.is_cancelled() {
                        bail!("Task cancelled");
                    }
                    self.report_progress(&task.id, Phase::Tool, Some(name));
                    self.store.event(
                        &task.id,
                        "tool_started",
                        &json!({"call_id":call_id,"tool":name,"arguments":args}),
                    )?;
                }
                let requests = calls.iter().map(|(call_id, name, args)| {
                    let denied = denials.contains(&json!({"tool":name,"arguments":args}));
                    let source = &source;
                    let settings = &model.settings;
                    async move {
                        let result = if denied {
                            Ok(json!({"denied":true,"executed":false,"previously_denied":true}))
                        } else {
                            self.execute_tool(&task.id, source, settings, name, args, cancel)
                                .await
                        };
                        // Persist each completed read immediately, even if a sibling
                        // is later cancelled. Transcript results remain in call order.
                        self.record_tool_outcome(&task.id, call_id, name, args, result)
                    }
                });
                let results = tokio::select! {
                    biased;
                    _ = cancel.cancelled() => bail!("Task cancelled"),
                    results = futures_util::future::try_join_all(requests) => results?,
                };
                Some(results.into_iter())
            } else {
                None
            };
            for (call_id, name, args) in calls {
                self.check_plan_editor(&task.id)?;
                self.store.check_plan_worker(&task.id)?;
                if cancel.is_cancelled() {
                    bail!("Task cancelled");
                }
                self.report_progress(&task.id, Phase::Tool, Some(&name));
                let action = json!({"tool":name,"arguments":args});
                if !read_batch {
                    self.store.event(
                        &task.id,
                        "tool_started",
                        &json!({"call_id":call_id,"tool":name,"arguments":args}),
                    )?;
                }
                let (output, is_error) = if let Some(results) = &mut read_results {
                    results.next().context("Missing read batch result")?
                } else {
                    let result: Result<Value> = async {
                    if delegated && !self.store.pending_subagent_inputs(&task.id)?.is_empty() {
                        return Ok(json!({"executed":false,"reason":"New supervisor input arrived; consume it before performing remaining actions."}));
                    }
                    self.authorize_project_tool(&task.id, &name, &args)?;
                    if let Some(receipt) = self.ensure_tool_loaded(&task.id, &name)? { return Ok(receipt); }
                    tools::validate_for(delegated, &name,&args)?;
                    self.authorize_project_tool(&task.id, &name, &args)?;
                    if name == "task_finish" {
                        if delegated { bail!("Subagents must use subagent_finish"); }
                        if call_count != 1 { bail!("Finish must be the only tool call in its response"); }
                        if self.processes.has_running(&task.id).await { bail!("Wait for foreground commands before finishing"); }
                        if args["summary"].as_str().is_none_or(|s| s.trim().is_empty()) { bail!("A concrete outcome summary is required"); }
                        let field = if args["status"] == "completed" { "evidence" } else { "blockers" };
                        if args[field].as_str().is_none_or(|s| s.trim().is_empty()) { bail!("An execution outcome requires {field}"); }
                        finished = Some(args.clone());
                        return Ok(json!({"recorded":true,"status":args["status"]}));
                    }
                    if name == "subagent_finish" {
                        if call_count != 1 { bail!("Finish must be the only tool call in its response"); }
                        if self.processes.has_running(&task.id).await { bail!("Inspect or cancel running commands before reporting an outcome"); }
                        if args["summary"].as_str().is_none_or(|s| s.trim().is_empty()) { bail!("A concrete outcome summary is required"); }
                        if self.store.review_for_worker(&task.id)?.is_some() && args["status"] == "completed" {
                            let report = serde_json::from_value(args["review"].clone()).context("Plan reviewers must return a structured review")?;
                            self.store.validate_review_report(&task.id, &report)?;
                        }
                        if self.store.completion_audit_for_worker(&task.id)?.is_some() && args["status"] == "completed" {
                            let report = serde_json::from_value(args["completion_audit"].clone()).context("Completion auditors must return a structured report")?;
                            self.store.validate_completion_report(&task.id, &report)?;
                            self.validate_completion_observations(&task.id, &report)?;
                        }
                        if self.task_profile(&task.id)? == "security" && self.store.review_for_worker(&task.id)?.is_none() { crate::security::validate_report(&args)?; }
                        finished = Some(args.clone());
                        return Ok(json!({"recorded":true}));
                    }
                    if name=="request_approval" {
                        let target=args["tool"].as_str().ok_or_else(||anyhow::anyhow!("Target tool required"))?;
                        tools::validate(target,&args["arguments"])?;
                        self.authorize_project_tool(&task.id, target, &args["arguments"])?;
                        let requested=json!({"tool":target,"arguments":args["arguments"]});
                        if denials.contains(&requested) {return Ok(json!({"approved":false,"previously_denied":true}));}
                        if grants.contains(&requested) {return Ok(json!({"approved":true,"single_use":true,"previously_granted":true}));}
                        let approved=self.approve(&task.id,requested.clone(),cancel).await?;
                        if approved {grants.push(requested);} else {denials.push(requested);}
                        self.store.put("task_grants", &task.id, &grants)?;
                        self.store.put("task_denials", &task.id, &denials)?;
                        return Ok(json!({"approved":approved,"single_use":true}));
                    }
                    if denials.contains(&action) {return Ok(json!({"denied":true,"executed":false,"previously_denied":true}));}
                    if matches!(name.as_str(), "library_export" | "document_create" | "document_edit" | "file_write" | "file_edit" | "file_patch") {
                        let workspace = self.task_workspace(&task.id, &model.settings.workspace)?;
                        let rules = self.pending_repository_rules(&task.id, &workspace, &name, &args)?;
                        if !rules.is_empty() { return Ok(json!({"executed":false,"repository_instructions":rules,"instruction":"Read this guidance before retrying; no mutation or approval consumption occurred."})); }
                    }
                    let requires=((matches!(name.as_str(),"command_run"|"command_input") || name == "integration_call" && args["action"] != "list") && model.settings.command_policy==ActionPolicy::Ask) || ((matches!(name.as_str(),"library_export" | "document_create"|"document_edit"|"file_write"|"file_edit"|"file_patch") || name == "file_history" && args["action"] == "restore" || name == "workspace_checkpoint" && args["action"] != "list" || name == "worktree_manage" && args["action"] == "integrate") && model.settings.file_write_policy==ActionPolicy::Ask);
                    let requires = requires || name == "connector_call" && self.connector_requires_approval(&model.settings, &args);
                    if requires {
                        if let Some(index)=grants.iter().position(|g|g==&action) {grants.remove(index);self.store.put("task_grants", &task.id, &grants)?;}
                        else if !self.approve(&task.id,action.clone(),cancel).await? {denials.push(action.clone());self.store.put("task_denials", &task.id, &denials)?;return Ok(json!({"denied":true,"executed":false,"instruction":"Owner denied this exact action. Do not retry or bypass it."}));}
                    }
                    self.report_progress(&task.id, Phase::Tool, Some(&name));
                    self.execute_tool(&task.id,&source,&model.settings,&name,&args,cancel).await
                    }.await;
                    self.record_tool_outcome(&task.id, &call_id, &name, &args, result)?
                };
                let browser_evidence = browser_progress.observe(&name, &args, &output, is_error);
                let progress = format!(
                    "{}:browser:{}",
                    self.processes.progress_signature(&task.id).await,
                    browser_progress.revision
                );
                let effect_completed = !is_error
                    && output["executed"] != false
                    && output["denied"] != true
                    && matches!(
                        name.as_str(),
                        "library_export"
                            | "document_create"
                            | "document_edit"
                            | "file_write"
                            | "file_edit"
                            | "file_patch"
                            | "command_run"
                            | "workspace_checkpoint"
                            | "integration_call"
                            | "command_input"
                    );
                let looping = name != "command_status"
                    && repetition.observe(
                        format!(
                            "{action}:{}",
                            browser_evidence.unwrap_or_else(|| output.to_string())
                        ),
                        progress,
                        effect_completed,
                    );
                let content =
                    crate::tool_receipt::bounded(output, tool_byte_budget, &task.id, &call_id);
                let tool_message = Message {
                    role: Role::Tool,
                    content: vec![ContentBlock::ToolResult {
                        tool_call_id: call_id,
                        name: Some(name.clone()),
                        content,
                        is_error,
                    }],
                    provider_data: None,
                };
                if !delegated {
                    self.store
                        .append(Some(&task.id), &tool_message, false, true, "tool")?;
                }
                messages.push(tool_message);
                if looping {
                    self.store.event(&task.id, "loop_detected", &json!({"tool":name,"action":action,"reason":"identical result without observed progress"}))?;
                    self.recover_execution(&task.id, "no_progress", json!({"tool":name,"action":action,"processes":self.processes.list().await.into_iter().filter(|p|p.owner==task.id).collect::<Vec<_>>() }))?;
                    messages.push(Message::user("Three identical observations produced no visible progress. Diagnose the cause before the next action: inspect a blocking process with command_status/command_output, load the exact tool schema after argument errors, retrieve task_inspect after missing context, or run one focused behavioral test. Choose a different evidence-backed strategy. Silence from a live process is not proof of failure. Do not replay completed effects."));
                }
                if delegated
                    && matches!(
                        name.as_str(),
                        "subagent_spawn" | "subagent_status" | "subagent_cancel" | "subagent_send"
                    )
                {
                    return Ok(
                        json!({"status":"blocked","summary":"Delegation request rejected by the engine.","blockers":"Only the master may manage subagents. Ask the master for assistance.","artifacts":"","evidence":format!("task_inspect task_id={}",task.id)}),
                    );
                }
            }
            self.store.put("execution_context", &task.id, messages)?;
            self.execution_stage(&task.id, "running")?;
            if delegated {
                self.store.save_subagent_context(&task.id, messages, 0)?;
            }
            if let Some(result) = finished {
                if !delegated {
                    let summary = Message::text(
                        Role::Assistant,
                        result["summary"].as_str().unwrap_or_default(),
                    );
                    self.store
                        .publish_message(Some(&task.id), &summary, true, "agent")?;
                    self.bus_wake.notify_waiters();
                }
                return Ok(result);
            }
        }
        bail!("Task reached the bounded model-call limit; review the trace before continuing")
    }

    async fn infer_with_retry(
        &self,
        task: &Task,
        model: &ModelConnection,
        request: &fenix_core::ChatRequest,
        cancel: &CancellationToken,
    ) -> Result<fenix_core::ChatResponse> {
        for attempt in 0..3 {
            let response_id = fenix_storage::id();
            self.progress.send_replace(Progress {
                task_id: Some(task.id.clone()),
                response_id: Some(response_id),
                phase: Phase::Generating,
                ..Progress::default()
            });
            let mut raw_preview = String::new();
            let mut saw_delta = false;
            let mut published = std::time::Instant::now() - Duration::from_secs(1);
            let mut preview = |delta: &str| {
                saw_delta = true;
                // Keep previews bounded; the provider client bounds the entire response.
                let available = 32_000usize.saturating_sub(raw_preview.len());
                let mut end = delta.len().min(available);
                while !delta.is_char_boundary(end) {
                    end -= 1;
                }
                raw_preview.push_str(&delta[..end]);
                if published.elapsed() >= Duration::from_millis(100) {
                    let text = self.redactor.preview(&raw_preview);
                    self.progress.send_modify(|progress| {
                        if progress.task_id.as_deref() == Some(&task.id) {
                            progress.preview = text;
                        }
                    });
                    published = std::time::Instant::now();
                }
            };
            let config = model.settings.provider_config();
            let result = tokio::select! {
                _ = cancel.cancelled() => bail!("Task cancelled"),
                result = model.client.complete_with_preview(
                    &config, model.settings.project.as_deref(), request, &mut preview,
                ) => result,
            };
            match result {
                Ok(response) => return Ok(response),
                Err(error) if error.is_retryable() && attempt < 2 && !saw_delta => {
                    self.report_progress(&task.id, Phase::Retrying, None);
                    self.store.event(
                        &task.id,
                        "model_retry",
                        &json!({"attempt":attempt+1,"error":error.to_string()}),
                    )?;
                    let delay = match error {
                        fenix_providers::ProviderError::RateLimited {
                            retry_after: Some(delay),
                        } => delay.min(Duration::from_secs(30)),
                        _ => Duration::from_secs(1 << attempt),
                    };
                    tokio::select! {_=cancel.cancelled()=>bail!("Task cancelled"),_=tokio::time::sleep(delay)=>{}}
                }
                Err(error) => return Err(error.into()),
            }
        }
        unreachable!("bounded retry loop always returns")
    }
}
