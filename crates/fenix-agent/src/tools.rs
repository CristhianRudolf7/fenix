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
pub fn catalog() -> Vec<ToolDefinition> {
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
            "Lexically search explicit durable memory by words, title or ID. Empty query lists recent records.",
            json!({"query":string,"limit":integer}),
            &["query"],
        ),
        definition(
            "memory_manage",
            "Manage explicitly requested memories only. Actions get/add/update/delete. Search for duplicates before add; source is recorded automatically.",
            json!({"action":{"type":"string","enum":["get","add","update","delete"]},"id":string,"title":string,"content":string}),
            &["action"],
        ),
        definition(
            "history_search",
            "Search the original conversation lexically and return message references.",
            json!({"query":string,"limit":integer}),
            &["query"],
        ),
        definition(
            "history_read",
            "Recover a referenced original message and nearby history. At most 15 neighboring messages on each side.",
            json!({"message_id":string,"radius":integer}),
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
            json!({"action":{"type":"string","enum":["list","get","create","update","pause","resume","cancel"]},"id":string,"name":string,"content":string,"mode":{"type":"string","enum":["reminder","task"]},"kind":{"type":"string","enum":["once","interval","cron"]},"at":string,"interval_seconds":integer,"cron":string}),
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

pub fn validate(name: &str, args: &Value) -> Result<()> {
    let definition = catalog()
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
        validate(name, args)?;
        if cancel.is_cancelled() {
            bail!("Task cancelled");
        }
        match name {
            "files_list" | "file_read" | "files_search" | "file_write" | "file_edit" => {
                let runtime = self.clone();
                let workspace = settings.workspace.clone();
                let name = name.to_owned();
                let args = args.clone();
                tokio::task::spawn_blocking(move || runtime.files_execute(&workspace, &name, &args))
                    .await?
            }
            "command_run" => {
                self.processes
                    .run(
                        task,
                        s(args, "command")?,
                        std::path::Path::new(s(args, "cwd")?),
                        args["timeout_seconds"].as_u64().unwrap_or(120),
                        cancel,
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
            "memory_search" => Ok(json!(self.store.memories(
                s(args, "query")?,
                args["limit"].as_u64().unwrap_or(10) as usize
            )?)),
            "memory_manage" => {
                let result = match s(args, "action")? {
                    "get" => {
                        let id = s(args, "id")?;
                        Ok(json!(
                            self.store
                                .memories(id, 1)?
                                .into_iter()
                                .find(|m| m.id == id)
                                .context("Memory not found")?
                        ))
                    }
                    "add" => Ok(json!(self.store.save_memory(
                        None,
                        s(args, "title")?,
                        s(args, "content")?,
                        source
                    )?)),
                    "update" => Ok(json!(self.store.save_memory(
                        Some(s(args, "id")?),
                        s(args, "title")?,
                        s(args, "content")?,
                        source
                    )?)),
                    "delete" => {
                        self.store.delete_memory(s(args, "id")?)?;
                        Ok(json!({"deleted":args["id"]}))
                    }
                    _ => bail!("Unknown memory action"),
                };
                self.export_memory()?;
                result
            }
            "history_search" => Ok(json!(self.store.history_search(
                s(args, "query")?,
                args["limit"].as_u64().unwrap_or(10) as usize
            )?)),
            "history_read" => Ok(json!(self.store.history_read(
                s(args, "message_id")?,
                args["radius"].as_u64().unwrap_or(3) as usize
            )?)),
            "skill_load" => {
                let name = s(args, "name")?;
                let content = self.skill_load(name)?;
                let metadata = self.skills()?.into_iter().find(|s| s.name == name);
                Ok(json!({"skill":metadata,"content":content}))
            }
            "schedule_manage" => self.schedule_manage(args.clone()).await,
            "task_inspect" => {
                let id = args["task_id"].as_str().unwrap_or(task);
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
            _ => bail!("This tool requires permission dispatch"),
        }
    }
}
fn s<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .with_context(|| format!("Missing text argument: {key}"))
}
