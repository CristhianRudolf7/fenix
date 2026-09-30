use crate::{Runtime, redaction::truncate};
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::{
    io::Read,
    path::{Component, Path, PathBuf},
};

pub(crate) const MAX_FILE_BYTES: u64 = 1_048_576;

impl Runtime {
    pub(crate) fn file_path(&self, workspace: &Path, path: &str, create: bool) -> Result<PathBuf> {
        let requested = Path::new(path);
        if requested
            .components()
            .any(|p| matches!(p, Component::ParentDir))
        {
            bail!("Parent traversal is not allowed in file tools");
        }
        let joined = if requested.is_absolute() {
            requested.to_owned()
        } else {
            workspace.join(requested)
        };
        let mut existing = joined.as_path();
        let mut tail = vec![];
        while !existing.exists() && create {
            tail.push(existing.file_name().context("Invalid path")?.to_owned());
            existing = existing.parent().context("Invalid parent directory")?;
        }
        let mut resolved = std::fs::canonicalize(existing)?;
        for name in tail.into_iter().rev() {
            resolved.push(name);
        }
        if !resolved.starts_with(workspace) || resolved.starts_with(&self.data_dir) {
            bail!(
                "File tool path is outside the configured file workspace or inside private runtime data"
            );
        }
        if resolved.components().any(|part| {
            let text = part.as_os_str().to_string_lossy();
            text == ".git"
                || text == ".env"
                || text.starts_with(".env.")
                || matches!(text.as_ref(), "credentials" | "secrets")
                || text.ends_with(".pem")
        }) {
            bail!("File tools do not access credential files or Git internals");
        }
        Ok(resolved)
    }

    pub(crate) fn files_execute(
        &self,
        workspace: &Path,
        name: &str,
        args: &Value,
    ) -> Result<Value> {
        if name == "library_export" {
            let doc = self.store.library_document(
                args["id"].as_str().context("Document id required")?,
                args["revision"].as_u64(),
            )?;
            let path = self.file_path(
                workspace,
                args["path"].as_str().context("Destination path required")?,
                true,
            )?;
            let bytes = self.library_original(&doc)?;
            fenix_execution::fs::create_workspace(&path, &bytes, None)?;
            return Ok(
                json!({"path":path,"sha256":doc.sha256,"document_id":doc.id,"revision":doc.revision,"bytes":bytes.len(),"verified":true}),
            );
        }
        if name.starts_with("document_") {
            return self.documents_execute(workspace, name, args);
        }
        if matches!(name, "files_search" | "files_glob") {
            return self.search_files(workspace, name, args);
        }
        if name == "file_patch" {
            return self.apply_file_patch(workspace, args);
        }
        let path = args["path"].as_str().unwrap_or(".");
        let creating = name == "file_write";
        let path = self.file_path(workspace, path, creating)?;
        match name {
            "files_list" => {
                let mut entries = vec![];
                use std::os::fd::AsRawFd;
                let directory = fenix_execution::fs::directory(&path)?;
                for entry in
                    std::fs::read_dir(format!("/proc/self/fd/{}", directory.as_raw_fd()))?.take(200)
                {
                    let entry = entry?;
                    let kind = entry.file_type()?;
                    entries.push(json!({"name":entry.file_name().to_string_lossy(),"kind":if kind.is_dir(){"directory"}else if kind.is_symlink(){"symlink"}else{"file"}}));
                }
                Ok(json!({"entries":entries,"limit":200}))
            }
            "file_read" => {
                let text = read_text(&path)?;
                use sha2::Digest;
                let sha256 = format!("{:x}", sha2::Sha256::digest(text.as_bytes()));
                let start = args["start_line"].as_u64().unwrap_or(1).max(1) as usize;
                let limit = args["line_count"].as_u64().unwrap_or(200).clamp(1, 500) as usize;
                let lines = text
                    .lines()
                    .skip(start - 1)
                    .take(limit)
                    .enumerate()
                    .map(|(i, line)| format!("{}: {line}", start + i))
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(
                    json!({"path":path,"sha256":sha256,"start_line":start,"total_lines":text.lines().count(),"content":self.redactor.text(&truncate(&lines,32_000))}),
                )
            }
            "file_write" => {
                let content = args["content"].as_str().context("Content required")?;
                self.check_instruction_write(&path, content)?;
                if content.len() > MAX_FILE_BYTES as usize {
                    bail!("File exceeds 1 MiB");
                }
                fenix_execution::fs::create_workspace(&path, content.as_bytes(), None).context(
                    "File already exists or cannot be created; use file_edit for an existing file",
                )?;
                Ok(
                    json!({"path":path,"bytes":content.len(),"before_sha256":null,"sha256":fenix_documents::sha256(content.as_bytes()),"verified":read_text(&path)?==content}),
                )
            }
            "file_edit" => {
                let expected = args["expected"]
                    .as_str()
                    .context("Expected text required")?;
                let replacement = args["replacement"]
                    .as_str()
                    .context("Replacement required")?;
                let before = read_text(&path)?;
                if expected.is_empty() || before.matches(expected).count() != 1 {
                    bail!("Expected text must match exactly once; read the file again");
                }
                let after = before.replacen(expected, replacement, 1);
                if after.len() > MAX_FILE_BYTES as usize {
                    bail!("Edited file would exceed 1 MiB");
                }
                // Re-read after approval and match again. Runtime has one tool writer.
                self.check_instruction_write(&path, &after)?;
                if read_text(&path)? != before {
                    bail!("File changed before editing; inspect it again");
                }
                fenix_execution::fs::replace(&path, before.as_bytes(), after.as_bytes())?;
                Ok(
                    json!({"path":path,"bytes":after.len(),"before_sha256":fenix_documents::sha256(before.as_bytes()),"sha256":fenix_documents::sha256(after.as_bytes()),"verified":read_text(&path)?==after}),
                )
            }
            _ => bail!("Unknown file tool"),
        }
    }
}
pub(crate) fn read_text(path: &Path) -> Result<String> {
    let file = fenix_execution::fs::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
        bail!("Only regular UTF-8 files up to 1 MiB are supported");
    }
    let mut text = String::new();
    file.take(MAX_FILE_BYTES + 1).read_to_string(&mut text)?;
    if text.len() > MAX_FILE_BYTES as usize {
        bail!("File grew beyond the size limit");
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn file_tools_reject_escapes_and_verify_expected_content() {
        let dir = tempfile::tempdir().unwrap();
        let workspace = dir.path().join("workspace");
        std::fs::create_dir(&workspace).unwrap();
        let runtime = Runtime::open(
            &dir.path().join("private"),
            Some(&dir.path().join("skills")),
        )
        .await
        .unwrap();
        std::os::unix::fs::symlink(dir.path(), workspace.join("escape")).unwrap();
        for path in [
            "../outside",
            "escape/private/instance.lock",
            ".env",
            ".git/config",
        ] {
            assert!(runtime.file_path(&workspace, path, true).is_err());
        }
        runtime
            .files_execute(
                &workspace,
                "file_write",
                &json!({"path":"note.txt","content":"original"}),
            )
            .unwrap();
        assert!(
            runtime
                .files_execute(
                    &workspace,
                    "file_write",
                    &json!({"path":"note.txt","content":"overwrite"})
                )
                .is_err()
        );
        assert!(
            runtime
                .files_execute(
                    &workspace,
                    "file_edit",
                    &json!({"path":"note.txt","expected":"stale","replacement":"wrong"})
                )
                .is_err()
        );
        let result = runtime
            .files_execute(
                &workspace,
                "file_edit",
                &json!({"path":"note.txt","expected":"original","replacement":"verified"}),
            )
            .unwrap();
        assert_eq!(result["verified"], true);
        assert_eq!(
            std::fs::read_to_string(workspace.join("note.txt")).unwrap(),
            "verified"
        );
    }
}
