//! Project contracts reuse fixed project scopes, the planner lock and tool discovery.
use crate::Runtime;
use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;

#[cfg(test)]
mod tests;

impl Runtime {
    pub(crate) fn authorize_project_spec(&self, task: &str, args: &Value) -> Result<String> {
        let project = self.readable_project(task, args)?.context(
            "A project-scoped contract is required; general chat has no implicit project",
        )?;
        let action = args["action"].as_str().context("Missing contract action")?;
        ensure!(
            matches!(action, "get" | "changes" | "propose" | "apply" | "discard"),
            "Unknown project contract action"
        );
        if !matches!(action, "get" | "changes") {
            let binding = self.store.task_binding(task)?;
            let owner = self.store.task(task)?;
            ensure!(
                binding.bound
                    && binding.mode == "project"
                    && binding.project_id.as_ref() == Some(&project),
                "Contract changes must stay in this task's writable project"
            );
            ensure!(
                owner.status == "running" && owner.origin != "subagent_report",
                "Only running execution tasks may amend contracts"
            );
            ensure!(
                !matches!(action, "apply" | "discard") || owner.origin != "subagent",
                "Workers propose; the master consolidates project contracts"
            );
        }
        Ok(project)
    }

    pub(crate) async fn project_spec_tool(
        &self,
        task: &str,
        args: &Value,
        cancel: &CancellationToken,
    ) -> Result<Value> {
        self.authorize_project_tool(task, "project_spec", args)?;
        let project = self.authorize_project_spec(task, args)?;
        let offset = args["offset"].as_u64().unwrap_or(0).min(usize::MAX as u64) as usize;
        match args["action"].as_str() {
            Some("get") => {
                return if let Some(id) = args["change_id"].as_str() {
                    self.store.project_spec_change(&project, id)
                } else {
                    self.store.project_spec(&project, offset, 20)
                };
            }
            Some("changes") => return self.store.project_spec_changes(&project, offset),
            _ => {}
        }
        // Contract updates cannot race the requirement preflight of a Git integration.
        let _plan = tokio::select! {
            _=cancel.cancelled()=>bail!("Contract update cancelled"),
            _=self.stop.cancelled()=>bail!("Runtime stopped"),
            lock=self.plan_lock.lock()=>lock,
        };
        self.store.check_plan_worker(task)?;
        self.authorize_project_tool(task, "project_spec", args)?;
        ensure!(!cancel.is_cancelled(), "Contract update cancelled");
        let expected = args["expected_version"]
            .as_u64()
            .context("Read the contract and provide expected_version")?;
        let reason = args["reason"]
            .as_str()
            .context("A rationale with owner/source references is required")?;
        match args["action"].as_str() {
            Some("propose") => self.store.propose_project_spec(
                task,
                &project,
                expected,
                reason,
                &serde_json::from_value::<Vec<fenix_storage::SpecDeltaInput>>(
                    args["deltas"].clone(),
                )?,
            ),
            Some("apply" | "discard") => Ok(json!(self.store.resolve_project_spec(
                task,
                &project,
                args["change_id"].as_str().context("Missing change_id")?,
                expected,
                args["action"] == "apply",
                reason
            )?)),
            _ => bail!("Unknown project contract action"),
        }
    }
}
