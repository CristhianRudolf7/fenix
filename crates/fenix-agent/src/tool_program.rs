//! Small declarative programs compose authorized tools without intermediate model turns.
use crate::{Runtime, config::Settings, workflows::model};
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use tokio_util::sync::CancellationToken;

pub(crate) fn parameters() -> Value {
    json!({
        "steps": {"type":"array", "minItems":1, "maxItems":50, "items":{
            "type":"object", "additionalProperties":false,
            "required":["id","tool","arguments"],
            "properties":{
                "id":{"type":"string","pattern":"^[A-Za-z0-9_]+$","maxLength":64},
                "tool":{"type":"string","pattern":"^[a-z][a-z0-9_]*$",
                    "description":"Exact bare tool name, e.g. file_read or command_run. Never functions.file_read. Only composable tools available in this task."},
                "arguments":{"type":"object","additionalProperties":true},
                "foreach":{}, "select":{},
                "when":{"type":"object","additionalProperties":false,
                    "required":["reference","operator"],"properties":{
                        "reference":{"type":"string"},
                        "operator":{"type":"string","enum":["equals","not_equals","contains","exists","count_equals"]},
                        "value":{}}}
            }
        }},
        "output":{}
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Program {
    steps: Vec<Step>,
    output: Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    id: String,
    tool: String,
    arguments: Value,
    #[serde(default)]
    when: Option<model::Predicate>,
    #[serde(default)]
    foreach: Option<Value>,
    #[serde(default)]
    select: Option<Value>,
}
impl Program {
    fn validate(&self) -> Result<()> {
        ensure!(
            !self.steps.is_empty() && self.steps.len() <= 50,
            "Use 1–50 steps"
        );
        let mut ids = BTreeSet::new();
        for step in &self.steps {
            ensure!(
                !step.id.is_empty()
                    && step.id != "item"
                    && step.id.len() <= 64
                    && step
                        .id
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_')
                    && !ids.contains(&step.id),
                "Invalid or duplicate step ID"
            );
            validate_references(&step.arguments, &ids, step.foreach.is_some())?;
            if let Some(value) = &step.foreach {
                validate_references(value, &ids, false)?;
            }
            if let Some(condition) = &step.when {
                ensure!(
                    ids.iter().any(|id: &&String| id.as_str()
                        == condition.reference.split('/').next().unwrap_or("")),
                    "Conditions must reference an earlier step"
                );
            }
            ids.insert(&step.id);
            if let Some(value) = &step.select {
                validate_references(value, &ids, step.foreach.is_some())?;
            }
            ensure!(
                model::allowed_tool(&step.tool),
                "Step '{}': '{}' is not composable. Use exact bare names such as file_read or command_run, without a functions. prefix. environment_inspect and skill_load require direct calls. No program steps have executed.",
                step.id,
                step.tool
            );
            ensure!(step.arguments.is_object(), "Arguments must be an object");
            ensure!(
                step.arguments["persistent"] != true && step.arguments["tty"] != true,
                "Programs cannot start persistent or interactive processes"
            );
        }
        validate_references(&self.output, &ids, false)?;
        Ok(())
    }
}
fn validate_references(value: &Value, known: &BTreeSet<&String>, item: bool) -> Result<()> {
    match value {
        Value::Object(object) if object.contains_key("$ref") => {
            ensure!(object.len() == 1, "A reference must contain only $ref");
            let reference = object["$ref"].as_str().context("Reference must be text")?;
            let source = reference.split('/').next().unwrap_or("");
            ensure!(
                (item && source == "item") || known.iter().any(|id| id.as_str() == source),
                "References must name an earlier step (or the current step in select)"
            );
        }
        Value::Object(object) => {
            for value in object.values() {
                validate_references(value, known, item)?;
            }
        }
        Value::Array(array) => {
            for value in array {
                validate_references(value, known, item)?;
            }
        }
        _ => {}
    }
    Ok(())
}
impl Runtime {
    pub(crate) async fn tool_program(
        self: &Arc<Self>,
        task: &str,
        settings: &Settings,
        args: &Value,
        cancel: &CancellationToken,
    ) -> Result<Value> {
        ensure!(args.to_string().len() <= 128_000, "Program exceeds 128 KB");
        let program: Program = serde_json::from_value(args.clone())?;
        program.validate()?;
        // Reject unavailable tools before executing any prefix of the program.
        let catalog = self.task_tool_catalog(task)?;
        for step in &program.steps {
            ensure!(
                catalog.iter().any(|t| t.name == step.tool),
                "Step '{}': tool '{}' is unavailable in this profile. No program steps have executed.",
                step.id,
                step.tool
            );
        }
        let id = fenix_storage::id();
        let mut outputs = BTreeMap::new();
        let mut calls = 0;
        for step in program.steps {
            if let Some(condition) = &step.when
                && !model::condition(condition, &outputs)?
            {
                outputs.insert(step.id, Value::Null);
                continue;
            }
            let looping = step.foreach.is_some();
            let items = if let Some(source) = step.foreach {
                model::resolve(&source, &outputs)?
                    .as_array()
                    .context("foreach must resolve to an array")?
                    .clone()
            } else {
                vec![Value::Null]
            };
            ensure!(
                calls + items.len() <= 50,
                "Program exceeds 50 tool calls; previous results remain in task_inspect"
            );
            let mut collected = Vec::new();
            for item in items {
                ensure!(!cancel.is_cancelled(), "Program cancelled");
                outputs.insert("item".into(), item);
                let input = model::resolve(&step.arguments, &outputs)?;
                ensure!(
                    input["persistent"] != true && input["tty"] != true,
                    "Interactive/background processes are unavailable in programs"
                );
                let call_id = format!("{id}-{}-{calls}", step.id);
                calls += 1;
                self.store.event(
                    task,
                    "tool_started",
                    &json!({"call_id":call_id,"tool":step.tool,"arguments":input,"program_id":id}),
                )?;
                if matches!(
                    step.tool.as_str(),
                    "file_write"
                        | "file_edit"
                        | "file_patch"
                        | "document_create"
                        | "document_edit"
                        | "library_export"
                ) {
                    let workspace = self.task_workspace(task, &settings.workspace)?;
                    let rules =
                        self.pending_repository_rules(task, &workspace, &step.tool, &input)?;
                    if !rules.is_empty() {
                        let receipt = json!({"executed":false,"repository_instructions":rules});
                        self.record_tool_outcome(
                            task,
                            &call_id,
                            &step.tool,
                            &input,
                            Ok(receipt.clone()),
                        )?;
                        return Ok(
                            json!({"program_id":id,"status":"paused","call_id":call_id,"executed":false,"repository_instructions":rules,"instruction":"Read this guidance; resume only the remaining steps. Earlier calls are saved in task_inspect."}),
                        );
                    }
                }
                let result = self
                    .workflow_tool(task, &step.tool, &input, settings, cancel)
                    .await;
                let (value, failed) =
                    self.record_tool_outcome(task, &call_id, &step.tool, &input, result)?;
                if failed {
                    return Ok(
                        json!({"program_id":id,"status":"failed","failed_call_id":call_id,"calls":calls,"error":value,"completed_steps":outputs.keys().filter(|k| k.as_str()!="item").collect::<Vec<_>>(),"instruction":"No automatic retries. Inspect saved output before resuming effects."}),
                    );
                }
                outputs.insert(step.id.clone(), value.clone());
                let selected = if let Some(selection) = &step.select {
                    model::resolve(selection, &outputs)?
                } else {
                    value
                };
                collected.push(selected);
                ensure!(
                    serde_json::to_vec(&collected)?.len() <= 2_000_000,
                    "Selected results exceed 2 MB; select smaller fields"
                );
            }
            outputs.remove("item");
            outputs.insert(
                step.id,
                if looping {
                    json!(collected)
                } else {
                    collected.pop().unwrap_or(Value::Null)
                },
            );
        }
        let mut output = model::resolve(&program.output, &outputs)?;
        self.redactor.value(&mut output);
        let text = serde_json::to_string(&output)?;
        self.store.event(
            task,
            "program_output",
            &json!({"program_id":id,"output":output}),
        )?;
        if text.len() > 24_000 {
            // Journal as a normal receipt too, so existing saved-output search can retrieve it.
            self.record_tool_outcome(task, &id, "tool_program", &json!({}), Ok(output))?;
            return Ok(
                json!({"program_id":id,"status":"completed","calls":calls,"truncated":true,"preview":crate::redaction::truncate(&text,24000),"call_id":id,"instruction":"Use task_inspect call_id to recover the full result."}),
            );
        }
        Ok(json!({"program_id":id,"status":"completed","calls":calls,"output":output}))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn invalid_later_step_does_not_execute_an_earlier_write() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let workspace = dir.path().join("workspace");
        std::fs::create_dir(&workspace)?;
        let runtime =
            Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
        let settings = Settings::initial(workspace.clone());
        let task = runtime.store.enqueue("Create a file", "web", None)?;
        let args = json!({"steps":[
            {"id":"write","tool":"file_write","arguments":{"path":"result.txt","content":"unexpected"}},
            {"id":"read","tool":"functions.file_read","arguments":{"path":"result.txt"}}
        ],"output":{"$ref":"read"}});
        let error = runtime
            .tool_program(&task.id, &settings, &args, &CancellationToken::new())
            .await
            .unwrap_err();
        assert!(error.to_string().contains("functions.file_read"));
        assert!(error.to_string().contains("No program steps have executed"));
        assert!(!workspace.join("result.txt").exists());
        Ok(())
    }

    #[test]
    fn programs_reject_recursion_and_duplicate_ids() {
        for tool in ["tool_program", "subagent_spawn", "request_approval"] {
            let p: Program = serde_json::from_value(
                json!({"steps":[{"id":"a","tool":tool,"arguments":{}}],"output":null}),
            )
            .unwrap();
            assert!(p.validate().is_err());
        }
        let p: Program = serde_json::from_value(json!({"steps":[{"id":"a","tool":"file_read","arguments":{}},{"id":"a","tool":"file_read","arguments":{}}],"output":null})).unwrap();
        assert!(p.validate().is_err());
    }
}
