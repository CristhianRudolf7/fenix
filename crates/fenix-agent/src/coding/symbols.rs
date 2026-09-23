//! File-scoped symbol navigation over ordinary LSP document symbols.
//! No project index, executable edits, or additional model calls are required.
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;

impl crate::Runtime {
    /// Optional evidence beside LSP locations. Resolve through exactly the same
    /// account/worktree/private-file policy as file_read, never through raw URIs.
    pub(crate) fn enrich_code_locations(&self, workspace: &Path, response: &mut Value) {
        let Some(locations) = response["result"].as_array() else {
            return;
        };
        let mut contexts = Vec::new();
        for location in locations.iter().take(20) {
            let uri = location["uri"].as_str().or(location["targetUri"].as_str());
            let range = location
                .get("range")
                .or(location.get("targetSelectionRange"));
            let context = (|| -> Result<Value> {
                let url = url::Url::parse(uri.context("Missing location URI")?)?;
                let path = url.to_file_path().map_err(|_| {
                    anyhow::anyhow!("Only local workspace file locations can have excerpts")
                })?;
                let path =
                    self.file_path(workspace, path.to_str().context("Non-UTF-8 path")?, false)?;
                let text = crate::files::read_text(&path)?;
                let line = range.context("Missing location range")?["start"]["line"]
                    .as_u64()
                    .context("Missing location line")?;
                if line >= text.split('\n').count() as u64 {
                    bail!("Location is outside the current file; rerun the query");
                }
                let start = line.saturating_sub(1) as usize;
                let excerpt = text
                    .split('\n')
                    .skip(start)
                    .take(5)
                    .collect::<Vec<_>>()
                    .join("\n");
                let mut cut = excerpt.len().min(1000);
                while !excerpt.is_char_boundary(cut) {
                    cut -= 1;
                }
                Ok(
                    json!({"status":"available","path":path.strip_prefix(workspace).unwrap_or(&path),
                    "start_line":start+1,"excerpt":&excerpt[..cut],"excerpt_truncated":cut < excerpt.len(),
                    "file_sha256":format!("{:x}", Sha256::digest(text.as_bytes()))}),
                )
            })();
            contexts.push(context.unwrap_or_else(
                |error| json!({"status":"unavailable","reason":error.to_string()}),
            ));
        }
        let omitted = locations.len().saturating_sub(contexts.len());
        response["contexts"] = json!(contexts);
        response["contexts_omitted"] = json!(omitted);
    }
}

#[derive(Debug)]
pub(super) struct Symbol {
    pub name_path: String,
    pub kind: Value,
    pub range: Value,
    pub selection: Value,
}

pub(super) fn flatten(raw: &Value, uri: &str) -> Vec<Symbol> {
    fn walk(items: &[Value], parent: &str, uri: &str, out: &mut Vec<Symbol>) {
        for item in items {
            let Some(name) = item["name"].as_str() else {
                continue;
            };
            // SymbolInformation can refer to another document. Never interpret
            // that range against the requested file or read a server-supplied path.
            if item["location"]["uri"].as_str().is_some_and(|u| u != uri) {
                continue;
            }
            let container = if parent.is_empty() {
                item["containerName"].as_str().unwrap_or("")
            } else {
                parent
            };
            let name_path = if container.is_empty() {
                name.into()
            } else {
                format!("{container}/{name}")
            };
            let range = item
                .get("range")
                .unwrap_or(&item["location"]["range"])
                .clone();
            let selection = item.get("selectionRange").unwrap_or(&range).clone();
            out.push(Symbol {
                name_path: name_path.clone(),
                kind: item["kind"].clone(),
                range,
                selection,
            });
            if let Some(children) = item["children"].as_array() {
                walk(children, &name_path, uri, out);
            }
        }
    }
    let mut out = Vec::new();
    if let Some(items) = raw.as_array() {
        walk(items, "", uri, &mut out);
    }
    out
}

fn matches(symbol: &Symbol, pattern: &str) -> bool {
    if let Some(exact) = pattern.strip_prefix('/') {
        symbol.name_path == exact
    } else {
        symbol.name_path == pattern || symbol.name_path.ends_with(&format!("/{pattern}"))
    }
}

fn pattern(args: &Value) -> Result<&str> {
    let pattern = args["name_path"]
        .as_str()
        .filter(|p| !p.is_empty() && *p != "/")
        .context("A non-empty name_path is required, for example Class/method or /Class/method")?;
    Ok(pattern)
}

pub(super) fn unique<'a>(symbols: &'a [Symbol], args: &Value) -> Result<&'a Symbol> {
    let pattern = pattern(args)?;
    let mut candidates = symbols.iter().filter(|s| matches(s, pattern));
    let symbol = candidates
        .next()
        .context("No symbol matches name_path in this file")?;
    if candidates.next().is_some() {
        bail!(
            "Ambiguous name_path; inspect find_symbol results and use a full path or explicit line/character for overloaded symbols"
        );
    }
    Ok(symbol)
}

/// Translate LSP UTF-16 coordinates into byte offsets, rejecting half-surrogates
/// and positions outside a line instead of silently selecting the wrong text.
pub(super) fn offset(text: &str, position: &Value) -> Result<usize> {
    let line = position["line"].as_u64().context("Missing LSP line")?;
    let character = position["character"]
        .as_u64()
        .context("Missing LSP character")?;
    let mut start = 0;
    let mut lines = text.split('\n');
    for _ in 0..line {
        start += lines.next().context("LSP line outside document")?.len() + 1;
    }
    let line = lines.next().context("LSP line outside document")?;
    let line = line.strip_suffix('\r').unwrap_or(line);
    let mut units = 0;
    for (byte, c) in line.char_indices() {
        if units == character {
            return Ok(start + byte);
        }
        units += c.len_utf16() as u64;
        if units > character {
            bail!("LSP position splits a UTF-16 surrogate pair");
        }
    }
    if units == character {
        Ok(start + line.len())
    } else {
        bail!("LSP character outside line")
    }
}

pub(super) fn selection(symbol: &Symbol, text: &str) -> Result<Value> {
    offset(text, &symbol.selection["start"])?;
    Ok(symbol.selection["start"].clone())
}

pub(super) fn response(raw: &Value, uri: &str, text: &str, args: &Value) -> Result<Value> {
    let symbols = flatten(raw, uri);
    let operation = args["operation"].as_str().unwrap_or("");
    let hash = format!("{:x}", Sha256::digest(text.as_bytes()));
    if operation == "read_symbol" {
        let symbol = unique(&symbols, args)?;
        let start = offset(text, &symbol.range["start"])?;
        let end = offset(text, &symbol.range["end"])?;
        let body = text.get(start..end).context("Invalid LSP symbol range")?;
        let limit = args["max_bytes"]
            .as_u64()
            .unwrap_or(16_384)
            .clamp(1, 32_768) as usize;
        let mut cut = body.len().min(limit);
        while !body.is_char_boundary(cut) {
            cut -= 1;
        }
        return Ok(
            json!({"name_path":symbol.name_path,"kind":symbol.kind,"range":symbol.range,
            "selection_range":symbol.selection,"body":&body[..cut],"body_bytes":body.len(),
            "truncated":cut < body.len(),"file_sha256":hash,
            "instruction":"Ranges are zero-based UTF-16. Use existing file tools for edits; truncated bodies are not complete edit context."}),
        );
    }
    let pattern = if operation == "find_symbol" {
        Some(pattern(args)?)
    } else {
        None
    };
    let matches: Vec<_> = symbols
        .iter()
        .filter(|s| pattern.is_none_or(|p| matches(s, p)))
        .collect();
    let offset = args["offset"].as_u64().unwrap_or(0).min(usize::MAX as u64) as usize;
    let limit = args["limit"].as_u64().unwrap_or(50).clamp(1, 100) as usize;
    let items: Vec<_> = matches
        .iter()
        .skip(offset)
        .take(limit)
        .map(|s| {
            json!({
                "name_path":s.name_path,"kind":s.kind,"range":s.range,"selection_range":s.selection,
            })
        })
        .collect();
    let next = offset.saturating_add(items.len());
    Ok(
        json!({"symbols":items,"total":matches.len(),"offset":offset,
        "next_offset":if next < matches.len() {Some(next)} else {None},"file_sha256":hash}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn range(start: u64, end: u64) -> Value {
        json!({"start":{"line":0,"character":start},"end":{"line":0,"character":end}})
    }
    #[tokio::test]
    async fn reference_excerpts_enforce_workspace_and_private_file_boundaries() {
        let dir = tempfile::tempdir().unwrap();
        let workspace = dir.path().join("workspace");
        std::fs::create_dir(&workspace).unwrap();
        let runtime = crate::Runtime::open(
            &dir.path().join("private"),
            Some(&dir.path().join("skills")),
        )
        .await
        .unwrap();
        let source = workspace.join("a.rs");
        let secret = workspace.join(".env");
        let other_user = dir.path().join("other-user.rs");
        std::fs::write(&source, "fn run() {}\nrun();\n").unwrap();
        std::fs::write(&secret, "PRIVATE_VALUE").unwrap();
        std::fs::write(&other_user, "OTHER_USER_VALUE").unwrap();
        std::os::unix::fs::symlink(&other_user, workspace.join("escape.rs")).unwrap();
        let locations: Vec<_> = [&source, &secret, &other_user, &workspace.join("escape.rs")]
            .iter()
            .map(|p| {
                json!({
                    "uri":url::Url::from_file_path(p).unwrap().to_string(),"range":range(0,1)
                })
            })
            .collect();
        let mut response = json!({"result":locations});
        runtime.enrich_code_locations(&workspace, &mut response);
        assert_eq!(response["contexts"][0]["status"], "available");
        assert_eq!(response["contexts"][0]["start_line"], 1);
        for index in 1..4 {
            assert_eq!(response["contexts"][index]["status"], "unavailable");
        }
        assert!(!response.to_string().contains("PRIVATE_VALUE"));
        assert!(!response.to_string().contains("OTHER_USER_VALUE"));
        let mut bounded = json!({"result":vec![response["result"][0].clone();25]});
        runtime.enrich_code_locations(&workspace, &mut bounded);
        assert_eq!(bounded["contexts"].as_array().unwrap().len(), 20);
        assert_eq!(bounded["contexts_omitted"], 5);
        runtime.shutdown().await.unwrap();
    }
    #[test]
    fn hierarchy_pagination_ambiguity_and_flat_locations() {
        let raw = json!([
            {"name":"A","kind":5,"range":range(0,4),"children":[{"name":"run","kind":6,"range":range(0,2)}]},
            {"name":"B","kind":5,"range":range(0,4),"children":[{"name":"run","kind":6,"range":range(2,4)}]},
            {"name":"outside","location":{"uri":"file:///outside","range":range(0,4)}}
        ]);
        let all = flatten(&raw, "file:///test");
        assert_eq!(all.len(), 4);
        assert!(unique(&all, &json!({"name_path":"run"})).is_err());
        assert!(unique(&all, &json!({"name_path":"un"})).is_err());
        assert_eq!(
            unique(&all, &json!({"name_path":"/A/run"}))
                .unwrap()
                .name_path,
            "A/run"
        );
        let page = response(
            &raw,
            "file:///test",
            "abcd",
            &json!({"operation":"find_symbol","name_path":"run","limit":1}),
        )
        .unwrap();
        assert_eq!(page["total"], 2);
        assert_eq!(page["next_offset"], 1);
        let flat = flatten(
            &json!([{"name":"method","containerName":"Class","location":{"uri":"file:///test","range":range(0,4)}}]),
            "file:///test",
        );
        assert_eq!(flat[0].name_path, "Class/method");
    }
    #[test]
    fn unicode_crlf_ranges_and_bounded_bodies() {
        let text = "a🦀b\r\nnext\n";
        assert_eq!(offset(text, &json!({"line":0,"character":3})).unwrap(), 5);
        assert!(offset(text, &json!({"line":0,"character":2})).is_err());
        assert!(offset(text, &json!({"line":0,"character":5})).is_err());
        assert_eq!(offset(text, &json!({"line":1,"character":0})).unwrap(), 8);
        assert_eq!(offset(text, &json!({"line":2,"character":0})).unwrap(), 13);
        assert!(offset(text, &json!({"line":3,"character":0})).is_err());
        let raw = json!([{"name":"x","range":range(1,3)}]);
        let body = response(
            &raw,
            "u",
            text,
            &json!({"operation":"read_symbol","name_path":"x"}),
        )
        .unwrap();
        assert_eq!(body["body"], "🦀");
        let short = response(
            &raw,
            "u",
            text,
            &json!({"operation":"read_symbol","name_path":"x","max_bytes":3}),
        )
        .unwrap();
        assert_eq!(short["body"], "");
        assert_eq!(short["truncated"], true);
        assert!(
            response(
                &json!([{"name":"x","range":range(3,1)}]),
                "u",
                text,
                &json!({"operation":"read_symbol","name_path":"x"})
            )
            .is_err()
        );
    }
}
