use crate::{
    Runtime,
    files::{MAX_FILE_BYTES, read_text},
    redaction::truncate,
};
use anyhow::{Context, Result, bail};
use globset::{GlobBuilder, GlobMatcher};
use regex::RegexBuilder;
use serde_json::{Value, json};
use std::path::Path;

fn glob(pattern: &str) -> Result<GlobMatcher> {
    Ok(GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()?
        .compile_matcher())
}
impl Runtime {
    pub(crate) fn search_files(&self, workspace: &Path, name: &str, args: &Value) -> Result<Value> {
        let root = self.file_path(workspace, args["path"].as_str().unwrap_or("."), false)?;
        let pattern = args["glob"].as_str().map(glob).transpose()?;
        let exclude = args["exclude"].as_str().map(glob).transpose()?;
        let limit = args["limit"].as_u64().unwrap_or(100).clamp(1, 500) as usize;
        let regex = if name == "files_search" {
            let query = args["query"].as_str().context("Search query required")?;
            if query.is_empty() || query.chars().count() > 4096 {
                bail!("Query must contain 1–4096 characters");
            }
            let expression = if args["regex"].as_bool().unwrap_or(false) {
                query.to_owned()
            } else {
                regex::escape(query)
            };
            Some(
                RegexBuilder::new(&expression)
                    .case_insensitive(!args["case_sensitive"].as_bool().unwrap_or(false))
                    .size_limit(1_000_000)
                    .build()?,
            )
        } else {
            None
        };
        let mut builder = ignore::WalkBuilder::new(workspace);
        builder
            .follow_links(false)
            .hidden(false)
            .require_git(false)
            .git_global(false)
            .parents(false)
            .max_depth(Some(32))
            .max_filesize(Some(MAX_FILE_BYTES))
            .sort_by_file_path(|a, b| a.cmp(b));
        // Keep private/generated directories excluded even when ignore files negate them.
        let private = self.data_dir.clone();
        let search_root = root.clone();
        builder.filter_entry(move |e| {
            (e.path().starts_with(&search_root) || search_root.starts_with(e.path()))
                && !e.path().starts_with(&private)
                && !matches!(
                    e.file_name().to_str(),
                    Some(".git" | "node_modules" | "target" | ".fenix" | "credentials" | "secrets")
                )
        });
        let mut matches = Vec::new();
        let mut scanned = 0;
        let mut visited = 0;
        let mut errors = 0;
        let mut truncated = false;
        let mut bytes = 0;
        for entry in builder.build() {
            visited += 1;
            if visited > 20_000 || bytes > 64 * 1024 * 1024 {
                truncated = true;
                break;
            }
            let entry = match entry {
                Ok(e) => e,
                Err(_) => {
                    errors += 1;
                    continue;
                }
            };
            if !entry.file_type().is_some_and(|t| t.is_file()) {
                continue;
            }
            let relative = entry.path().strip_prefix(&root).unwrap_or(entry.path());
            if pattern.as_ref().is_some_and(|p| !p.is_match(relative))
                || exclude.as_ref().is_some_and(|p| p.is_match(relative))
            {
                continue;
            }
            let Ok(path) = self.file_path(workspace, &entry.path().to_string_lossy(), false) else {
                continue;
            };
            scanned += 1;
            if let Some(regex) = &regex {
                let text = match read_text(&path) {
                    Ok(t) => t,
                    Err(_) => {
                        errors += 1;
                        continue;
                    }
                };
                bytes += text.len();
                for (i, line) in text.lines().enumerate() {
                    if regex.is_match(line) {
                        matches.push(json!({"path":path,"line":i+1,"text":self.redactor.text(&truncate(line,1000))}));
                        if matches.len() > limit {
                            break;
                        }
                    }
                }
            } else {
                matches.push(json!({"path":path}));
            }
            if matches.len() > limit {
                truncated = true;
                matches.truncate(limit);
                break;
            }
        }
        Ok(
            json!({"matches":matches,"files_scanned":scanned,"limit":limit,"truncated":truncated,"skipped_errors":errors,"ignore_files":true,"max_entries":20000,"max_depth":32,"max_scan_bytes":67108864}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn regex_globs_ignore_files_and_private_paths() {
        let dir = tempfile::tempdir().unwrap();
        let ws = dir.path().join("ws");
        std::fs::create_dir_all(ws.join("src")).unwrap();
        let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
            .await
            .unwrap();
        for (name, text) in [
            ("src/a.rs", "fn first() {}\nfn second() {}\n"),
            ("src/b.py", "def first(): pass"),
            ("ignored.rs", "fn hidden() {}"),
            (".env", "fn secret() {}"),
            (".gitignore", "ignored.rs\n"),
        ] {
            std::fs::write(ws.join(name), text).unwrap();
        }
        let r = rt
            .search_files(
                &ws,
                "files_search",
                &json!({"query":"^fn ","regex":true,"glob":"**/*.rs","limit":1}),
            )
            .unwrap();
        assert_eq!(r["matches"].as_array().unwrap().len(), 1);
        assert_eq!(r["truncated"], true);
        let paths = rt
            .search_files(&ws, "files_glob", &json!({"glob":"**/*.rs"}))
            .unwrap();
        assert_eq!(paths["matches"].as_array().unwrap().len(), 1);
        let excluded = rt
            .search_files(
                &ws,
                "files_glob",
                &json!({"glob":"**/*","exclude":"src/**"}),
            )
            .unwrap();
        assert!(!excluded.to_string().contains("secret"));
        assert!(
            rt.search_files(&ws, "files_search", &json!({"query":"[","regex":true}))
                .is_err()
        );
        assert!(
            rt.search_files(&ws, "files_glob", &json!({"path":"../","glob":"**/*"}))
                .is_err()
        );
    }
}
