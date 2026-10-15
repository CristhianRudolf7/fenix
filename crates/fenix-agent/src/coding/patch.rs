//! Exact-context multi-file patches. All edits are prepared before the first write.
use crate::{
    Runtime,
    files::{MAX_FILE_BYTES, read_text},
};
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

#[derive(Debug)]
struct Edit {
    path: String,
    kind: String,
    destination: Option<String>,
    lines: Vec<String>,
}
fn parse(args: &Value) -> Result<Vec<Edit>> {
    let patch = args["patch"].as_str().context("patch required")?;
    if patch.len() > 256_000 {
        bail!("Patch exceeds 256 KB");
    }
    let lines: Vec<_> = patch.lines().collect();
    if lines.first() != Some(&"*** Begin Patch") || lines.last() != Some(&"*** End Patch") {
        bail!("Expected Begin Patch / End Patch markers");
    }
    let mut edits: Vec<Edit> = Vec::new();
    for line in &lines[1..lines.len() - 1] {
        if let Some((kind, path)) = ["Add", "Update", "Delete"].iter().find_map(|kind| {
            line.strip_prefix(&format!("*** {kind} File: "))
                .map(|p| (*kind, p))
        }) {
            if path.is_empty() {
                bail!("Empty patch path");
            }
            edits.push(Edit {
                path: path.into(),
                kind: kind.into(),
                destination: None,
                lines: Vec::new(),
            });
        } else {
            let edit = edits
                .last_mut()
                .context("Patch content precedes a file marker")?;
            if let Some(destination) = line.strip_prefix("*** Move to: ") {
                if edit.kind != "Update"
                    || edit.destination.is_some()
                    || !edit.lines.is_empty()
                    || destination.is_empty()
                {
                    bail!("Move must follow an Update marker");
                }
                edit.destination = Some(destination.into());
            } else {
                edit.lines.push((*line).into());
            }
        }
    }
    if edits.is_empty() || edits.len() > 32 {
        bail!("Patch needs 1–32 files");
    }
    Ok(edits)
}
pub(crate) fn patch_paths(args: &Value) -> Result<Vec<String>> {
    Ok(parse(args)?
        .into_iter()
        .flat_map(|e| std::iter::once(e.path).chain(e.destination))
        .collect())
}
fn updated(before: &str, lines: &[String]) -> Result<String> {
    if lines.is_empty() {
        return Ok(before.into());
    }
    if lines.first().map(String::as_str) != Some("@@") {
        bail!("Update hunks must start with @@ on its own line");
    }
    let crlf = before.contains("\r\n");
    if crlf && before.replace("\r\n", "").contains('\n') {
        bail!("Mixed line endings: use exact file_edit to preserve untouched bytes");
    }
    let separator = if crlf { "\r\n" } else { "\n" };
    let trailing = before.ends_with('\n');
    let mut content: Vec<String> = before.lines().map(str::to_owned).collect();
    let mut cursor = 0;
    for (hunk_index, hunk) in lines[1..].split(|l| l == "@@").enumerate() {
        if hunk.is_empty() {
            bail!("Empty patch hunk");
        }
        let mut old = Vec::new();
        let mut new = Vec::new();
        for line in hunk {
            match line.as_bytes().first() {
                Some(b' ') => {
                    old.push(line[1..].to_owned());
                    new.push(line[1..].to_owned());
                }
                Some(b'-') => old.push(line[1..].to_owned()),
                Some(b'+') => new.push(line[1..].to_owned()),
                _ => bail!("Hunk lines must start with space, +, or -"),
            }
        }
        if old.is_empty() {
            bail!("Update needs existing context; use Add for a new file");
        }
        let positions: Vec<_> = content
            .windows(old.len())
            .enumerate()
            .filter(|(i, w)| *i >= cursor && *w == old.as_slice())
            .map(|(i, _)| i)
            .collect();
        if positions.len() != 1 {
            let candidates = positions
                .iter()
                .take(6)
                .map(|line| line + 1)
                .collect::<Vec<_>>();
            bail!(
                "Hunk {} has {} exact matches after line {}; candidate lines: {:?}. No files were changed during preflight. Reread this region: zero matches can mean an earlier edit already applied; multiple matches need distinguishing surrounding lines. Retry only the corrected patch, not an unchanged hunk.",
                hunk_index + 1,
                positions.len(),
                cursor,
                candidates
            );
        }
        let start = positions[0];
        cursor = start + new.len();
        content.splice(start..start + old.len(), new);
    }
    let mut after = content.join(separator);
    if trailing && !content.is_empty() {
        after.push_str(separator);
    }
    Ok(after)
}
struct Prepared {
    path: PathBuf,
    destination: PathBuf,
    before: Option<String>,
    after: Option<String>,
}
impl Runtime {
    pub(crate) fn apply_file_patch(&self, workspace: &Path, args: &Value) -> Result<Value> {
        let mut prepared = Vec::new();
        let mut seen = HashSet::new();
        for edit in parse(args)? {
            let path = self.file_path(workspace, &edit.path, edit.kind == "Add")?;
            let destination = match edit.destination {
                Some(ref p) => self.file_path(workspace, p, true)?,
                None => path.clone(),
            };
            if !seen.insert(path.clone())
                || destination != path && !seen.insert(destination.clone())
            {
                bail!("Patch paths overlap");
            }
            let before = if edit.kind == "Add" {
                if path.exists() {
                    bail!("Add target already exists");
                }
                None
            } else {
                Some(read_text(&path)?)
            };
            if destination != path && destination.exists() {
                bail!("Move destination already exists");
            }
            let after = match edit.kind.as_str() {
                "Add" => {
                    let mut text = String::new();
                    for line in edit.lines {
                        text.push_str(
                            line.strip_prefix('+')
                                .context("Add lines must start with +")?,
                        );
                        text.push('\n');
                    }
                    Some(text)
                }
                "Delete" => {
                    if !edit.lines.is_empty() {
                        bail!("Delete cannot contain hunks");
                    }
                    None
                }
                _ => Some(
                    updated(before.as_deref().unwrap(), &edit.lines)
                        .map_err(|error| anyhow::anyhow!("Patch target {}: {error}", edit.path))?,
                ),
            };
            if after
                .as_ref()
                .is_some_and(|s| s.len() > MAX_FILE_BYTES as usize)
            {
                bail!("Patched file exceeds 1 MiB");
            }
            if let Some(content) = &after {
                self.check_instruction_write(&destination, content)?;
            }
            prepared.push(Prepared {
                path,
                destination,
                before,
                after,
            });
        }
        // Reject stale content across the complete set before mutating any file.
        for edit in &prepared {
            if edit.before.is_some() && read_text(&edit.path)? != *edit.before.as_ref().unwrap() {
                bail!("File changed during patch preparation");
            }
        }
        let mut applied = Vec::new();
        for edit in prepared {
            let apply = || -> Result<()> {
                // Revalidate path boundaries and content under the runtime effect lease.
                self.file_path(
                    workspace,
                    &edit.path.to_string_lossy(),
                    edit.before.is_none(),
                )?;
                self.file_path(workspace, &edit.destination.to_string_lossy(), true)?;
                if let Some(before) = &edit.before
                    && read_text(&edit.path)? != *before
                {
                    bail!("File changed before patch write");
                }
                if let Some(after) = &edit.after {
                    if edit.before.is_none() || edit.destination != edit.path {
                        let permissions = if edit.before.is_some() {
                            Some(
                                fenix_execution::fs::open_user_file(&edit.path)?
                                    .metadata()?
                                    .permissions(),
                            )
                        } else {
                            None
                        };
                        fenix_execution::fs::create_workspace(
                            &edit.destination,
                            after.as_bytes(),
                            permissions,
                        )?;
                    } else {
                        fenix_execution::fs::replace_user_file(
                            &edit.destination,
                            edit.before
                                .as_ref()
                                .context("Missing original content")?
                                .as_bytes(),
                            after.as_bytes(),
                        )?;
                    }
                }
                if let Some(after) = &edit.after
                    && read_text(&edit.destination)? != *after
                {
                    bail!("Patched file did not match the prepared content");
                }
                if edit.after.is_none() || edit.destination != edit.path {
                    fenix_execution::fs::remove_user_file(
                        &edit.path,
                        edit.before
                            .as_ref()
                            .context("Missing original content")?
                            .as_bytes(),
                    )?;
                }
                Ok(())
            };
            if let Err(error) = apply() {
                // Filesystems do not provide a multi-file atomic transaction. Report evidence honestly.
                bail!(
                    "Patch stopped: {error}; completed files: {}; inspect current file {} and destination {} before retrying",
                    serde_json::to_string(&applied)?,
                    edit.path.display(),
                    edit.destination.display()
                );
            }
            applied.push(json!({"path":edit.path,"destination":edit.destination,"deleted":edit.after.is_none(),"before_sha256":edit.before.as_ref().map(|s|fenix_documents::sha256(s.as_bytes())),"sha256":edit.after.as_ref().map(|s|fenix_documents::sha256(s.as_bytes())),"bytes":edit.after.as_ref().map(String::len)}));
        }
        Ok(json!({"applied":applied,"verified":true}))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_hunks_reject_ambiguity_and_keep_newlines() {
        assert_eq!(
            updated(
                "a\nb\nc\n",
                &[
                    "@@".into(),
                    " a".into(),
                    "-b".into(),
                    "+B".into(),
                    " c".into()
                ]
            )
            .unwrap(),
            "a\nB\nc\n"
        );
        assert!(updated("x\nx\n", &["@@".into(), "-x".into(), "+y".into()]).is_err());
    }
    #[test]
    fn ambiguous_patch_reports_candidates_and_unique_context_recovers() {
        let before = "first\nx\nsecond\nx\n";
        let ambiguous = vec!["@@".into(), "-x".into(), "+y".into()];
        let error = updated(before, &ambiguous).unwrap_err().to_string();
        assert!(error.contains("2 exact matches"));
        assert!(error.contains("[2, 4]"));
        let fixed = vec!["@@".into(), " second".into(), "-x".into(), "+y".into()];
        let after = updated(before, &fixed).unwrap();
        assert_eq!(after, "first\nx\nsecond\ny\n");
        assert!(
            updated(&after, &fixed)
                .unwrap_err()
                .to_string()
                .contains("0 exact matches")
        );
    }

    #[test]
    fn edits_preserve_crlf_and_reject_mixed_endings() {
        let hunk = vec!["@@".into(), "-old".into(), "+new".into()];
        assert_eq!(updated("old\r\n", &hunk).unwrap(), "new\r\n");
        assert!(updated("old\r\nother\n", &hunk).is_err());
    }
    #[tokio::test]
    async fn multi_file_preflight_and_move() {
        let dir = tempfile::tempdir().unwrap();
        let ws = dir.path().join("ws");
        std::fs::create_dir(&ws).unwrap();
        let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
            .await
            .unwrap();
        std::fs::write(ws.join("a"), "old\n").unwrap();
        let bad = json!({"patch":"*** Begin Patch\n*** Update File: a\n@@\n-old\n+new\n*** Delete File: missing\n*** End Patch"});
        assert!(rt.apply_file_patch(&ws, &bad).is_err());
        assert_eq!(read_text(&ws.join("a")).unwrap(), "old\n");
        let good = json!({"patch":"*** Begin Patch\n*** Update File: a\n*** Move to: sub/b\n@@\n-old\n+new\n*** Add File: c\n+created\n*** End Patch"});
        rt.apply_file_patch(&ws, &good).unwrap();
        assert!(!ws.join("a").exists());
        assert_eq!(read_text(&ws.join("sub/b")).unwrap(), "new\n");
        assert!(
            rt.apply_file_patch(
                &ws,
                &json!({"patch":"*** Begin Patch\n*** Add File: ../escape\n+x\n*** End Patch"})
            )
            .is_ok()
        );
        assert_eq!(read_text(&dir.path().join("escape")).unwrap(), "x\n");
    }
}
