//! Durable, versioned workflows executed outside the conversational model loop.
mod execute;
pub(crate) mod model;
use crate::Runtime;
use anyhow::{Context, Result, ensure};
pub use model::{Action, Node, NodeRun, Predicate, Run, Workflow, WorkflowInput};
use serde_json::{Value, json};

impl Runtime {
    pub(crate) fn workflow_node_read_only(&self, node: &Node) -> bool {
        if node.kind == "connector" {
            return self
                .connectors
                .effect(
                    node.arguments["connection"].as_str().unwrap_or_default(),
                    node.arguments["operation"].as_str().unwrap_or_default(),
                    &node.arguments["arguments"],
                )
                .is_ok_and(|e| e == "read");
        }
        model::read_only(node)
    }
    pub fn workflows(&self) -> Result<Vec<Workflow>> {
        self.store.records("workflows")
    }
    pub fn workflow(&self, id: &str) -> Result<Workflow> {
        self.store
            .get("workflows", id)?
            .context("Workflow not found in this account")
    }
    pub fn workflow_run(&self, id: &str) -> Result<Run> {
        self.store
            .get("workflow_runs", id)?
            .context("Execution not found in this account")
    }
    pub fn workflow_runs(&self, id: Option<&str>) -> Result<Vec<Run>> {
        Ok(self
            .store
            .records::<Run>("workflow_runs")?
            .into_iter()
            .filter(|r| id.is_none_or(|id| r.workflow.id == id))
            .take(100)
            .collect())
    }
    pub async fn save_workflow(
        &self,
        id: Option<&str>,
        version: Option<u64>,
        spec: WorkflowInput,
    ) -> Result<Workflow> {
        spec.validate()?;
        if let Some(project) = &spec.project_id {
            self.store.project(project)?;
        }
        let _lock = self.workflow_lock.lock().await;
        let previous = id.map(|id| self.workflow(id)).transpose()?;
        if let Some(old) = &previous {
            ensure!(
                version == Some(old.version),
                "This flow has changed. Reload it before saving."
            );
        }
        let flow = Workflow {
            id: id.map(str::to_owned).unwrap_or_else(fenix_storage::id),
            version: previous.map_or(1, |w| w.version + 1),
            spec,
            updated_at: fenix_storage::now(),
        };
        self.store.put("workflows", &flow.id, &flow)?;
        self.store.put(
            "workflow_versions",
            &format!("{}:{}", flow.id, flow.version),
            &flow,
        )?;
        self.store.publish_event(
            None,
            "workflow.saved",
            &json!({"id":flow.id,"version":flow.version}),
        )?;
        self.bus_wake.notify_waiters();
        Ok(flow)
    }
    pub async fn enqueue_workflow(
        &self,
        id: &str,
        test_mode: bool,
        schedule_id: Option<&str>,
    ) -> Result<Run> {
        let _lock = self.workflow_lock.lock().await;
        let flow = self.workflow(id)?;
        flow.spec.validate()?;
        let previous = self.store.records::<Run>("workflow_runs")?;
        ensure!(
            !previous.iter().any(|r| r.workflow.id == id
                && matches!(r.status.as_str(), "queued" | "running" | "waiting")),
            "This workflow already has a pending execution"
        );
        ensure!(
            !previous.iter().any(|r| r.workflow.id == id
                && matches!(r.status.as_str(), "failed" | "interrupted")
                && r.workflow
                    .spec
                    .nodes
                    .iter()
                    .any(|n| !self.workflow_node_read_only(n)
                        && r.nodes.get(&n.id).is_some_and(|s| matches!(
                            s.status.as_str(),
                            "failed" | "interrupted" | "running"
                        )))),
            "A previous execution has an uncertain effect. Inspect and continue or explicitly cancel that execution before starting another."
        );
        let settings = self.settings()?.context("Configure a workspace first")?;
        let project = flow
            .spec
            .project_id
            .as_deref()
            .map(|id| self.store.project(id))
            .transpose()?;
        let binding = fenix_storage::TaskBinding {
            bound: true,
            project_id: flow.spec.project_id.clone(),
            read_project_ids: flow.spec.project_id.iter().cloned().collect(),
            mode: if project.is_some() {
                "project"
            } else {
                "general"
            }
            .into(),
            workspace: project
                .as_ref()
                .and_then(|p| p.workspace.clone())
                .or(Some(settings.workspace.to_string_lossy().into_owned())),
            context_version: project.as_ref().map_or(0, |p| p.context_version),
            ..Default::default()
        };
        let task = self.store.enqueue_bound(
            &format!("Workflow: {} (version {})", flow.spec.name, flow.version),
            "workflow",
            schedule_id,
            &binding,
        )?;
        self.store.set_task_destination(&task.id, "web", None)?;
        let run = Run {
            id: fenix_storage::id(),
            task_id: task.id,
            nodes: flow
                .spec
                .nodes
                .iter()
                .map(|n| (n.id.clone(), NodeRun::default()))
                .collect(),
            workflow: flow,
            status: "queued".into(),
            created_at: fenix_storage::now(),
            updated_at: fenix_storage::now(),
            test_mode,
            error: None,
        };
        self.persist_workflow_run(&run)?;
        self.workflow_wake.notify_one();
        Ok(run)
    }
    pub async fn cancel_workflow(&self, id: &str) -> Result<Run> {
        if let Some(cancel) = self.workflow_active.lock().await.get(id) {
            cancel.cancel();
        }
        let mut run = self.workflow_run(id)?;
        run.status = "cancelled".into();
        self.persist_workflow_run(&run)?;
        self.store
            .set_task_status(&run.task_id, "cancelled", None)?;
        self.processes.cancel_task(&run.task_id).await;
        Ok(run)
    }
    pub async fn resume_workflow(&self, id: &str) -> Result<Run> {
        let _lock = self.workflow_lock.lock().await;
        ensure!(
            !self.workflow_active.lock().await.contains_key(id),
            "This execution is still active"
        );
        let mut run = self.workflow_run(id)?;
        ensure!(
            matches!(run.status.as_str(), "failed" | "interrupted" | "paused"),
            "Only paused, failed or interrupted executions can continue"
        );
        ensure!(
            !self
                .store
                .records::<Run>("workflow_runs")?
                .iter()
                .any(|other| other.id != run.id
                    && other.workflow.id == run.workflow.id
                    && matches!(other.status.as_str(), "queued" | "running" | "waiting")),
            "Another execution of this workflow is already pending"
        );
        // Unknown writes require a textual owner decision when the worker resumes.
        for node in &run.workflow.spec.nodes {
            let state = run
                .nodes
                .get_mut(&node.id)
                .context("Missing step checkpoint")?;
            if matches!(state.status.as_str(), "running" | "failed" | "interrupted") {
                state.status = if self.workflow_node_read_only(node) {
                    "pending"
                } else {
                    "needs_confirmation"
                }
                .into();
                state.error = None;
            }
        }
        run.status = "queued".into();
        run.test_mode = false;
        run.error = None;
        self.store
            .set_task_status(&run.task_id, "workflow_queued", None)?;
        self.persist_workflow_run(&run)?;
        self.workflow_wake.notify_one();
        Ok(run)
    }
    pub(crate) fn persist_workflow_run(&self, run: &Run) -> Result<()> {
        let mut run = run.clone();
        run.updated_at = fenix_storage::now();
        self.store.put("workflow_runs", &run.id, &run)?;
        self.store.publish_event(Some(&run.task_id),"workflow.progress",&json!({"run_id":run.id,"workflow_id":run.workflow.id,"version":run.workflow.version,"status":run.status,"steps":run.nodes.iter().map(|(id,n)|json!({"id":id,"status":n.status})).collect::<Vec<_>>()}))?;
        self.bus_wake.notify_waiters();
        Ok(())
    }
    pub(crate) fn recover_workflow_runs(&self) -> Result<()> {
        for mut run in self.store.records::<Run>("workflow_runs")? {
            if matches!(run.status.as_str(), "running" | "waiting") {
                run.status = "interrupted".into();
                run.error=Some("Service stopped. Inspect the last step before continuing; external effects were not replayed.".into());
                for state in run.nodes.values_mut() {
                    if state.status == "running" {
                        state.status = "interrupted".into();
                    }
                }
                self.store
                    .set_task_status(&run.task_id, "interrupted", run.error.as_deref())?;
                self.persist_workflow_run(&run)?;
            }
        }
        Ok(())
    }
    pub async fn workflow_manage(&self, task: &str, args: &Value) -> Result<Value> {
        let action = args["action"].as_str().unwrap_or("list");
        let id = args["id"].as_str();
        let binding = self.store.task_binding(task)?;
        if let Some(id) = id {
            let flow = self.workflow(id)?;
            ensure!(
                flow.spec.project_id == binding.project_id,
                "Workflow is outside this task's project"
            );
        }
        if let Some(run_id) = args["run_id"].as_str() {
            let run = self.workflow_run(run_id)?;
            ensure!(
                run.workflow.spec.project_id == binding.project_id,
                "Execution is outside this task's project"
            );
        }
        if self.store.task(task)?.origin == "subagent" {
            ensure!(
                matches!(action, "list" | "get" | "runs" | "inspect"),
                "Subagents may inspect workflows but cannot create nested background work"
            );
        }
        let value = match action {
            "list" => json!(
                self.workflows()?
                    .into_iter()
                    .filter(|w| w.spec.project_id == binding.project_id)
                    .collect::<Vec<_>>()
            ),
            "get" => json!(self.workflow(id.context("Workflow ID required")?)?),
            "create" | "update" => {
                let mut spec: WorkflowInput = serde_json::from_value(args["workflow"].clone())?;
                if spec.project_id.is_none() {
                    spec.project_id = binding.project_id.clone();
                }
                ensure!(
                    spec.project_id == binding.project_id,
                    "Cannot create a flow in another project"
                );
                json!(
                    self.save_workflow(
                        if action == "update" {
                            Some(id.context("Workflow ID required")?)
                        } else {
                            None
                        },
                        args["expected_version"].as_u64(),
                        spec
                    )
                    .await?
                )
            }
            "run" | "test" => {
                let run = self
                    .enqueue_workflow(id.context("Workflow ID required")?, action == "test", None)
                    .await?;
                if let Some((channel, address)) = self.store.task_destination(task)? {
                    self.store
                        .set_task_destination(&run.task_id, &channel, address.as_deref())?;
                }
                json!(run)
            }
            "runs" => json!(
                self.workflow_runs(id)?
                    .into_iter()
                    .filter(|r| r.workflow.spec.project_id == binding.project_id)
                    .collect::<Vec<_>>()
            ),
            "inspect" => {
                json!(self.workflow_run(args["run_id"].as_str().context("Run ID required")?)?)
            }
            "cancel" => json!(
                self.cancel_workflow(args["run_id"].as_str().context("Run ID required")?)
                    .await?
            ),
            "resume" => json!(
                self.resume_workflow(args["run_id"].as_str().context("Run ID required")?)
                    .await?
            ),
            _ => anyhow::bail!("Unknown workflow action"),
        };
        if matches!(action, "create" | "update" | "get") {
            self.store.publish_event(Some(task),"workspace.open",&json!({"kind":"workflow","id":value["id"],"title":value["name"],"project_id":binding.project_id}))?;
            self.bus_wake.notify_waiters();
        }
        Ok(value)
    }
}
