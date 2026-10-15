//! Runtime-enforced project scopes. Model routing selects a scope once; tools
//! cannot silently widen it or retarget a running task when chat focus changes.
use crate::{Runtime, redaction::truncate, tools};
use anyhow::{Context, Result, bail};
use fenix_core::ToolDefinition;
use fenix_storage::{ProjectInput, TaskBinding};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

impl Runtime {
    pub(crate) fn project_catalog(&self) -> Result<Value> {
        Ok(json!(
            self.store
                .projects()?
                .into_iter()
                .map(|p| json!({
                    "id":p.id,"name":p.name,"aliases":p.aliases,
                    "description":truncate(&p.description,500),"context_version":p.context_version,
                }))
                .collect::<Vec<_>>()
        ))
    }

    pub(crate) fn task_workspace(&self, task: &str, fallback: &Path) -> Result<PathBuf> {
        if let Some(worktree) = self.worktree(task)? {
            anyhow::ensure!(worktree.status != "provisioning", "Worktree is not ready");
            return Ok(worktree.path);
        }
        Ok(self
            .store
            .task_binding(task)?
            .workspace
            .map(PathBuf::from)
            .unwrap_or_else(|| fallback.to_owned()))
    }

    pub(crate) fn readable_project(&self, task: &str, args: &Value) -> Result<Option<String>> {
        let binding = self.store.task_binding(task)?;
        if args["scope"] == "global" {
            if args.get("project_id").is_some() {
                bail!("Global scope cannot include a project ID");
            }
            return Ok(None);
        }
        let project = args["project_id"]
            .as_str()
            .map(str::to_owned)
            .or(binding.project_id.clone());
        if let Some(id) = &project {
            if binding.project_id.as_ref() != Some(id) && !binding.read_project_ids.contains(id) {
                bail!(
                    "Project is outside this task's fixed scope; the owner must request it in the conversation"
                );
            }
            self.store.project(id)?;
        }
        Ok(project)
    }

    pub(crate) fn authorize_task_read(&self, reader: &str, target: &str) -> Result<()> {
        if reader == target {
            return Ok(());
        }
        if self.store.task(reader)?.origin == "subagent" {
            if self
                .completion_audit_evidence_sources(reader)?
                .iter()
                .any(|id| id == target)
                || self
                    .store
                    .plan_evidence_sources(reader)?
                    .iter()
                    .any(|id| id == target)
                || self
                    .review_evidence_sources(reader)?
                    .iter()
                    .any(|id| id == target)
            {
                return Ok(());
            }
            bail!("Subagents may inspect only their own task evidence");
        }
        let other = self.store.task_binding(target)?;
        let own = self.store.task_binding(reader)?;
        if !other.bound || other.mode == "clarification" {
            bail!("This task has no resolved scope and cannot be read from another task");
        }
        if other.mode == "coordination" {
            // A multi-project transcript cannot be exposed through a task-ID
            // shortcut to a reader that only has one of its project scopes.
            if !other
                .read_project_ids
                .iter()
                .all(|id| own.read_project_ids.contains(id) || own.project_id.as_ref() == Some(id))
            {
                bail!("This task contains evidence from other project scopes");
            }
        } else if let Some(id) = other.project_id {
            self.readable_project(reader, &json!({"project_id":id}))?;
        } else if own.project_id.is_some() {
            bail!(
                "General task transcripts are not project evidence; use explicit global memories"
            );
        }
        Ok(())
    }

    pub(crate) fn task_tool_catalog(&self, task: &str) -> Result<Vec<ToolDefinition>> {
        let origin = self.store.task(task)?.origin;
        let delegated = origin == "subagent";
        let binding = self.store.task_binding(task)?;
        let mut catalog = tools::catalog_for(delegated);
        catalog.retain(|tool| tool.name != "execution_status");
        if matches!(binding.mode.as_str(), "coordination" | "clarification")
            || origin == "subagent_report"
        {
            catalog.retain(|t| !t.name.starts_with("browser_"));
        }
        if matches!(binding.mode.as_str(), "coordination" | "clarification") {
            catalog.retain(|t| {
                !matches!(
                    t.name.as_str(),
                    "files_list"
                        | "document_read"
                        | "document_extract"
                        | "file_read"
                        | "files_search"
                        | "files_glob"
                        | "code_query"
                        | "document_create"
                        | "design_present"
                        | "diagram_edit"
                        | "library_export"
                        | "library_import"
                        | "design_publish"
                        | "design_import"
                        | "document_edit"
                        | "file_write"
                        | "file_edit"
                        | "file_patch"
                        | "command_run"
                        | "environment_inspect"
                        | "workspace_checkpoint"
                        | "integration_call"
                        | "connector_call"
                        | "workflow_manage"
                        | "tool_program"
                        | "file_history"
                        | "skill_learning"
                        | "command_input"
                        | "command_status"
                        | "command_output"
                        | "command_list"
                        | "command_cancel"
                        | "request_approval"
                )
            });
        }
        if origin == "subagent_report" {
            catalog.retain(|t| {
                !matches!(
                    t.name.as_str(),
                    "document_create"
                        | "design_present"
                        | "diagram_edit"
                        | "library_export"
                        | "library_import"
                        | "design_publish"
                        | "design_import"
                        | "document_edit"
                        | "file_write"
                        | "file_edit"
                        | "file_patch"
                        | "command_run"
                        | "environment_inspect"
                        | "workspace_checkpoint"
                        | "integration_call"
                        | "connector_call"
                        | "workflow_manage"
                        | "tool_program"
                        | "file_history"
                        | "skill_learning"
                        | "command_input"
                        | "command_cancel"
                        | "request_approval"
                        | "subagent_spawn"
                        | "subagent_cancel"
                        | "subagent_send"
                        | "plan_manage"
                )
            });
        }
        let completion_auditor = self.store.completion_audit_for_worker(task)?.is_some();
        let mut filtered = Vec::new();
        for mut tool in catalog {
            if self.review_tool_allowed(task, &tool.name, &json!({"action":"get"}))? {
                if tool.name == "worktree_manage" && delegated {
                    tool.parameters["properties"]["action"]["enum"] = json!(["get"]);
                }
                if tool.name == "project_spec" {
                    let read_only = completion_auditor
                        || self.store.review_for_worker(task)?.is_some()
                        || self.store.get::<String>("review_edits", task)?.is_some()
                        || binding.mode != "project"
                        || origin == "subagent_report";
                    tool.parameters["properties"]["action"]["enum"] = if read_only {
                        json!(["get", "changes"])
                    } else if delegated {
                        json!(["get", "changes", "propose"])
                    } else {
                        json!(["get", "changes", "propose", "apply", "discard"])
                    };
                }
                if tool.name == "subagent_finish" && self.store.review_for_worker(task)?.is_some() {
                    tool.parameters["properties"]
                        .as_object_mut()
                        .unwrap()
                        .remove("security_report");
                }
                if tool.name == "subagent_finish" && self.store.review_for_worker(task)?.is_none() {
                    tool.parameters["properties"]
                        .as_object_mut()
                        .unwrap()
                        .remove("review");
                }
                if tool.name == "subagent_finish" && !completion_auditor {
                    tool.parameters["properties"]
                        .as_object_mut()
                        .unwrap()
                        .remove("completion_audit");
                }
                if completion_auditor
                    && matches!(
                        tool.name.as_str(),
                        "schedule_manage" | "project_manage" | "plan_evidence"
                    )
                {
                    tool.parameters["properties"]["action"]["enum"] =
                        if tool.name == "plan_evidence" {
                            json!(["list"])
                        } else {
                            json!(["get", "list"])
                        };
                    tool.description = format!(
                        "Read-only inspection for completion auditing. {}",
                        tool.description
                    );
                }
                filtered.push(tool);
            }
        }
        Ok(filtered)
    }

    pub(crate) fn authorize_project_tool(
        &self,
        task: &str,
        name: &str,
        args: &Value,
    ) -> Result<()> {
        let binding = self.store.task_binding(task)?;
        if name == "project_spec" {
            self.authorize_project_spec(task, args)?;
        }
        if name == "worktree_manage" {
            if let Some(target) = args["subagent_id"].as_str() {
                self.authorize_task_read(task, target)?;
            }
            if args["action"] == "integrate" {
                let current = self.store.task(task)?;
                anyhow::ensure!(
                    current.origin != "subagent"
                        && current.origin != "subagent_report"
                        && !matches!(
                            self.store.task_binding(task)?.mode.as_str(),
                            "coordination" | "clarification"
                        ),
                    "Worktree integration requires an authorized master execution task"
                );
            }
        }
        if name == "skill_load" {
            self.authorize_skill(task, args["name"].as_str().unwrap_or_default())?;
        }
        self.check_plan_editor(task)?;
        if !self.review_tool_allowed(task, name, args)? {
            bail!(
                "Plan reviewers may only inspect evidence and ask questions; the master alone resolves plans"
            );
        }
        if (self.store.get::<String>("plan_edits", task)?.is_some()
            || self
                .store
                .plan_for_task(task)?
                .is_some_and(|p| p.task_id == task))
            && (matches!(
                name,
                "document_create"
                    | "document_extract"
                    | "design_present"
                    | "diagram_edit"
                    | "library_export"
                    | "library_import"
                    | "design_publish"
                    | "design_import"
                    | "document_edit"
                    | "file_write"
                    | "file_edit"
                    | "file_patch"
                    | "command_run"
                    | "environment_inspect"
                    | "workspace_checkpoint"
                    | "integration_call"
                    | "connector_call"
                    | "workflow_manage"
                    | "tool_program"
                    | "file_history"
                    | "skill_learning"
                    | "command_input"
                    | "subagent_spawn"
            ) || name == "memory_manage" && args["action"] != "get"
                || name == "knowledge_manage"
                    && matches!(
                        args["action"].as_str(),
                        Some("entity_save" | "schema_save" | "relation_save" | "review_resolve")
                    )
                || name == "schedule_manage"
                    && !matches!(args["action"].as_str(), Some("get" | "list")))
        {
            bail!(
                "This turn manages a plan. Delegate effects through its milestones instead of executing them twice"
            );
        }

        if self.store.task(task)?.origin == "subagent_report"
            && (matches!(
                name,
                "document_create"
                    | "document_extract"
                    | "design_present"
                    | "diagram_edit"
                    | "library_export"
                    | "library_import"
                    | "design_publish"
                    | "design_import"
                    | "document_edit"
                    | "file_write"
                    | "file_edit"
                    | "file_patch"
                    | "command_run"
                    | "environment_inspect"
                    | "workspace_checkpoint"
                    | "integration_call"
                    | "connector_call"
                    | "workflow_manage"
                    | "tool_program"
                    | "file_history"
                    | "skill_learning"
                    | "command_input"
                    | "command_cancel"
                    | "request_approval"
                    | "subagent_spawn"
                    | "subagent_cancel"
                    | "subagent_send"
            ) || (name == "memory_manage" && args["action"] != "get"
                || name == "knowledge_manage"
                    && matches!(
                        args["action"].as_str(),
                        Some("entity_save" | "schema_save" | "relation_save" | "review_resolve")
                    ))
                || (name == "schedule_manage"
                    && !matches!(args["action"].as_str(), Some("list" | "get")))
                || (name == "project_manage"
                    && !matches!(args["action"].as_str(), Some("list" | "get"))))
        {
            bail!(
                "Background reports may inspect evidence and notify the owner, but cannot start new effects"
            );
        }
        if matches!(binding.mode.as_str(), "coordination" | "clarification") {
            if matches!(
                name,
                "files_list"
                    | "document_read"
                    | "document_extract"
                    | "file_read"
                    | "files_search"
                    | "files_glob"
                    | "code_query"
                    | "document_create"
                    | "design_present"
                    | "diagram_edit"
                    | "library_export"
                    | "library_import"
                    | "design_publish"
                    | "design_import"
                    | "document_edit"
                    | "file_write"
                    | "file_edit"
                    | "file_patch"
                    | "command_run"
                    | "environment_inspect"
                    | "workspace_checkpoint"
                    | "integration_call"
                    | "connector_call"
                    | "workflow_manage"
                    | "tool_program"
                    | "file_history"
                    | "skill_learning"
                    | "command_input"
                    | "command_status"
                    | "command_output"
                    | "command_list"
                    | "command_cancel"
                    | "request_approval"
            ) {
                bail!(
                    "This turn coordinates multiple projects; delegate execution with an explicit project_id"
                );
            }
            if (name == "memory_manage" && args["action"] != "get"
                || name == "knowledge_manage"
                    && matches!(
                        args["action"].as_str(),
                        Some("entity_save" | "schema_save" | "relation_save" | "review_resolve")
                    ))
                || (name == "schedule_manage"
                    && !matches!(args["action"].as_str(), Some("list" | "get")))
                || (name == "project_manage"
                    && !matches!(args["action"].as_str(), Some("list" | "get")))
            {
                bail!(
                    "A coordinating turn cannot mutate project data; delegate the specific destination task"
                );
            }
        }
        if name.starts_with("browser_")
            && (matches!(binding.mode.as_str(), "coordination" | "clarification")
                || self.store.task(task)?.origin == "subagent_report")
        {
            bail!("Browser execution requires an authorized execution task");
        }
        match name {
            "memory_search" | "memory_manage" | "history_search" | "history_read"
            | "schedule_manage" => {
                let project = self.readable_project(task, args)?;
                if ((name == "memory_manage" && args["action"] != "get"
                    || name == "knowledge_manage"
                        && matches!(
                            args["action"].as_str(),
                            Some(
                                "entity_save" | "schema_save" | "relation_save" | "review_resolve"
                            )
                        ))
                    || (name == "schedule_manage"
                        && !matches!(args["action"].as_str(), Some("list" | "get"))))
                    && project != binding.project_id
                {
                    bail!(
                        "Writes must stay in the task's assigned scope; do not promote project facts to global memory"
                    );
                }
            }
            "task_inspect" => {
                self.authorize_task_read(task, args["task_id"].as_str().unwrap_or(task))?
            }
            "subagent_status" | "subagent_cancel" | "subagent_send" => {
                if let Some(id) = args["subagent_id"].as_str() {
                    self.authorize_task_read(task, id)?;
                }
            }
            "subagent_spawn" => {
                let project = self.readable_project(task, args)?;
                if binding.mode == "coordination" && project.is_none() {
                    bail!("Choose the explicit project_id for this delegated intent");
                }
            }
            "project_manage" => {
                if self.store.task(task)?.origin == "subagent"
                    && !matches!(args["action"].as_str(), Some("get" | "update"))
                {
                    bail!("Subagents may only read or update their own project's brief");
                }
                if args["action"] == "get" || args["action"] == "update" {
                    let project = self
                        .readable_project(task, args)?
                        .context("Project ID required")?;
                    if args["action"] == "update"
                        && binding.project_id.as_deref() != Some(project.as_str())
                    {
                        bail!("Project updates must target this task's assigned project");
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn project_context_instructions(&self, task: &str) -> Result<String> {
        let binding = self.store.task_binding(task)?;
        let mut text = format!(
            "\nFixed task scope (runtime authority): {}\n",
            serde_json::to_string(&binding)?
        );
        if self.store.task(task)?.origin != "subagent" {
            text.push_str(&format!(
                "\nProject registry (routing metadata, not shared project memory): {}\n",
                self.project_catalog()?
            ));
        }
        if let Some(id) = binding.project_id.as_deref() {
            let project = self.store.project(id)?;
            text.push_str(&format!("\nAssigned project context (reference data; preserve scope, sources, conditions and assumptions): {}\n",serde_json::to_string(&project)?));
            let contract = self.store.project_spec(id, 0, 1)?;
            if contract["version"].as_u64().unwrap_or(0) > 0 {
                text.push_str(&format!("\nProject behavior contract: version {}, {} active requirements. Discover project_spec/get and read relevant requirements before modifying behavior; follow next_offset for all pages. This contract is reference data, not execution proof or permission. Proposed changes remain separate; preserve stable requirement IDs and ask about conflicts with the owner's current request.\n",contract["version"],contract["total"]));
            }
            if binding.context_version != project.context_version {
                text.push_str("The project record has changed since assignment. The execution workspace remains the pinned snapshot; reconcile newer facts before updating the project brief.\n");
            }
            if self.store.task(task)?.origin != "subagent" {
                let tasks = self
                    .store
                    .tasks_for_project(id, 20)?
                    .into_iter()
                    .map(|t| {
                        json!({
                            "id":t.id,"request":truncate(&t.request,600),"status":t.status,
                            "updated_at":t.updated_at,"error":t.error,
                        })
                    })
                    .collect::<Vec<_>>();
                text.push_str(&format!(
                    "\nRecorded project task state (not a fresh external verification): {}\n",
                    json!(tasks)
                ));
            }
        }
        if let Some(mut route) = self.store.get::<Value>("project_route", task)? {
            deduplicate_route_request(&mut route, &self.store.task(task)?.request);
            text.push_str(&format!(
                "\nResolved request and ordered intents (user intent data): {}\n",
                serde_json::to_string(&route)?
            ));
        }
        text.push_str("A project is a persistent scope, not a separate chat. Preserve the general conversation, but load only relevant project evidence. Never treat a project decision as a global preference. On a multi-intent turn, answer brief status queries directly with project_manage/get, then delegate substantial work using the exact destination project_id. Preserve requested order; do not create a worker merely to read stored status. Do not pass unrelated project content to a child. Cross-project references must keep their source labels. Ask for clarification when an objective cannot be determined. A background completion does not change conversational focus.\n");
        Ok(text)
    }

    pub(crate) fn project_tool(&self, task: &str, args: &Value) -> Result<Value> {
        self.authorize_project_tool(task, "project_manage", args)?;
        match args["action"].as_str() {
            Some("list") => self.project_catalog(),
            Some("get") => {
                let id = self
                    .readable_project(task, args)?
                    .context("Project ID required")?;
                let project = self.store.project(&id)?;
                let tasks = self.store.tasks_for_project(&id, 50)?;
                let children = self.store.subagents(None,200)?.into_iter().filter_map(|job| {
                    match self.store.task_binding(&job.id) {
                        Ok(b) if b.project_id.as_deref() == Some(id.as_str()) => Some(Ok(json!({"id":job.id,"status":job.status,"objective":job.spec.objective,"result":job.result}))),
                        Ok(_) => None,
                        Err(error) => Some(Err(error)),
                    }
                }).collect::<Result<Vec<_>>>()?;
                Ok(
                    json!({"project":project,"tasks":tasks,"subagents":children,"status_source":"Recorded task state; inspect evidence if a fresh verification is required."}),
                )
            }
            Some("create") => {
                let input = ProjectInput {
                    name: args["name"]
                        .as_str()
                        .context("Project name required")?
                        .into(),
                    aliases: aliases(args),
                    description: args["description"].as_str().unwrap_or_default().into(),
                    workspace: args["workspace"].as_str().map(str::to_owned),
                    brief: args["brief"].as_str().unwrap_or_default().into(),
                };
                let project = self.save_project(None, input)?;
                self.store
                    .event(task, "project_created", &json!({"project_id":project.id}))?;
                Ok(json!(project))
            }
            Some("update") => {
                let id = self
                    .readable_project(task, args)?
                    .context("Project ID required")?;
                // Context changes use optimistic concurrency so a late worker
                // cannot replace newer decisions with its older snapshot.
                let brief = args["brief"]
                    .as_str()
                    .context("An updated project brief is required")?;
                let version = args["expected_version"]
                    .as_u64()
                    .context("Read the project first and supply expected_version")?;
                let result =
                    self.store
                        .update_project_brief(&id, &self.redactor.text(brief), version)?;
                self.store.event(
                    task,
                    "project_context_updated",
                    &json!({"project_id":id,"previous_version":version}),
                )?;
                Ok(json!(result))
            }
            _ => bail!("Unknown project action"),
        }
    }

    pub(crate) fn scoped_subagent_status(&self, task: &str, id: Option<&str>) -> Result<Value> {
        if let Some(id) = id {
            self.authorize_task_read(task, id)?;
            return self.subagent_status(Some(id));
        }
        let binding: TaskBinding = self.store.task_binding(task)?;
        let jobs = self
            .store
            .subagents(None, 100)?
            .into_iter()
            .filter_map(|job| match self.store.task_binding(&job.id) {
                Ok(other)
                    if other.project_id == binding.project_id
                        || other
                            .project_id
                            .as_ref()
                            .is_some_and(|id| binding.read_project_ids.contains(id)) =>
                {
                    Some(Ok(job))
                }
                Ok(_) => None,
                Err(error) => Some(Err(error)),
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(json!({"subagents":jobs,"max_concurrent":4,"max_attempts":3}))
    }
}

fn aliases(args: &Value) -> Vec<String> {
    args["aliases"]
        .as_str()
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;
    use fenix_core::{Message, Role};
    use tokio_util::sync::CancellationToken;

    #[tokio::test]
    async fn scope_checks_cover_id_shortcuts_global_promotion_and_workspace_changes() -> Result<()>
    {
        let dir = tempfile::tempdir()?;
        let runtime =
            Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills"))).await?;
        let first = dir.path().join("first");
        let second = dir.path().join("second");
        std::fs::create_dir(&first)?;
        std::fs::create_dir(&second)?;
        let a = runtime.save_project(
            None,
            ProjectInput {
                name: "Alpha".into(),
                workspace: Some(first.to_string_lossy().into_owned()),
                ..Default::default()
            },
        )?;
        let b = runtime.save_project(
            None,
            ProjectInput {
                name: "Beta".into(),
                workspace: Some(second.to_string_lossy().into_owned()),
                ..Default::default()
            },
        )?;
        let make_task = |project: &fenix_storage::Project| {
            runtime.store.enqueue_bound(
                "Scoped task",
                "web",
                None,
                &TaskBinding {
                    project_id: Some(project.id.clone()),
                    mode: "project".into(),
                    ..Default::default()
                },
            )
        };
        let task_a = make_task(&a)?;
        let task_b = make_task(&b)?;
        let unbound = runtime
            .store
            .enqueue("Unresolved Beta request", "web", None)?;
        let b_message = runtime.store.append(
            Some(&task_b.id),
            &Message::user("B_ONLY_ORIGINAL"),
            true,
            true,
            "web",
        )?;
        let b_memory = runtime.store.save_memory_scoped(
            None,
            "Beta fact",
            "B_ONLY_FACT",
            "owner",
            Some(&b.id),
            Some(&task_b.id),
        )?;
        let settings = Settings::initial(dir.path().to_owned());
        let cancel = CancellationToken::new();
        assert!(
            runtime
                .authorize_task_read(&task_a.id, &unbound.id)
                .is_err()
        );
        assert!(runtime.authorize_task_read(&task_a.id, &task_b.id).is_err());
        runtime.store.event(&task_b.id, "tool_result", &json!({"tool":"file_read","call_id":"private-call","result":{"content":"B_ONLY_OUTPUT"}}))?;
        assert!(
            runtime
                .execute_tool(
                    &task_a.id,
                    "search",
                    &settings,
                    "task_inspect",
                    &json!({"task_id":task_b.id,"query":"B_ONLY_OUTPUT"}),
                    &cancel
                )
                .await
                .is_err()
        );
        assert!(
            runtime
                .execute_tool(
                    &task_a.id,
                    "recent",
                    &settings,
                    "task_inspect",
                    &json!({"task_id":task_b.id,"recent":true}),
                    &cancel
                )
                .await
                .is_err()
        );
        let own_recent = runtime
            .execute_tool(
                &task_b.id,
                "recent",
                &settings,
                "task_inspect",
                &json!({"recent":true}),
                &cancel,
            )
            .await?;
        assert_eq!(own_recent["events"][0]["data"]["call_id"], "private-call");
        let own_search = runtime
            .execute_tool(
                &task_b.id,
                "search",
                &settings,
                "task_inspect",
                &json!({"query":"B_ONLY_OUTPUT"}),
                &cancel,
            )
            .await?;
        assert_eq!(own_search["results"][0]["call_id"], "private-call");
        assert!(
            runtime
                .execute_tool(
                    &task_a.id,
                    "source",
                    &settings,
                    "history_read",
                    &json!({"message_id":b_message}),
                    &cancel
                )
                .await
                .is_err()
        );
        assert!(runtime.execute_tool(&task_a.id,"source",&settings,"memory_manage",&json!({"action":"update","id":b_memory.id,"title":"Wrong scope","content":"overwrite"}),&cancel).await.is_err());
        assert!(runtime.execute_tool(&task_a.id,"source",&settings,"memory_manage",&json!({"action":"add","scope":"global","title":"Project preference","content":"must not promote"}),&cancel).await.is_err());
        let found = runtime
            .execute_tool(
                &task_a.id,
                "source",
                &settings,
                "memory_search",
                &json!({"query":"B_ONLY_FACT"}),
                &cancel,
            )
            .await?;
        assert!(found["results"].as_array().unwrap().is_empty());
        runtime.save_project(
            Some(&a.id),
            ProjectInput {
                name: a.name,
                workspace: b.workspace,
                ..Default::default()
            },
        )?;
        runtime
            .execute_tool(
                &task_a.id,
                "source",
                &settings,
                "file_write",
                &json!({"path":"pinned.txt","content":"original destination"}),
                &cancel,
            )
            .await?;
        assert!(first.join("pinned.txt").exists());
        assert!(!second.join("pinned.txt").exists());
        Ok(())
    }

    #[tokio::test]
    async fn project_context_keeps_general_dialogue_without_other_project_transcripts() -> Result<()>
    {
        let dir = tempfile::tempdir()?;
        let runtime =
            Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills"))).await?;
        let a = runtime.save_project(
            None,
            ProjectInput {
                name: "Alpha".into(),
                brief: "A_PROJECT_BRIEF".into(),
                ..Default::default()
            },
        )?;
        let b = runtime.save_project(
            None,
            ProjectInput {
                name: "Beta".into(),
                brief: "B_PROJECT_BRIEF".into(),
                ..Default::default()
            },
        )?;
        let general =
            runtime
                .store
                .enqueue_bound("General message", "web", None, &TaskBinding::default())?;
        runtime.store.append(
            Some(&general.id),
            &Message::text(Role::Assistant, "GENERAL_DIALOGUE_MARKER"),
            true,
            true,
            "agent",
        )?;
        let other = runtime.store.enqueue_bound(
            "Beta message",
            "web",
            None,
            &TaskBinding {
                project_id: Some(b.id),
                mode: "project".into(),
                ..Default::default()
            },
        )?;
        runtime.store.append(
            Some(&other.id),
            &Message::text(Role::Assistant, "B_TRANSCRIPT_MARKER"),
            true,
            true,
            "agent",
        )?;
        let task = runtime.store.enqueue_bound(
            "Alpha message",
            "web",
            None,
            &TaskBinding {
                project_id: Some(a.id),
                mode: "project".into(),
                ..Default::default()
            },
        )?;
        runtime.store.append(
            Some(&task.id),
            &Message::user("A_CURRENT_MARKER"),
            true,
            true,
            "web",
        )?;
        let model = runtime.model_connection(
            Settings::initial(dir.path().to_owned()),
            fenix_credentials::ApiKey::new("fixture-key")?,
        )?;
        let context = serde_json::to_string(&runtime.build_context(&task, &model).await?)?;
        assert!(context.contains("GENERAL_DIALOGUE_MARKER"));
        assert!(context.contains("A_CURRENT_MARKER"));
        assert!(context.contains("A_PROJECT_BRIEF"));
        assert!(!context.contains("B_TRANSCRIPT_MARKER"));
        assert!(!context.contains("B_PROJECT_BRIEF"));
        let coordinator = runtime.store.enqueue_bound(
            "Compare projects",
            "web",
            None,
            &TaskBinding {
                mode: "coordination".into(),
                read_project_ids: runtime
                    .store
                    .projects()?
                    .into_iter()
                    .map(|p| p.id)
                    .collect(),
                ..Default::default()
            },
        )?;
        assert!(
            runtime
                .authorize_project_tool(
                    &coordinator.id,
                    "file_write",
                    &json!({"path":"x","content":"y"})
                )
                .is_err()
        );
        assert!(
            runtime
                .authorize_task_read(&task.id, &coordinator.id)
                .is_err()
        );
        Ok(())
    }
}

// Prompt projection only: durable routing and distinct intent text remain intact.
fn deduplicate_route_request(route: &mut Value, request: &str) {
    let reference = "[Same as the current task request; use its original text in this context.]";
    if route["resolved_request"].as_str() == Some(request) {
        route["resolved_request"] = json!(reference);
    }
    if let Some(intents) = route["intents"].as_array_mut() {
        for intent in intents {
            if intent["request"].as_str() == Some(request) {
                intent["request"] = json!(reference);
            }
        }
    }
}

#[cfg(test)]
mod prompt_projection_tests {
    use super::*;
    #[test]
    fn only_exact_repeated_requests_are_referenced() {
        let original = json!({"resolved_request":"Exact owner instruction","intents":[{"request":"Exact owner instruction","project_id":"a"},{"request":"Different intent","project_id":"b"}],"clarification":["Unresolved choice"]});
        let mut projected = original.clone();
        deduplicate_route_request(&mut projected, "Exact owner instruction");
        assert_ne!(projected["resolved_request"], original["resolved_request"]);
        assert_eq!(projected["intents"][1], original["intents"][1]);
        assert_eq!(projected["clarification"], original["clarification"]);
        assert_eq!(projected["intents"][0]["project_id"], "a");
        assert_eq!(original["intents"][0]["request"], "Exact owner instruction");
    }
}
