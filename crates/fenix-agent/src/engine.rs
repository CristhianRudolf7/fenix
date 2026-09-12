use crate::{
    Runtime,
    config::{ActionPolicy, ModelConnection},
    permissions::ActiveTask,
    redaction::truncate,
    tools,
};
use anyhow::{Result, bail};
use fenix_core::{ContentBlock, FinishReason, Message, Role};
use fenix_storage::Task;
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio_util::sync::CancellationToken;

const MAX_MODEL_STEPS: usize = 32;

impl Runtime {
    pub(crate) async fn worker(self: Arc<Self>) {
        loop {
            if self.stop.is_cancelled() {
                break;
            }
            let work = {
                let _lock = self.configure_lock.lock().await;
                match self.store.claim_next() {
                    Ok(Some(task)) => {
                        let cancel = self.stop.child_token();
                        *self.active.lock().await = Some(ActiveTask {
                            task_id: task.id.clone(),
                            cancel: cancel.clone(),
                        });
                        Some((task, cancel))
                    }
                    Ok(None) => None,
                    Err(error) => {
                        eprintln!(
                            "Task queue error: {}",
                            self.redactor.text(&error.to_string())
                        );
                        None
                    }
                }
            };
            if let Some((task, cancel)) = work {
                let result = tokio::select! {
                    result=self.run_task(&task,&cancel)=>result,
                    _=cancel.cancelled()=>Err(anyhow::anyhow!("Task cancelled")),
                    _=tokio::time::sleep(Duration::from_secs(3600))=>Err(anyhow::anyhow!("Task exceeded the one-hour execution limit")),
                };
                self.processes.cancel_task(&task.id).await;
                *self.pending.lock().await = None;
                let (status, error) = match result {
                    Ok(()) => ("completed", None),
                    Err(error) => {
                        let status = if self.stop.is_cancelled() {
                            "interrupted"
                        } else if cancel.is_cancelled() {
                            "cancelled"
                        } else {
                            "failed"
                        };
                        let error = self.redactor.text(&error.to_string());
                        // Do not send incomplete tool-call sequences to a future model turn.
                        let _ = self.store.discard_context(&task.id);
                        let notice = format!("Tarea {} · {}\n{}", task.id, status, error);
                        let _ = self.notify(Some(&task.id), &notice);
                        let _=self.store.append(Some(&task.id),&Message::user(format!("Previous request: {}\nOutcome: {notice}. No automatic resumption. Inspect saved task events before repeating any effect.",task.request)),false,true,"runtime");
                        (status, Some(error))
                    }
                };
                let _ = self
                    .store
                    .set_task_status(&task.id, status, error.as_deref());
                let _ = self.store.event(
                    &task.id,
                    "task_finished",
                    &json!({"status":status,"error":error}),
                );
                *self.active.lock().await = None;
            } else {
                tokio::select! {_=self.stop.cancelled()=>break,_=self.wake.notified()=>{},_=tokio::time::sleep(Duration::from_secs(1))=>{}}
            }
        }
    }

    async fn run_task(self: &Arc<Self>, task: &Task, cancel: &CancellationToken) -> Result<()> {
        let model = self
            .model
            .read()
            .await
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Model is not configured"))?;
        self.store.event(&task.id,"task_started",&json!({"session_id":task.session_id,"objective":task.request,"provider":model.settings.provider,"model":model.settings.model,"reasoning_effort":model.settings.reasoning_effort,"workspace":model.settings.workspace}))?;
        let mut messages = self.build_context(task, &model).await?;
        let source = self
            .store
            .context(0)?
            .into_iter()
            .find(|m| m.task_id.as_deref() == Some(&task.id) && m.role == "user")
            .map(|m| m.id)
            .unwrap_or_else(|| task.id.clone());
        let mut repetition = HashMap::<String, usize>::new();
        let mut grants = Vec::<Value>::new();
        let mut denials = Vec::<Value>::new();
        for step in 0..MAX_MODEL_STEPS {
            if cancel.is_cancelled() {
                bail!("Task cancelled");
            }
            let mut request = model.settings.request(messages.clone());
            request.tools = tools::catalog();
            if serde_json::to_vec(&request)?.len() > 512_000 {
                bail!(
                    "This task's active context is too large. Original history and tool evidence remain available; split the next request into smaller steps"
                );
            }
            let response = self
                .infer_with_retry(task, &model, &request, cancel)
                .await?;
            self.store.event(&task.id,"model_response",&json!({"turn_id":fenix_storage::id(),"step":step+1,"usage":response.usage,"finish_reason":response.finish_reason,"response_id":response.id}))?;
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
            self.store
                .append(Some(&task.id), &message, !text.is_empty(), true, "agent")?;
            if !text.is_empty()
                && self
                    .telegram_settings()?
                    .is_some_and(|s| s.enabled && s.owner_id.is_some())
            {
                self.store.queue_delivery(&text)?;
            }
            messages.push(message);
            if matches!(
                response.finish_reason,
                FinishReason::Length | FinishReason::ContentFilter
            ) {
                bail!(
                    "The model response was truncated or filtered; task completion has not been verified"
                );
            }
            if calls.is_empty() {
                if text.trim().is_empty() {
                    bail!("The model returned neither text nor a tool action");
                }
                if self.processes.has_running(&task.id).await {
                    bail!(
                        "The model ended before verifying a running command; remaining managed processes were cancelled"
                    );
                }
                return Ok(());
            }
            if calls.len() > 16 {
                bail!("Too many tool calls in one response");
            }
            for (call_id, name, args) in calls {
                let action = json!({"tool":name,"arguments":args});
                self.store.event(
                    &task.id,
                    "tool_started",
                    &json!({"call_id":call_id,"tool":name,"arguments":args}),
                )?;
                let result:Result<Value>=async {
                    tools::validate(&name,&args)?;
                    if name=="request_approval" {
                        let target=args["tool"].as_str().ok_or_else(||anyhow::anyhow!("Target tool required"))?;
                        tools::validate(target,&args["arguments"])?;
                        let requested=json!({"tool":target,"arguments":args["arguments"]});
                        if denials.contains(&requested) {return Ok(json!({"approved":false,"previously_denied":true}));}
                        let approved=self.approve(&task.id,requested.clone(),cancel).await?;
                        if approved {grants.push(requested);} else {denials.push(requested);}
                        return Ok(json!({"approved":approved,"single_use":true}));
                    }
                    if denials.contains(&action) {return Ok(json!({"denied":true,"executed":false,"previously_denied":true}));}
                    let requires=(name=="command_run" && model.settings.command_policy==ActionPolicy::Ask) || (matches!(name.as_str(),"file_write"|"file_edit") && model.settings.file_write_policy==ActionPolicy::Ask);
                    if requires {
                        if let Some(index)=grants.iter().position(|g|g==&action) {grants.remove(index);}
                        else if !self.approve(&task.id,action.clone(),cancel).await? {denials.push(action.clone());return Ok(json!({"denied":true,"executed":false,"instruction":"Owner denied this exact action. Do not retry or bypass it."}));}
                    }
                    self.execute_tool(&task.id,&source,&model.settings,&name,&args,cancel).await
                }.await;
                let (mut output, is_error) = match result {
                    Ok(value) => (value, false),
                    Err(error) => (
                        json!({"error":self.redactor.text(&error.to_string())}),
                        true,
                    ),
                };
                self.redactor.value(&mut output);
                self.store.event(
                    &task.id,
                    "tool_result",
                    &json!({"call_id":call_id,"tool":name,"result":output,"is_error":is_error}),
                )?;
                // command_status intentionally waits; the managed-process timeout is its loop bound.
                if name != "command_status" {
                    let fingerprint = format!("{action}:{output}");
                    let count = repetition.entry(fingerprint).or_default();
                    *count += 1;
                    if *count >= 3 {
                        bail!(
                            "Repeated tool call produced the same result three times; stopped to avoid a loop"
                        );
                    }
                }
                let serialized = output.to_string();
                let content = if serialized.len() > 16_000 {
                    json!({"truncated":true,"preview":truncate(&serialized,12_000),"task_id":task.id,"call_id":call_id,"retrieve":"Use task_inspect with task_id, call_id, offset and length to read the saved output in pages."})
                } else {
                    output
                };
                let tool_message = Message {
                    role: Role::Tool,
                    content: vec![ContentBlock::ToolResult {
                        tool_call_id: call_id,
                        name: Some(name),
                        content,
                        is_error,
                    }],
                    provider_data: None,
                };
                self.store
                    .append(Some(&task.id), &tool_message, false, true, "tool")?;
                messages.push(tool_message);
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
            match model
                .client
                .complete(
                    &model.settings.provider_config(),
                    model.settings.project.as_deref(),
                    request,
                )
                .await
            {
                Ok(response) => return Ok(response),
                Err(error) if error.is_retryable() && attempt < 2 => {
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
