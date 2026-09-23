use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowInput {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub project_id: Option<String>,
    pub nodes: Vec<Node>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub version: u64,
    #[serde(flatten)]
    pub spec: WorkflowInput,
    pub updated_at: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub kind: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub arguments: Value,
    #[serde(default)]
    pub when: Option<Predicate>,
    #[serde(default)]
    pub steps: Vec<Action>,
    #[serde(default = "one")]
    pub attempts: u8,
}
fn one() -> u8 {
    1
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub id: String,
    pub tool: String,
    pub arguments: Value,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Predicate {
    pub reference: String,
    pub operator: String,
    #[serde(default)]
    pub value: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeRun {
    pub status: String,
    pub attempts: u8,
    pub output: Value,
    pub error: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}
impl Default for NodeRun {
    fn default() -> Self {
        Self {
            status: "pending".into(),
            attempts: 0,
            output: Value::Null,
            error: None,
            started_at: None,
            finished_at: None,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub task_id: String,
    pub workflow: Workflow,
    pub status: String,
    pub nodes: BTreeMap<String, NodeRun>,
    pub created_at: String,
    pub updated_at: String,
    pub test_mode: bool,
    pub error: Option<String>,
}

pub fn reference<'a>(outputs: &'a BTreeMap<String, Value>, value: &str) -> Result<&'a Value> {
    let (node, pointer) = value.split_once('/').unwrap_or((value, ""));
    let output = outputs
        .get(node)
        .with_context(|| format!("Step {node} has no output"))?;
    if pointer.is_empty() {
        Ok(output)
    } else {
        output
            .pointer(&format!("/{pointer}"))
            .with_context(|| format!("Missing step output: {value}"))
    }
}
pub fn resolve(value: &Value, outputs: &BTreeMap<String, Value>) -> Result<Value> {
    match value {
        Value::Object(object) if object.contains_key("$ref") => {
            ensure!(object.len() == 1, "A data reference must contain only $ref");
            Ok(reference(
                outputs,
                object["$ref"]
                    .as_str()
                    .context("Reference must be a string")?,
            )?
            .clone())
        }
        Value::Object(object) => Ok(Value::Object(
            object
                .iter()
                .map(|(k, v)| Ok((k.clone(), resolve(v, outputs)?)))
                .collect::<Result<_>>()?,
        )),
        Value::Array(items) => Ok(Value::Array(
            items
                .iter()
                .map(|v| resolve(v, outputs))
                .collect::<Result<_>>()?,
        )),
        _ => Ok(value.clone()),
    }
}
pub fn condition(condition: &Predicate, outputs: &BTreeMap<String, Value>) -> Result<bool> {
    let found = reference(outputs, &condition.reference);
    if condition.operator == "exists" {
        return Ok(found.is_ok_and(|v| !v.is_null()));
    }
    let found = found?;
    Ok(match condition.operator.as_str() {
        "equals" => found == &condition.value,
        "not_equals" => found != &condition.value,
        "contains" => {
            found
                .as_str()
                .zip(condition.value.as_str())
                .is_some_and(|(a, b)| a.contains(b))
                || found
                    .as_array()
                    .is_some_and(|a| a.contains(&condition.value))
        }
        "count_equals" => found
            .as_array()
            .zip(condition.value.as_u64())
            .is_some_and(|(a, b)| a.len() as u64 == b),
        _ => anyhow::bail!("Unknown condition operator"),
    })
}

impl WorkflowInput {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            !self.name.trim().is_empty() && self.name.len() <= 200,
            "Supply a workflow name"
        );
        ensure!(
            !self.nodes.is_empty() && self.nodes.len() <= 200,
            "A workflow needs 1–200 steps"
        );
        ensure!(
            serde_json::to_vec(self)?.len() <= 512_000,
            "Workflow definition is too large"
        );
        let mut seen = BTreeSet::new();
        for node in &self.nodes {
            ensure!(
                !node.id.is_empty()
                    && node.id.len() <= 80
                    && node
                        .id
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c)),
                "Step IDs use letters, digits, underscores or hyphens"
            );
            ensure!(
                seen.insert(node.id.clone()),
                "Duplicate step ID: {}",
                node.id
            );
            ensure!(!node.label.trim().is_empty(), "Every step needs a label");
            ensure!(
                [
                    "tool",
                    "connector",
                    "browser",
                    "script",
                    "prompt",
                    "condition",
                    "wait",
                    "notify"
                ]
                .contains(&node.kind.as_str()),
                "Unknown step type"
            );
            ensure!(
                (1..=3).contains(&node.attempts),
                "Use one to three attempts"
            );
            if node.kind == "tool" {
                ensure!(
                    node.tool.as_deref().is_some_and(allowed_tool),
                    "This tool is unavailable in workflows"
                );
            }
            if node.kind == "browser" {
                ensure!(
                    !node.steps.is_empty() && node.steps.len() <= 40,
                    "A browser step needs 1–40 actions"
                );
                let mut ids = BTreeSet::new();
                for action in &node.steps {
                    ensure!(
                        action.tool.starts_with("browser_") && allowed_tool(&action.tool),
                        "Browser steps only use browser tools"
                    );
                    ensure!(
                        !action.id.is_empty() && ids.insert(&action.id),
                        "Browser action IDs must be unique"
                    );
                }
            }
            if let Some(predicate) = &node.when {
                validate_predicate(predicate)?;
            }
        }
        let mut completed = BTreeSet::new();
        while completed.len() < self.nodes.len() {
            let before = completed.len();
            for node in &self.nodes {
                ensure!(
                    node.depends_on.iter().all(|id| seen.contains(id)),
                    "Unknown dependency"
                );
                if node.depends_on.iter().all(|id| completed.contains(id)) {
                    completed.insert(node.id.clone());
                }
            }
            ensure!(
                completed.len() > before,
                "Workflow dependencies contain a cycle"
            );
        }
        for node in &self.nodes {
            let mut ancestors = BTreeSet::new();
            let mut frontier = node.depends_on.clone();
            while let Some(id) = frontier.pop() {
                if ancestors.insert(id.clone()) {
                    frontier.extend(
                        self.nodes
                            .iter()
                            .find(|n| n.id == id)
                            .unwrap()
                            .depends_on
                            .clone(),
                    );
                }
            }
            validate_refs(&node.arguments, &ancestors)?;
            if node.kind == "condition" {
                let p: Predicate = serde_json::from_value(node.arguments.clone())?;
                validate_predicate(&p)?;
                ensure!(
                    ancestors.contains(p.reference.split('/').next().unwrap_or("")),
                    "Conditions can only read dependency outputs"
                );
            }
            if let Some(p) = &node.when {
                ensure!(
                    ancestors.contains(p.reference.split('/').next().unwrap_or("")),
                    "Conditions can only read dependency outputs"
                );
            }
            for action in &node.steps {
                validate_refs(&action.arguments, &ancestors)?;
                ancestors.insert(action.id.clone());
            }
        }
        Ok(())
    }
}
fn validate_predicate(p: &Predicate) -> Result<()> {
    ensure!(
        ["equals", "not_equals", "contains", "exists", "count_equals"]
            .contains(&p.operator.as_str()),
        "Unknown condition operator"
    );
    Ok(())
}
fn validate_refs(value: &Value, allowed: &BTreeSet<String>) -> Result<()> {
    match value {
        Value::Object(o) => {
            if let Some(v) = o.get("$ref") {
                ensure!(
                    o.len() == 1
                        && v.as_str()
                            .is_some_and(|s| allowed.contains(s.split('/').next().unwrap_or(""))),
                    "Data references must point to a dependency or an earlier browser action"
                );
            } else {
                for v in o.values() {
                    validate_refs(v, allowed)?;
                }
            }
        }
        Value::Array(a) => {
            for v in a {
                validate_refs(v, allowed)?;
            }
        }
        _ => {}
    }
    Ok(())
}
pub(crate) fn allowed_tool(name: &str) -> bool {
    (name.starts_with("browser_") && !matches!(name, "browser_connection"))
        || matches!(
            name,
            "file_read"
                | "file_write"
                | "file_edit"
                | "file_patch"
                | "files_list"
                | "files_search"
                | "files_glob"
                | "document_read"
                | "document_extract"
                | "document_create"
                | "document_edit"
                | "web_search"
                | "web_read"
                | "library_export"
                | "command_run"
                | "command_status"
                | "connector_call"
        )
}
pub(crate) fn read_only(node: &Node) -> bool {
    match node.kind.as_str() {
        "condition" => true,
        "connector" => matches!(
            node.arguments["operation"].as_str(),
            Some(
                "test"
                    | "drive_search"
                    | "mail_list"
                    | "calendar_list"
                    | "files_search"
                    | "repositories"
                    | "issues"
                    | "channels"
                    | "history"
            )
        ),
        "tool" => matches!(
            node.tool.as_deref(),
            Some(
                "file_read"
                    | "files_list"
                    | "files_search"
                    | "files_glob"
                    | "document_read"
                    | "web_search"
                    | "web_read"
                    | "command_status"
            )
        ),
        _ => false,
    }
}
