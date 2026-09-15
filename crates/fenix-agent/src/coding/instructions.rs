use crate::{Runtime, files::read_text};
use anyhow::{Result, bail};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;

impl Runtime {
    /// Only walk within the bound workspace: never inherit another project's rules.
    pub(crate) fn repository_instructions(
        &self,
        workspace: &Path,
        path: &Path,
    ) -> Result<Vec<Value>> {
        if !path.starts_with(workspace) {
            bail!("Rules path is outside the project");
        }
        let directory = if path.is_dir() {
            path
        } else {
            path.parent().unwrap_or(workspace)
        };
        let mut directories = directory
            .ancestors()
            .take_while(|p| p.starts_with(workspace))
            .collect::<Vec<_>>();
        directories.reverse();
        let mut rules = Vec::new();
        let mut size = 0;
        for directory in directories {
            let candidate = directory.join("AGENTS.md");
            if !candidate.exists() {
                continue;
            }
            let candidate = self.file_path(workspace, &candidate.to_string_lossy(), false)?;
            let content = read_text(&candidate)?;
            size += content.len();
            if content.len() > 32_000 || size > 64_000 {
                bail!(
                    "Repository instructions exceed the 32 KB file / 64 KB chain limit; shorten AGENTS.md"
                );
            }
            rules.push(json!({"path":candidate,"scope":directory,"sha256":format!("{:x}",Sha256::digest(content.as_bytes())),"content":self.redactor.text(&content)}));
        }
        Ok(rules)
    }
    pub(crate) fn root_repository_instructions(
        &self,
        task: &str,
        workspace: &Path,
    ) -> Result<String> {
        let rules = self.repository_instructions(workspace, workspace)?;
        self.store.put("root_repository_rules", task, &rules)?;
        Ok(format!(
            "\nRepository guidance for this bound workspace (subordinate to the owner's request and runtime policies; deeper AGENTS.md applies only to its subtree):\n{}\nNested rules are returned by file tools before mutations. Read them before retrying.\n",
            serde_json::to_string(&rules)?
        ))
    }
    /// Mark only guidance actually included in the completed model request as seen.
    pub(crate) fn acknowledge_repository_rules(
        &self,
        task: &str,
        messages: &[fenix_core::Message],
    ) -> Result<()> {
        let epoch = self
            .store
            .get::<String>("repository_rules_epoch", task)?
            .unwrap_or_default();
        // Root guidance was already in the system context. Reuse its exact snapshot
        // instead of making the model consume a redundant tool response.
        for rule in self
            .store
            .get::<Vec<Value>>("root_repository_rules", task)?
            .unwrap_or_default()
        {
            if let (Some(path), Some(hash)) = (rule["path"].as_str(), rule["sha256"].as_str()) {
                self.store.put(
                    "repository_rules_seen",
                    &format!("{task}:{path}"),
                    &format!("{epoch}:{hash}"),
                )?;
            }
        }
        for message in messages {
            for block in &message.content {
                if let fenix_core::ContentBlock::ToolResult { content, .. } = block
                    && let Some(rules) = content["repository_instructions"].as_array()
                {
                    for rule in rules {
                        if let (Some(path), Some(hash)) =
                            (rule["path"].as_str(), rule["sha256"].as_str())
                        {
                            self.store.put(
                                "repository_rules_seen",
                                &format!("{task}:{path}"),
                                &format!("{epoch}:{hash}"),
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
    /// New/changed nested guidance is delivered before effects, never after the write.
    pub(crate) fn pending_repository_rules(
        &self,
        task: &str,
        workspace: &Path,
        name: &str,
        args: &Value,
    ) -> Result<Vec<Value>> {
        let paths = if name == "file_patch" {
            super::patch::patch_paths(args)?
        } else {
            vec![args["path"].as_str().unwrap_or(".").to_owned()]
        };
        let mut unseen = Vec::new();
        for path in paths {
            let path = self.file_path(workspace, &path, true)?;
            for rule in self.repository_instructions(workspace, &path)? {
                let key = format!("{}:{}", task, rule["path"].as_str().unwrap());
                let epoch = self
                    .store
                    .get::<String>("repository_rules_epoch", task)?
                    .unwrap_or_default();
                let hash = format!("{epoch}:{}", rule["sha256"].as_str().unwrap());
                if self
                    .store
                    .get::<String>("repository_rules_seen", &key)?
                    .as_deref()
                    != Some(hash.as_str())
                {
                    unseen.push(rule);
                }
            }
        }
        Ok(unseen)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn nested_rules_are_scoped_ordered_and_refreshed() {
        let dir = tempfile::tempdir().unwrap();
        let ws = dir.path().join("ws");
        std::fs::create_dir_all(ws.join("src")).unwrap();
        let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
            .await
            .unwrap();
        std::fs::write(dir.path().join("AGENTS.md"), "outside project").unwrap();
        std::fs::write(ws.join("AGENTS.md"), "root guidance").unwrap();
        std::fs::write(ws.join("src/AGENTS.md"), "nested guidance").unwrap();
        let args = json!({"path":"src/new.rs","content":"fn main() {}"});
        let rules = rt
            .pending_repository_rules("task", &ws, "file_write", &args)
            .unwrap();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0]["content"], "root guidance");
        assert_eq!(rules[1]["content"], "nested guidance");
        rt.acknowledge_repository_rules(
            "task",
            &[fenix_core::Message {
                role: fenix_core::Role::Tool,
                provider_data: None,
                content: vec![fenix_core::ContentBlock::ToolResult {
                    tool_call_id: "rules".into(),
                    name: None,
                    content: json!({"repository_instructions":rules}),
                    is_error: false,
                }],
            }],
        )
        .unwrap();
        assert!(
            rt.pending_repository_rules("task", &ws, "file_write", &args)
                .unwrap()
                .is_empty()
        );
        std::fs::write(ws.join("src/AGENTS.md"), "updated guidance").unwrap();
        assert_eq!(
            rt.pending_repository_rules("task", &ws, "file_write", &args)
                .unwrap()[0]["content"],
            "updated guidance"
        );
        assert_eq!(
            rt.pending_repository_rules("different-task", &ws, "file_write", &args)
                .unwrap()
                .len(),
            2
        );
        assert!(
            !rt.root_repository_instructions("task", &ws)
                .unwrap()
                .contains("outside project")
        );
        std::fs::remove_file(ws.join("src/AGENTS.md")).unwrap();
        std::os::unix::fs::symlink(dir.path().join("AGENTS.md"), ws.join("src/AGENTS.md")).unwrap();
        assert!(
            rt.repository_instructions(&ws, &ws.join("src/new.rs"))
                .is_err()
        );
    }
}
