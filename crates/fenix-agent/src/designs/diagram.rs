//! Agent-authored diagrams: stable semantic IDs, local validation and Mermaid output.
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashSet;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    pub version: u32,
    #[serde(default = "direction")]
    pub direction: String,
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub edges: Vec<Edge>,
    #[serde(default)]
    pub groups: Vec<Group>,
}
fn direction() -> String {
    "LR".into()
}
fn shape() -> String {
    "rounded".into()
}
fn style() -> String {
    "solid".into()
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub label: String,
    #[serde(default = "shape")]
    pub shape: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(default)]
    pub detail: String,
    #[serde(default)]
    pub references: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    pub id: String,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub label: String,
    #[serde(default = "style")]
    pub style: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub id: String,
    pub label: String,
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
fn label(text: &str) -> Result<()> {
    ensure!(
        !text.trim().is_empty() && text.len() <= 500,
        "Labels require 1–500 bytes"
    );
    Ok(())
}
// Mermaid entity escapes keep labels as data, including quotes, brackets and markup.
fn quoted(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '"' | '<' | '>' | '&' | '#' | '`' | '\\' | '\n' | '\r' => format!("#{};", c as u32),
            _ => c.to_string(),
        })
        .collect()
}
fn node_line(node: &Node) -> String {
    let (left, right) = match node.shape.as_str() {
        "rectangle" => ("[", "]"),
        "diamond" => ("{", "}"),
        "circle" => ("((", "))"),
        "database" => ("[(", ")]"),
        _ => ("(", ")"),
    };
    format!(
        "n_{}{left}\"{}\"{right}\n",
        ident(&node.id),
        quoted(&node.label)
    )
}
// Hex encoding prevents identifier collisions (a-b vs a_dash_b), as well as reserved words.
fn ident(id: &str) -> String {
    id.bytes().map(|b| format!("{b:02x}")).collect()
}
pub fn compile(value: &Value) -> Result<String> {
    let graph: Graph = serde_json::from_value(value.clone()).context("Invalid diagram graph")?;
    ensure!(graph.version == 1, "Unsupported diagram graph version");
    ensure!(
        ["LR", "RL", "TB", "BT"].contains(&graph.direction.as_str()),
        "Invalid diagram direction"
    );
    ensure!(
        !graph.nodes.is_empty() && graph.nodes.len() <= 200,
        "Expected 1–200 diagram nodes"
    );
    ensure!(
        graph.edges.len() <= 600 && graph.groups.len() <= 50,
        "Diagram is too large; split into views"
    );
    let mut ids = HashSet::new();
    for id in graph
        .nodes
        .iter()
        .map(|n| &n.id)
        .chain(graph.edges.iter().map(|e| &e.id))
        .chain(graph.groups.iter().map(|g| &g.id))
    {
        ensure!(
            valid_id(id) && ids.insert(id),
            "Invalid or duplicate diagram ID: {id}"
        );
    }
    let nodes: HashSet<_> = graph.nodes.iter().map(|n| n.id.as_str()).collect();
    let groups: HashSet<_> = graph.groups.iter().map(|g| g.id.as_str()).collect();
    for n in &graph.nodes {
        label(&n.label)?;
        for color in [&n.fill, &n.stroke, &n.color].into_iter().flatten() {
            ensure!(
                color.len() == 7
                    && color.starts_with('#')
                    && color[1..].bytes().all(|b| b.is_ascii_hexdigit()),
                "Node colors must use #RRGGBB"
            );
        }
        ensure!(
            ["rectangle", "rounded", "diamond", "circle", "database"].contains(&n.shape.as_str()),
            "Invalid shape for {}",
            n.id
        );
        ensure!(
            n.group.as_ref().is_none_or(|g| groups.contains(g.as_str())),
            "Unknown group for {}",
            n.id
        );
        ensure!(
            n.detail.len() <= 8000
                && n.references.len() <= 20
                && n.references.iter().all(|r| r.len() <= 1000),
            "Node detail/references too large"
        );
    }
    let mut out = format!("flowchart {}\n", graph.direction);
    let write_node = |out: &mut String, n: &Node| {
        out.push_str(&node_line(n));
        let styles: Vec<_> = [
            ("fill", &n.fill),
            ("stroke", &n.stroke),
            ("color", &n.color),
        ]
        .into_iter()
        .filter_map(|(key, value)| value.as_ref().map(|value| format!("{key}:{value}")))
        .collect();
        if !styles.is_empty() {
            out.push_str(&format!("style n_{} {}\n", ident(&n.id), styles.join(",")));
        }
    };
    for g in &graph.groups {
        label(&g.label)?;
        out.push_str(&format!(
            "subgraph g_{}[\"{}\"]\n",
            ident(&g.id),
            quoted(&g.label)
        ));
        let children: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.group.as_deref() == Some(&g.id))
            .collect();
        ensure!(!children.is_empty(), "Group {} is empty", g.id);
        for n in children {
            write_node(&mut out, n);
        }
        out.push_str("end\n");
    }
    for n in graph.nodes.iter().filter(|n| n.group.is_none()) {
        write_node(&mut out, n);
    }
    for e in &graph.edges {
        ensure!(
            nodes.contains(e.from.as_str()) && nodes.contains(e.to.as_str()),
            "Unknown endpoint in edge {}",
            e.id
        );
        ensure!(
            e.label.len() <= 500 && ["solid", "dashed"].contains(&e.style.as_str()),
            "Invalid edge {}",
            e.id
        );
        let arrow = if e.style == "dashed" { "-.->" } else { "-->" };
        let text = if e.label.is_empty() {
            String::new()
        } else {
            format!("|\"{}\"|", quoted(&e.label))
        };
        out.push_str(&format!(
            "n_{} {arrow}{text} n_{}\n",
            ident(&e.from),
            ident(&e.to)
        ));
    }
    Ok(out)
}

pub fn prepare(content: &mut Value) -> Result<()> {
    if let Some(graph) = content.get("graph") {
        content["source"] = json!(compile(graph)?);
        content["format"] = json!("fenix-graph-v1");
    }
    let source = content["source"]
        .as_str()
        .context("Diagram requires graph or Mermaid source")?;
    ensure!(
        !source.trim().is_empty() && source.len() <= 128_000,
        "Diagram source must contain 1–128000 bytes"
    );
    // Configuration and executable directives belong to the trusted renderer, never to model output.
    ensure!(
        !source.contains("%%{") && !source.trim_start().starts_with("---"),
        "Diagram configuration directives are not supported"
    );
    ensure!(
        !source
            .lines()
            .any(|line| line.trim_start().starts_with("click ")),
        "Diagram click directives are not supported"
    );
    Ok(())
}

/// Apply an atomic batch to semantic data. Final validation catches dangling edges/groups.
pub fn edit(content: &Value, operations: &Value) -> Result<Value> {
    let mut result = content.clone();
    ensure!(
        result["graph"].is_object(),
        "This diagram uses Mermaid source; revise it with design_publish"
    );
    let ops = operations
        .as_array()
        .context("operations must be an array")?;
    ensure!(
        !ops.is_empty() && ops.len() <= 100,
        "Expected 1–100 operations"
    );
    for op in ops {
        let name = op["op"].as_str().context("Operation name required")?;
        if name == "direction" {
            result["graph"]["direction"] = op["value"].clone();
            continue;
        }
        let (collection, remove) = match name {
            "put_node" => ("nodes", false),
            "put_edge" => ("edges", false),
            "put_group" => ("groups", false),
            "remove_node" => ("nodes", true),
            "remove_edge" => ("edges", true),
            "remove_group" => ("groups", true),
            _ => bail!("Unknown diagram operation: {name}"),
        };
        let id = if remove {
            &op["id"]
        } else {
            &op["value"]["id"]
        };
        let id = id.as_str().context("Element ID required")?;
        if result["graph"].get(collection).is_none() {
            result["graph"][collection] = json!([]);
        }
        let entries = result["graph"][collection]
            .as_array_mut()
            .context("Invalid graph collection")?;
        let position = entries.iter().position(|v| v["id"] == id);
        if remove {
            entries.remove(position.context("Element to remove does not exist")?);
        } else if let Some(i) = position {
            entries[i] = op["value"].clone();
        } else {
            entries.push(op["value"].clone());
        }
    }
    prepare(&mut result)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn example() -> Value {
        json!({"graph":{"version":1,"nodes":[{"id":"request","label":"Purchase request","group":"office"},{"id":"approve","label":"Approved","detail":"Keep this detail"}],"edges":[{"id":"approval","from":"request","to":"approve","label":"Accept"}],"groups":[{"id":"office","label":"Office"}]}})
    }
    #[test]
    fn structured_diagrams_validate_relationships_and_escape_labels() {
        let mut c = example();
        c["graph"]["nodes"][0]["label"] = json!("A \"quote\" <script> `text` #1");
        prepare(&mut c).unwrap();
        assert!(!c["source"].as_str().unwrap().contains("<script>"));
        assert!(c["source"].as_str().unwrap().contains("#34;"));
        let mut broken = c.clone();
        broken["graph"]["edges"][0]["to"] = json!("missing");
        assert!(prepare(&mut broken).is_err());
        broken = c.clone();
        broken["graph"]["nodes"][1]["id"] = json!("request");
        assert!(prepare(&mut broken).is_err());
        broken = c;
        broken["graph"]["nodes"][0]["group"] = json!("missing");
        assert!(prepare(&mut broken).is_err());
        broken = example();
        broken["graph"]["nodes"][0]["fill"] = json!("url(https://example.com)");
        assert!(prepare(&mut broken).is_err());
    }
    #[test]
    fn edits_are_atomic_and_preserve_unrelated_content() {
        let c = example();
        let edited = edit(&c, &json!([
            {"op":"put_node","value":{"id":"manager","label":"Manager"}},
            {"op":"put_edge","value":{"id":"escalation","from":"request","to":"manager","label":"Over S/5,000"}},
            {"op":"direction","value":"TB"}
        ])).unwrap();
        assert_eq!(edited["graph"]["nodes"][1], c["graph"]["nodes"][1]);
        assert_eq!(edited["graph"]["nodes"].as_array().unwrap().len(), 3);
        assert!(edit(&c, &json!([{"op":"remove_node","id":"approve"}])).is_err());
        assert_eq!(c["graph"]["nodes"].as_array().unwrap().len(), 2);
        assert!(
            edit(
                &c,
                &json!([
                    {"op":"remove_edge","id":"approval"}, {"op":"remove_node","id":"approve"}
                ])
            )
            .is_ok()
        );
    }
    #[test]
    fn mermaid_supports_other_types_without_allowing_renderer_configuration() {
        assert!(prepare(&mut json!({"source":"sequenceDiagram\n Alice->>Bob: Hello"})).is_ok());
        assert!(
            prepare(
                &mut json!({"source":"%%{init: {securityLevel: 'loose'}}%%\nflowchart LR\n A-->B"})
            )
            .is_err()
        );
        assert!(prepare(&mut json!({"source":"flowchart LR\n A-->B\nclick A callMe()"})).is_err());
    }
}
