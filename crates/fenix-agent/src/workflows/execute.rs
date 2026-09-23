use super::*;
use std::{collections::BTreeMap, sync::Arc, time::Duration};
use tokio_util::sync::CancellationToken;

impl Runtime {
    pub(crate) async fn workflow_worker(self: Arc<Self>) {
        let mut jobs = tokio::task::JoinSet::new();
        loop {
            if self.stop.is_cancelled() {
                break;
            }
            if let Ok(runs) = self
                .store
                .records_by_status::<Run>("workflow_runs", "queued", 32)
            {
                for run in runs.into_iter().filter(|r| r.status == "queued") {
                    let mut active = self.workflow_active.lock().await;
                    if active.len() >= 16 {
                        break;
                    }
                    if active.contains_key(&run.id) {
                        continue;
                    }
                    let cancel = self.stop.child_token();
                    active.insert(run.id.clone(), cancel.clone());
                    drop(active);
                    let runtime = self.clone();
                    jobs.spawn(async move {
                        let id = run.id.clone();
                        if let Err(error) = runtime.run_workflow(run, cancel).await {
                            eprintln!(
                                "Workflow error: {}",
                                runtime.redactor.text(&error.to_string())
                            );
                        }
                        runtime.workflow_active.lock().await.remove(&id);
                    });
                }
            }
            tokio::select! { _=self.stop.cancelled()=>break,_=self.workflow_wake.notified()=>{},_=tokio::time::sleep(Duration::from_millis(500))=>{},Some(_)=jobs.join_next(),if !jobs.is_empty()=>{} }
        }
        self.workflow_active
            .lock()
            .await
            .values()
            .for_each(CancellationToken::cancel);
        while jobs.join_next().await.is_some() {}
    }

    pub async fn run_workflow(
        self: &Arc<Self>,
        mut run: Run,
        cancel: CancellationToken,
    ) -> Result<()> {
        if self.workflow_run(&run.id)?.status == "cancelled" {
            return Ok(());
        }
        run.status = "running".into();
        self.persist_workflow_run(&run)?;
        self.store
            .set_task_status(&run.task_id, "workflow_running", None)?;
        let result = self.run_workflow_steps(&mut run, &cancel).await;
        if let Err(error) = result {
            let message = self.redactor.text(&error.to_string());
            run.status = if cancel.is_cancelled() {
                if self.stop.is_cancelled() {
                    "interrupted"
                } else {
                    "cancelled"
                }
            } else {
                "failed"
            }
            .into();
            run.error = Some(message.clone());
            for state in run.nodes.values_mut().filter(|n| n.status == "running") {
                state.status = run.status.clone();
                state.error = Some(message.clone());
                state.finished_at = Some(fenix_storage::now());
            }
        } else if run.status == "running" {
            run.status = "completed".into();
        }
        self.persist_workflow_run(&run)?;
        self.store
            .set_task_status(&run.task_id, &run.status, run.error.as_deref())?;
        self.clear_pending(&run.task_id).await;
        if cancel.is_cancelled() {
            self.processes.cancel_task(&run.task_id).await;
        }
        let message = format!(
            "Flujo «{}»: {}.{}",
            run.workflow.spec.name,
            run.status,
            run.error
                .as_ref()
                .map(|e| format!(" {e}"))
                .unwrap_or_default()
        );
        self.notify(Some(&run.task_id), &message)?;
        Ok(())
    }

    async fn run_workflow_steps(
        self: &Arc<Self>,
        run: &mut Run,
        cancel: &CancellationToken,
    ) -> Result<()> {
        let settings = self.settings()?.context("Configure a workspace first")?;
        let mut outputs: BTreeMap<String, Value> = run
            .nodes
            .iter()
            .filter(|(_, n)| n.status == "completed")
            .map(|(id, n)| (id.clone(), n.output.clone()))
            .collect();
        loop {
            ensure!(!cancel.is_cancelled(), "Execution cancelled");
            let node =
                run.workflow
                    .spec
                    .nodes
                    .iter()
                    .find(|n| {
                        matches!(
                            run.nodes[&n.id].status.as_str(),
                            "pending" | "needs_confirmation"
                        ) && n.depends_on.iter().all(|d| {
                            matches!(run.nodes[d].status.as_str(), "completed" | "skipped")
                        })
                    })
                    .cloned();
            let Some(node) = node else {
                ensure!(
                    run.nodes
                        .values()
                        .all(|n| matches!(n.status.as_str(), "completed" | "skipped")),
                    "No runnable step; inspect dependencies"
                );
                break;
            };
            let skipped = node
                .depends_on
                .iter()
                .any(|d| run.nodes[d].status == "skipped");
            let allowed = if let Some(predicate) = &node.when {
                model::condition(predicate, &outputs)?
            } else {
                !skipped
            };
            if !allowed {
                run.nodes.get_mut(&node.id).unwrap().status = "skipped".into();
                self.persist_workflow_run(run)?;
                continue;
            }
            if run.test_mode && !self.workflow_node_read_only(&node) {
                run.status = "paused".into();
                run.error=Some("Preview stopped before a step that may create effects. Continue explicitly to execute it.".into());
                break;
            }
            if run.nodes[&node.id].status == "needs_confirmation" {
                run.status = "waiting".into();
                self.persist_workflow_run(run)?;
                let approved=self.approve(&run.task_id,json!({"workflow":run.workflow.id,"step":node.id,"reason":"The previous result is uncertain. Inspect the destination first. Repeating this step can duplicate an external action."}),cancel).await?;
                ensure!(
                    approved,
                    "Retry was declined; completed steps were preserved"
                );
                run.status = "running".into();
            }
            {
                let state = run.nodes.get_mut(&node.id).unwrap();
                state.status = "running".into();
                state.started_at = Some(fenix_storage::now());
                state.error = None;
            }
            self.persist_workflow_run(run)?;
            let args = model::resolve(&run_tokens(&node.arguments, run), &outputs)?;
            let attempts = if self.workflow_node_read_only(&node) {
                node.attempts
            } else {
                1
            };
            let mut result = Err(anyhow::anyhow!("Step did not run"));
            for attempt in 0..attempts {
                run.nodes.get_mut(&node.id).unwrap().attempts += 1;
                self.persist_workflow_run(run)?;
                result = self
                    .workflow_step(run, &node, &args, &outputs, &settings, cancel)
                    .await;
                if result.is_ok() || cancel.is_cancelled() {
                    break;
                }
                if attempt + 1 < attempts {
                    tokio::select! {_=cancel.cancelled()=>anyhow::bail!("Execution cancelled"),_=tokio::time::sleep(Duration::from_secs(u64::from(attempt)+1))=>{}}
                }
            }
            let mut output = result?;
            self.redactor.value(&mut output);
            {
                let state = run.nodes.get_mut(&node.id).unwrap();
                state.status = "completed".into();
                state.output = output.clone();
                state.finished_at = Some(fenix_storage::now());
            }
            outputs.insert(node.id, output);
            self.persist_workflow_run(run)?;
        }
        Ok(())
    }

    async fn workflow_step(
        self: &Arc<Self>,
        run: &mut Run,
        node: &Node,
        args: &Value,
        outputs: &BTreeMap<String, Value>,
        settings: &crate::config::Settings,
        cancel: &CancellationToken,
    ) -> Result<Value> {
        match node.kind.as_str() {
            "condition" => Ok(
                json!({"matches":model::condition(&serde_json::from_value(args.clone())?,outputs)?}),
            ),
            "notify" => {
                let text = args["text"]
                    .as_str()
                    .context("Notification text required")?;
                self.notify(Some(&run.task_id), text)?;
                Ok(json!({"published":true,"delivery":"queued"}))
            }
            "wait" => {
                run.status = "waiting".into();
                self.persist_workflow_run(run)?;
                let answer = self
                    .ask(
                        &run.task_id,
                        "clarification",
                        args["question"].as_str().context("Question required")?,
                        None,
                        cancel,
                    )
                    .await?;
                run.status = "running".into();
                Ok(json!({"answer":answer}))
            }
            "prompt" => {
                let prompt = args["prompt"]
                    .as_str()
                    .context("Agent instructions required")?;
                let binding = self.store.task_binding(&run.task_id)?;
                let child = self
                    .store
                    .enqueue_bound(prompt, "workflow_prompt", None, &binding)?;
                self.store.put(
                    "workflow_children",
                    &format!("{}:{}", run.id, node.id),
                    &child.id,
                )?;
                if let Some((channel, address)) = self.store.task_destination(&run.task_id)? {
                    self.store
                        .set_task_destination(&child.id, &channel, address.as_deref())?;
                }
                self.wake.notify_one();
                loop {
                    let task = self.store.task(&child.id)?;
                    match task.status.as_str() {
                        "completed" => {
                            return Ok(
                                json!({"task_id":child.id,"text":self.store.task_output(&child.id)?}),
                            );
                        }
                        "failed" | "cancelled" | "interrupted" | "blocked" => anyhow::bail!(
                            "Agent step {}: {}",
                            task.status,
                            task.error.unwrap_or_default()
                        ),
                        _ => {}
                    }
                    tokio::select! {_=cancel.cancelled()=>{self.store.set_task_status(&child.id,"cancelled",None)?;if let Some(active)=self.active.lock().await.get(&child.id){active.cancel.cancel();}anyhow::bail!("Agent step cancelled");},_=tokio::time::sleep(Duration::from_millis(500))=>{}}
                }
            }
            "browser" => {
                let mut local = outputs.clone();
                let mut results = run.nodes[&node.id]
                    .output
                    .as_object()
                    .cloned()
                    .unwrap_or_default();
                // A browser workflow is an explicit set of ordinary DOM tools; no recorded coordinates or arbitrary JS.
                for action in &node.steps {
                    if let Some(output) = results.get(&action.id) {
                        local.insert(action.id.clone(), output.clone());
                        continue;
                    }
                    let args = model::resolve(&run_tokens(&action.arguments, run), &local)?;
                    self.store.event(&run.task_id,"workflow.browser.step",&json!({"run_id":run.id,"node":node.id,"action":action.id,"tool":action.tool}))?;
                    let output = self
                        .workflow_tool(&run.task_id, &action.tool, &args, settings, cancel)
                        .await?;
                    local.insert(action.id.clone(), output.clone());
                    results.insert(action.id.clone(), output);
                    run.nodes.get_mut(&node.id).unwrap().output = Value::Object(results.clone());
                    self.persist_workflow_run(run)?;
                }
                Ok(Value::Object(results))
            }
            "script" => {
                self.workflow_tool(&run.task_id, "command_run", args, settings, cancel)
                    .await
            }
            "connector" => {
                let mut input = args.get("arguments").cloned().unwrap_or(json!({}));
                if matches!(
                    args["operation"].as_str(),
                    Some("drive_export" | "drive_download" | "file_download")
                ) && input.get("destination").is_none()
                {
                    input["destination"] = json!(format!(
                        "automations/{}/{}/{}.{}",
                        run.workflow.id,
                        run.id,
                        node.id,
                        if args["operation"] == "drive_export"
                            && input["mime_type"].as_str().unwrap_or("application/pdf")
                                == "application/pdf"
                        {
                            "pdf"
                        } else {
                            "bin"
                        }
                    ));
                }
                self.workflow_tool(&run.task_id,"connector_call",&json!({"action":"invoke","connection":args["connection"],"operation":args["operation"],"arguments":input}),settings,cancel).await
            }
            "tool" => {
                self.workflow_tool(
                    &run.task_id,
                    node.tool.as_deref().context("Tool required")?,
                    args,
                    settings,
                    cancel,
                )
                .await
            }
            _ => anyhow::bail!("Unknown workflow step"),
        }
    }

    pub(crate) async fn workflow_tool(
        self: &Arc<Self>,
        task: &str,
        name: &str,
        args: &Value,
        settings: &crate::config::Settings,
        cancel: &CancellationToken,
    ) -> Result<Value> {
        ensure!(
            model::allowed_tool(name),
            "Tool is unavailable in workflows"
        );
        ensure!(
            self.task_tool_catalog(task)?.iter().any(|t| t.name == name),
            "Tool is unavailable in this task scope"
        );
        crate::tools::validate(name, args)?;
        let effect = matches!(
            name,
            "command_run"
                | "connector_call"
                | "file_write"
                | "file_edit"
                | "file_patch"
                | "document_create"
                | "document_edit"
                | "library_export"
        );
        let policy = if matches!(name, "command_run" | "connector_call") {
            &settings.command_policy
        } else {
            &settings.file_write_policy
        };
        let action = json!({"tool":name,"arguments":args});
        let mut denials = self
            .store
            .get::<Vec<Value>>("task_denials", task)?
            .unwrap_or_default();
        ensure!(
            !denials.contains(&action),
            "This exact action was previously declined"
        );
        if (name == "connector_call" && self.connector_requires_approval(settings, args))
            || (name != "connector_call" && effect && *policy == crate::config::ActionPolicy::Ask)
        {
            let mut grants = self
                .store
                .get::<Vec<Value>>("task_grants", task)?
                .unwrap_or_default();
            if let Some(index) = grants.iter().position(|grant| grant == &action) {
                grants.remove(index);
                self.store.put("task_grants", task, &grants)?;
            } else if !self.approve(task, action.clone(), cancel).await? {
                denials.push(action);
                self.store.put("task_denials", task, &denials)?;
                anyhow::bail!("Action was declined");
            }
        }
        self.store.event(
            task,
            "workflow.tool.started",
            &json!({"tool":name,"arguments":args}),
        )?;
        // Box breaks the indirect dispatch cycle (workflow tools also live in the tool registry).
        let mut value =
            Box::pin(self.execute_tool(task, "workflow", settings, name, args, cancel)).await?;
        if let Some(process) = value["process_id"].as_str().map(str::to_owned) {
            loop {
                if value["status"] != "running" && !value["exit_code"].is_null() {
                    break;
                }
                ensure!(!cancel.is_cancelled(), "Command cancelled");
                value = self
                    .processes
                    .status(task, task, &process, &json!({"wait_ms":1000,"limit":12000}))
                    .await?;
                if matches!(
                    value["status"].as_str(),
                    Some("completed" | "failed" | "cancelled" | "timed_out" | "exited")
                ) {
                    break;
                }
            }
            ensure!(
                value["exit_code"] == 0,
                "Command did not finish successfully"
            );
        }
        ensure!(
            value["error"].is_null()
                && value["status"] != "failed"
                && value["denied"] != true
                && value["executed"] != false
                && value["ok"] != false
                && value["matched"] != false
                && value["passed"] != false,
            "Tool did not confirm successful completion"
        );
        self.redactor.value(&mut value);
        self.store.event(
            task,
            "workflow.tool.completed",
            &json!({"tool":name,"output":value}),
        )?;
        Ok(value)
    }
}

fn run_tokens(value: &Value, run: &Run) -> Value {
    match value {
        Value::String(text) => json!(
            text.replace("{{run_id}}", &run.id)
                .replace("{{workflow_id}}", &run.workflow.id)
        ),
        Value::Array(items) => Value::Array(items.iter().map(|v| run_tokens(v, run)).collect()),
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(k, v)| (k.clone(), run_tokens(v, run)))
                .collect(),
        ),
        _ => value.clone(),
    }
}
