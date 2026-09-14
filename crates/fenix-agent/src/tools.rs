//! The complete initial tool registry. Runtime validates every call before dispatch.
use crate::{Runtime, config::Settings};
use anyhow::{Context, Result, bail};
use fenix_core::ToolDefinition;
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;

fn definition(
    name: &str,
    description: &str,
    properties: Value,
    required: &[&str],
) -> ToolDefinition {
    ToolDefinition {
        name: name.into(),
        description: description.into(),
        parameters: json!({"type":"object","properties":properties,"required":required,"additionalProperties":false}),
    }
}
fn base_catalog() -> Vec<ToolDefinition> {
    let string = json!({"type":"string"});
    let integer = json!({"type":"integer","minimum":0});
    vec![
        definition(
            "files_list",
            "List up to 200 entries inside the file workspace.",
            json!({"path":string}),
            &[],
        ),
        definition(
            "file_read",
            "Read a bounded range of lines from a UTF-8 file inside the workspace (up to 1 MiB).",
            json!({"path":string,"start_line":integer,"line_count":integer}),
            &["path"],
        ),
        definition(
            "files_search",
            "Case-insensitive literal search in workspace text files. Bounded traversal, no symlinks.",
            json!({"path":string,"query":string}),
            &["query"],
        ),
        definition(
            "file_write",
            "Create a new UTF-8 file in the workspace. Never overwrites. The runtime requests approval when required.",
            json!({"path":string,"content":string}),
            &["path", "content"],
        ),
        definition(
            "file_edit",
            "Replace an exact unique string in an existing workspace file. Read it first. Runtime enforces approval.",
            json!({"path":string,"expected":string,"replacement":string}),
            &["path", "expected", "replacement"],
        ),
        definition(
            "command_run",
            "Run a shell command in any explicitly supplied OS-accessible cwd. Not restricted to the file workspace. Runtime approval, timeout and managed process required. Use command_status to obtain output and exit code.",
            json!({"command":string,"cwd":string,"timeout_seconds":integer}),
            &["command", "cwd"],
        ),
        definition(
            "command_status",
            "Wait for a managed process from this task and read bounded output. Use wait_ms=10000 to avoid busy polling. Completed does not imply success; check exit_code.",
            json!({"process_id":string,"wait_ms":integer}),
            &["process_id"],
        ),
        definition(
            "command_cancel",
            "Cancel a managed process owned by this task; does not undo effects.",
            json!({"process_id":string}),
            &["process_id"],
        ),
        definition(
            "memory_search",
            "Lexically search memory within the fixed task project. Optional scope=global reads explicit global preferences; project_id is allowed only for projects authorized on this turn. Empty query lists recent records.",
            json!({"query":string,"limit":integer,"project_id":string,"scope":{"type":"string","enum":["project","global"]}}),
            &["query"],
        ),
        definition(
            "memory_manage",
            "Manage explicitly requested memories only. Actions get/add/update/delete. Search for duplicates before add; source is recorded automatically.",
            json!({"action":{"type":"string","enum":["get","add","update","delete"]},"id":string,"title":string,"content":string,"project_id":string,"scope":{"type":"string","enum":["project","global"]}}),
            &["action"],
        ),
        definition(
            "history_search",
            "Search the original conversation lexically and return message references.",
            json!({"query":string,"limit":integer,"project_id":string,"scope":{"type":"string","enum":["project","global"]}}),
            &["query"],
        ),
        definition(
            "history_read",
            "Recover a referenced original message and nearby history. At most 15 neighboring messages on each side.",
            json!({"message_id":string,"radius":integer,"project_id":string,"scope":{"type":"string","enum":["project","global"]}}),
            &["message_id"],
        ),
        definition(
            "skill_load",
            "Load a relevant installed, enabled manual skill. Metadata is already in context; read full instructions before applying.",
            json!({"name":string}),
            &["name"],
        ),
        definition(
            "schedule_manage",
            "Manage explicitly requested reminders/tasks in America/Lima. Actions list/get/create/update/pause/resume/cancel. create/update need name,content,mode(reminder/task),kind(once/interval/cron) and at (ISO timestamp; no offset means Peru), interval_seconds or cron (5,6,7 fields). Updates replace the complete spec. No missed-event replay.",
            json!({"action":{"type":"string","enum":["list","get","create","update","pause","resume","cancel"]},"id":string,"name":string,"content":string,"mode":{"type":"string","enum":["reminder","task"]},"kind":{"type":"string","enum":["once","interval","cron"]},"at":string,"interval_seconds":integer,"cron":string,"project_id":string,"scope":{"type":"string","enum":["project","global"]}}),
            &["action"],
        ),
        definition(
            "project_manage",
            "Read a project's recorded status/context without delegating a status lookup. Actions list (compact registry), get (authorized project), create (register an explicitly requested project), update (replace this project's brief using expected_version from get). Briefs preserve Objective; Constraints and preferences; Progress; Key decisions; Relevant files; Next steps; Critical context, with sources and assumptions. Subagents may only get/update their own project. Workspace is optional for non-code projects; aliases are comma-separated.",
            json!({"action":{"type":"string","enum":["list","get","create","update"]},"project_id":string,"name":string,"aliases":string,"description":string,"workspace":string,"brief":string,"expected_version":integer}),
            &["action"],
        ),
        definition(
            "task_inspect",
            "Inspect task state and paginated tool events; no rollback. Omit task_id for current task. Use after for more events, or call_id plus offset/length to page through one saved output.",
            json!({"task_id":string,"after":integer,"limit":integer,"call_id":string,"offset":integer,"length":integer}),
            &[],
        ),
        definition(
            "ask_user",
            "Ask a concise clarification and wait for a textual answer in the continuous conversation. Independent messages still queue.",
            json!({"question":string}),
            &["question"],
        ),
        definition(
            "request_approval",
            "Request textual approval for exactly one future file_write, file_edit or command_run call. Include the precise tool name and arguments. The approval is consumed once and cannot authorize different arguments. Runtime also requests approval automatically when needed.",
            json!({"tool":{"type":"string","enum":["file_write","file_edit","command_run"]},"arguments":{"type":"object","additionalProperties":true}}),
            &["tool", "arguments"],
        ),
    ]
}

pub fn catalog() -> Vec<ToolDefinition> {
    catalog_for(false)
}

pub fn catalog_for(delegated: bool) -> Vec<ToolDefinition> {
    let string = json!({"type":"string"});
    let mut tools = base_catalog();
    if delegated {
        // Children receive the master's relevant context, not its entire transcript.
        tools.retain(|t| !matches!(t.name.as_str(), "history_search" | "history_read"));
        tools.push(definition(
            "subagent_finish",
            "Finish your assigned task with an explicit outcome. Must be the only call in this response. Report completed effects and evidence, or blockers and what remains. Do not claim success without evidence. Never delegate; request further help through blockers.",
            json!({"status":{"type":"string","enum":["completed","blocked","failed"]},"summary":string,"artifacts":string,"evidence":string,"blockers":string}),
            &["status", "summary"],
        ));
    } else {
        tools.extend([
            definition("subagent_spawn", "Assign substantial work to a background subagent. Specify objective and concrete expected_output; optionally relevant context/references and constraints. project_id selects an authorized destination for multi-project requests; otherwise inherit the task project. Do brief status reads directly. Returns immediately; continue helping the user. Children cannot delegate. Identical assignments reuse their ID; never recreate failed jobs to bypass limits.", json!({"objective":string,"expected_output":string,"context":string,"constraints":string,"project_id":string}), &["objective", "expected_output"]),
            definition("subagent_status", "Read background subagent state and results, or list recent subagents. Do not poll in a loop; final reports arrive automatically.", json!({"subagent_id":string}), &[]),
            definition("subagent_cancel", "Cancel a queued or running subagent by ID. Stops managed processes without undoing completed effects.", json!({"subagent_id":string}), &["subagent_id"]),
        ]);
    }
    tools
}

pub fn validate(name: &str, args: &Value) -> Result<()> {
    validate_for(false, name, args)
}

pub fn validate_for(delegated: bool, name: &str, args: &Value) -> Result<()> {
    let definition = catalog_for(delegated)
        .into_iter()
        .find(|t| t.name == name)
        .context("Unknown tool")?;
    let object = args
        .as_object()
        .context("Tool arguments must be an object")?;
    if args.to_string().len() > 128_000 {
        bail!("Tool arguments exceed the size limit");
    }
    for key in definition.parameters["required"]
        .as_array()
        .context("Invalid tool schema")?
    {
        if !object.contains_key(key.as_str().unwrap_or_default()) {
            bail!("Missing required argument: {key}");
        }
    }
    for (key, value) in object {
        let schema = &definition.parameters["properties"][key];
        if schema.is_null() {
            bail!("Unknown argument: {key}");
        }
        let valid = match schema["type"].as_str() {
            Some("string") => value.is_string(),
            Some("integer") => value.as_u64().is_some(),
            Some("object") => value.is_object(),
            _ => false,
        };
        if !valid {
            bail!("Invalid type for argument: {key}");
        }
        if let Some(values) = schema["enum"].as_array()
            && !values.contains(value)
        {
            bail!("Invalid value for argument: {key}");
        }
    }
    Ok(())
}

impl Runtime {
    pub(crate) async fn execute_tool(
        self: &std::sync::Arc<Self>,
        task: &str,
        source: &str,
        settings: &Settings,
        name: &str,
        args: &Value,
        cancel: &CancellationToken,
    ) -> Result<Value> {
        let delegated = self.store.task(task)?.origin == "subagent";
        validate_for(delegated, name, args)?;
        self.authorize_project_tool(task, name, args)?;
        if cancel.is_cancelled() {
            bail!("Task cancelled");
        }
        match name {
            "files_list" | "file_read" | "files_search" | "file_write" | "file_edit" => {
                let lease = if matches!(name, "file_write" | "file_edit") {
                    Some(self.effect_lease(task, cancel).await?)
                } else {
                    None
                };
                let runtime = self.clone();
                let workspace = self.task_workspace(task, &settings.workspace)?;
                let name = name.to_owned();
                let args = args.clone();
                let task = task.to_owned();
                let cancel = cancel.clone();
                let tracking = lease.as_ref().map(|_| self.file_effects.begin(&task));
                tokio::task::spawn_blocking(move || {
                    let _tracking = tracking;
                    let _lease = lease;
                    if _tracking.is_none() { return runtime.files_execute(&workspace, &name, &args); }
                    let operation_id = fenix_storage::id();
                    runtime.store.event(&task,"file_effect_started",&json!({"operation_id":operation_id,"tool":name,"arguments":args}))?;
                    let result = if cancel.is_cancelled() {
                        Err(anyhow::anyhow!("Task cancelled before filesystem mutation"))
                    } else { runtime.files_execute(&workspace, &name, &args) };
                    let mut outcome = match &result {
                        Ok(value) => json!({"operation_id":operation_id,"result":value,"is_error":false}),
                        Err(error) => json!({"operation_id":operation_id,"error":error.to_string(),"is_error":true}),
                    };
                    runtime.redactor.value(&mut outcome);
                    runtime.store.event(&task,"file_effect_finished",&outcome)?;
                    result
                })
                    .await?
            }
            "command_run" => {
                let lease = self.effect_lease(task, cancel).await?;
                let cwd = std::path::Path::new(s(args, "cwd")?);
                let cwd = if cwd.is_absolute() {
                    cwd.to_owned()
                } else {
                    self.task_workspace(task, &settings.workspace)?.join(cwd)
                };
                self.processes
                    .run(
                        task,
                        s(args, "command")?,
                        &cwd,
                        args["timeout_seconds"].as_u64().unwrap_or(120),
                        cancel,
                        Some(lease),
                    )
                    .await
            }
            "command_status" => {
                self.processes
                    .status(
                        task,
                        s(args, "process_id")?,
                        args["wait_ms"].as_u64().unwrap_or(10_000),
                        &self.redactor,
                    )
                    .await
            }
            "command_cancel" => self.processes.cancel(task, s(args, "process_id")?).await,
            "memory_search" => Ok(json!(self.store.memories_scoped(
                s(args, "query")?,
                args["limit"].as_u64().unwrap_or(10) as usize,
                self.readable_project(task, args)?.as_deref(),
                false
            )?)),
            "memory_manage" => {
                let project = self.readable_project(task, args)?;
                let _lease = if args["action"] != "get" {
                    Some(self.effect_lease(task, cancel).await?)
                } else {
                    None
                };
                let result = match s(args, "action")? {
                    "get" => {
                        let id = s(args, "id")?;
                        Ok(json!(
                            self.store
                                .memories_scoped(id, 1, project.as_deref(), false)?
                                .into_iter()
                                .find(|m| m.id == id)
                                .context("Memory not found")?
                        ))
                    }
                    "add" => Ok(json!(self.store.save_memory_scoped(
                        None,
                        s(args, "title")?,
                        s(args, "content")?,
                        source,
                        project.as_deref(),
                        Some(task)
                    )?)),
                    "update" => Ok(json!(self.store.save_memory_scoped(
                        Some(s(args, "id")?),
                        s(args, "title")?,
                        s(args, "content")?,
                        source,
                        project.as_deref(),
                        Some(task)
                    )?)),
                    "delete" => {
                        self.store
                            .delete_memory_scoped(s(args, "id")?, project.as_deref())?;
                        Ok(json!({"deleted":args["id"]}))
                    }
                    _ => bail!("Unknown memory action"),
                };
                self.export_memory()?;
                result
            }
            "history_search" => Ok(json!(self.store.history_search_scoped(
                s(args, "query")?,
                args["limit"].as_u64().unwrap_or(10) as usize,
                self.readable_project(task, args)?.as_deref()
            )?)),
            "history_read" => Ok(json!(self.store.history_read_scoped(
                s(args, "message_id")?,
                args["radius"].as_u64().unwrap_or(3) as usize,
                self.readable_project(task, args)?.as_deref()
            )?)),
            "skill_load" => {
                let name = s(args, "name")?;
                let content = self.skill_load(name)?;
                let metadata = self.skills()?.into_iter().find(|s| s.name == name);
                Ok(json!({"skill":metadata,"content":content}))
            }
            "schedule_manage" => {
                let _lease = if !matches!(args["action"].as_str(), Some("list" | "get")) {
                    Some(self.effect_lease(task, cancel).await?)
                } else {
                    None
                };
                let project = self.readable_project(task, args)?;
                let mut args = args.clone();
                args.as_object_mut().unwrap().remove("scope");
                args["project_id"] = json!(project);
                self.schedule_manage(args).await
            }
            "project_manage" => self.project_tool(task, args),
            "task_inspect" => {
                let id = args["task_id"].as_str().unwrap_or(task);
                if delegated && id != task {
                    bail!(
                        "Subagents may inspect only their own task evidence; ask the master for other context"
                    );
                }
                if let Some(call) = args["call_id"].as_str() {
                    return self.store.tool_output(
                        id,
                        call,
                        args["offset"].as_u64().unwrap_or(0) as usize,
                        args["length"].as_u64().unwrap_or(8000) as usize,
                    );
                }
                Ok(
                    json!({"task":self.store.task(id)?,"events":self.store.events(id,args["after"].as_i64().unwrap_or(0),args["limit"].as_u64().unwrap_or(20) as usize)?}),
                )
            }
            "ask_user" => Ok(
                json!({"answer":self.ask(task,"clarification",s(args,"question")?,None,cancel).await?}),
            ),
            "subagent_spawn" => self.spawn_subagent(task, args).await,
            "subagent_status" => self.scoped_subagent_status(task, args["subagent_id"].as_str()),
            "subagent_cancel" => self.cancel_subagent(s(args, "subagent_id")?).await,
            _ => bail!("This tool requires permission dispatch"),
        }
    }

    async fn effect_lease(
        &self,
        task: &str,
        cancel: &CancellationToken,
    ) -> Result<tokio::sync::OwnedMutexGuard<()>> {
        if self.processes.has_running(task).await {
            bail!(
                "This task has a running command. Inspect it with command_status or cancel it before requesting another effect"
            );
        }
        tokio::select! {
            biased;
            _ = cancel.cancelled() => bail!("Task cancelled before executing an effect"),
            lease = self.effects.clone().lock_owned() => Ok(lease),
        }
    }
}
fn s<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .with_context(|| format!("Missing text argument: {key}"))
}
