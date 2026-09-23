//! Native connectors share the authenticated Runtime and the task's fixed workspace.
use crate::Runtime;
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

impl Runtime {
    pub(crate) fn connector_requires_approval(
        &self,
        settings: &crate::config::Settings,
        args: &Value,
    ) -> bool {
        if args["action"] != "invoke" {
            return false;
        }
        let effect = self.connectors.effect(
            args["connection"].as_str().unwrap_or_default(),
            args["operation"].as_str().unwrap_or_default(),
            &args["arguments"],
        );
        match effect {
            Ok("read") => false,
            Ok("file_write") => settings.file_write_policy == crate::config::ActionPolicy::Ask,
            _ => settings.command_policy == crate::config::ActionPolicy::Ask,
        }
    }
    pub fn validate_connection_workspace(&self, workspace: &std::path::Path) -> Result<()> {
        self.validate_user_workspace(workspace)
    }
    pub async fn connector_call(&self, task: &str, args: &Value) -> Result<Value> {
        if args["action"] == "list" {
            return Ok(
                json!({"catalog":fenix_connectors::catalog(),"connections":self.connectors.list()?}),
            );
        }
        let settings = self.settings()?.context("Configure the workspace first")?;
        let workspace = self.task_workspace(task, &settings.workspace)?;
        self.validate_user_workspace(&workspace)?;
        let connection = args["connection"]
            .as_str()
            .context("Connection ID required")?;
        let operation = args["operation"].as_str().context("Operation required")?;
        if args["action"] == "describe" {
            return self.connectors.describe(connection, operation);
        }
        ensure!(
            args["arguments"].is_object(),
            "Connector arguments must be an object"
        );
        let mut value = self
            .connectors
            .invoke(connection, operation, &args["arguments"], &workspace)
            .await?;
        if let Some(path) = value["path"].as_str() {
            let _ = self.notify_workspace_file(task, &workspace, path, None);
        }
        self.redactor.value(&mut value);
        Ok(value)
    }
}
