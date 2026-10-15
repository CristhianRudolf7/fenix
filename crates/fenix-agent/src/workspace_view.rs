//! Read-only, account-confined workspace browsing and durable preview requests.
use crate::Runtime;
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::path::{Component, Path, PathBuf};
impl Runtime {
    pub fn view_workspace(&self, project: Option<&str>) -> Result<PathBuf> {
        let settings = self.settings()?.context("Configure the agent first")?;
        let root = if let Some(id) = project {
            self.store
                .project(id)?
                .workspace
                .map(PathBuf::from)
                .context("Project has no workspace")?
        } else {
            settings.workspace
        };
        let canonical = std::fs::canonicalize(&root)?;
        self.validate_user_workspace(&canonical)?;
        ensure!(
            canonical == root,
            "Workspace root must not contain symbolic links"
        );
        Ok(canonical)
    }
    fn view_path(&self, project: Option<&str>, path: &str) -> Result<PathBuf> {
        ensure!(
            path.len() <= 4096 && !Path::new(path).is_absolute(),
            "Use a relative project path"
        );
        ensure!(
            Path::new(path)
                .components()
                .all(|c| matches!(c, Component::Normal(_) | Component::CurDir)),
            "Invalid project path"
        );
        let root = self.view_workspace(project)?;
        let joined = Path::new(path)
            .components()
            .filter_map(|c| {
                if let Component::Normal(v) = c {
                    Some(v)
                } else {
                    None
                }
            })
            .fold(root.clone(), |p, c| p.join(c));
        let checked = self.file_path(&root, joined.to_str().context("Invalid path")?, false)?;
        ensure!(checked == joined, "Symbolic links cannot be browsed");
        Ok(checked)
    }
    pub fn workspace_files(
        &self,
        project: Option<&str>,
        path: &str,
        query: &str,
        offset: usize,
    ) -> Result<Value> {
        let resolved = self.view_path(project, path)?;
        let directory = fenix_execution::fs::directory(&resolved)?;
        let mut entries = vec![];
        for entry in fenix_execution::fs::entries(&directory)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || !name.to_lowercase().contains(&query.to_lowercase()) {
                continue;
            }
            let relative = Path::new(path).join(&name).to_string_lossy().to_string();
            let Ok(checked) = self.view_path(project, &relative) else {
                continue;
            };
            let kind = entry.file_type()?;
            if kind.is_symlink() || (!kind.is_dir() && !kind.is_file()) {
                continue;
            }
            let meta = if kind.is_file() {
                let Ok(file) = fenix_execution::fs::open(&checked) else {
                    continue;
                };
                file.metadata()?
            } else {
                fenix_execution::fs::directory(&checked)?.metadata()?
            };
            entries.push(json!({"name":name,"path":relative,"directory":kind.is_dir(),"bytes":meta.len(),"modified":meta.modified().ok().and_then(|t|t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d|d.as_secs())}));
            ensure!(
                entries.len() <= 20000,
                "Folder is too large; refine the filename search"
            );
        }
        entries.sort_by(|a, b| {
            b["directory"]
                .as_bool()
                .cmp(&a["directory"].as_bool())
                .then_with(|| {
                    a["name"]
                        .as_str()
                        .unwrap()
                        .to_lowercase()
                        .cmp(&b["name"].as_str().unwrap().to_lowercase())
                })
        });
        let more = offset.saturating_add(100) < entries.len();
        Ok(
            json!({"entries":entries.into_iter().skip(offset).take(100).collect::<Vec<_>>(),"path":path,"project_id":project,"next_offset":more.then_some(offset.saturating_add(100))}),
        )
    }
    pub fn workspace_bytes(&self, project: Option<&str>, path: &str) -> Result<Vec<u8>> {
        fenix_execution::fs::read(&self.view_path(project, path)?, 64 * 1024 * 1024)
    }
    pub fn workspace_preview(
        &self,
        project: Option<&str>,
        path: &str,
        offset: usize,
        page: Option<u64>,
    ) -> Result<Value> {
        ensure!(
            page.is_none_or(|p| p > 0 && p <= 2000),
            "Invalid page/slide"
        );
        let bytes = self.workspace_bytes(project, path)?;
        let format = Path::new(path)
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_lowercase();
        let mut out = json!({"path":path,"project_id":project,"title":Path::new(path).file_name().and_then(|n|n.to_str()).unwrap_or(path),"format":format,"sha256":fenix_documents::sha256(&bytes),"bytes":bytes.len()});
        match format.as_str() {
            "docx" | "xlsx" | "pptx" => {
                out["kind"] = json!("office");
                out["extraction"] = if format == "pptx"
                    && let Some(page) = page
                {
                    let mut extracted = fenix_documents::extract(&bytes)?;
                    let part = format!("ppt/slides/slide{page}.xml");
                    let blocks: Vec<_> = extracted["blocks"]
                        .as_array()
                        .context("Missing blocks")?
                        .iter()
                        .filter(|b| b["part"] == part)
                        .cloned()
                        .collect();
                    extracted["total_blocks"] = json!(blocks.len());
                    extracted["blocks"] = json!(
                        blocks
                            .into_iter()
                            .skip(offset)
                            .take(100)
                            .collect::<Vec<_>>()
                    );
                    extracted["next_offset"] = if offset.saturating_add(100)
                        < extracted["total_blocks"].as_u64().unwrap_or(0) as usize
                    {
                        json!(offset.saturating_add(100))
                    } else {
                        Value::Null
                    };
                    extracted
                } else {
                    fenix_documents::inspect(&bytes, offset, 100)?
                };
            }
            "pdf" => out["kind"] = json!("pdf"),
            "png" | "jpg" | "jpeg" | "webp" | "gif" | "svg" => out["kind"] = json!("image"),
            "mp4" | "webm" => out["kind"] = json!("video"),
            _ => {
                if bytes.len() <= 2 * 1024 * 1024
                    && let Ok(text) = std::str::from_utf8(&bytes)
                    && !text.contains('\0')
                {
                    out["kind"] = json!(if format == "md" {
                        "markdown"
                    } else if matches!(format.as_str(), "html" | "htm") {
                        "web"
                    } else {
                        "text"
                    });
                    out["text"] = json!(text);
                } else {
                    out["kind"] = json!("download");
                }
            }
        }
        Ok(out)
    }
    pub(crate) fn notify_workspace_file(
        &self,
        task: &str,
        workspace: &Path,
        path: &str,
        page: Option<u64>,
    ) -> Result<Value> {
        let checked = self.file_path(workspace, path, false)?;
        // Read through the descriptor gate before exposing a target to the UI.
        fenix_execution::fs::open(&checked)?;
        let binding = self.store.task_binding(task)?;
        let root = self.view_workspace(binding.project_id.as_deref())?;
        let relative = checked
            .strip_prefix(&root)
            .context("Preview is outside the project workspace")?
            .to_str()
            .context("Invalid path")?;
        let target = json!({"kind":"file","path":relative,"title":checked.file_name().and_then(|n|n.to_str()),"project_id":binding.project_id,"page":page});
        self.store
            .publish_event(Some(task), "workspace.open", &target)?;
        Ok(json!({"opened":true,"target":target}))
    }
    pub(crate) fn workspace_open(&self, task: &str, args: &Value) -> Result<Value> {
        let project = self.readable_project(task, args)?;
        let page = args["page"].as_u64();
        ensure!(
            page.is_none_or(|p| p > 0 && p <= 2000),
            "Page/slide must be between 1 and 2000"
        );
        let kind = args["kind"].as_str().unwrap_or("file");
        let target = match kind {
            "file" => {
                ensure!(
                    project == self.store.task_binding(task)?.project_id,
                    "File preview must use this task's workspace"
                );
                let root = self.task_workspace(
                    task,
                    &self.settings()?.context("Configure the agent")?.workspace,
                )?;
                return self.notify_workspace_file(
                    task,
                    &root,
                    args["path"].as_str().context("path required")?,
                    page,
                );
            }
            "design" => {
                let id = args["id"].as_str().context("id required")?;
                let d = self.store.design(id, None)?;
                ensure!(d.project_id == project, "Design is outside this task scope");
                json!({"kind":kind,"id":id,"title":d.title,"project_id":project,"page":page})
            }
            "library" => {
                let id = args["id"].as_str().context("id required")?;
                let d = self.store.library_document(id, args["revision"].as_u64())?;
                ensure!(
                    d.project_id == project,
                    "Document is outside this task scope"
                );
                json!({"kind":kind,"id":id,"title":d.title,"project_id":project,"revision":args["revision"],"page":page})
            }
            _ => anyhow::bail!("Expected file, design or library"),
        };
        self.store
            .publish_event(Some(task), "workspace.open", &target)?;
        Ok(json!({"opened":true,"target":target}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;
    use tokio_util::sync::CancellationToken;
    #[tokio::test]
    async fn successful_writes_and_explicit_opens_publish_scoped_preview_events() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
        let workspace = dir.path().join("workspace");
        std::fs::create_dir(&workspace)?;
        let settings = Settings::initial(workspace.clone());
        rt.store.put("settings", "active", &settings)?;
        let task = rt.store.enqueue("Create and show a report", "web", None)?;
        let cancel = CancellationToken::new();
        rt.execute_tool(
            &task.id,
            "web",
            &settings,
            "file_write",
            &json!({"path":"report.md","content":"# Report"}),
            &cancel,
        )
        .await?;
        let opened = rt
            .store
            .recent_bus_events(20)?
            .into_iter()
            .filter(|e| e.topic == "workspace.open")
            .collect::<Vec<_>>();
        assert_eq!(opened.len(), 1);
        assert_eq!(opened[0].data["path"], "report.md");
        assert_eq!(opened[0].task_id.as_deref(), Some(task.id.as_str()));
        assert_eq!(opened[0].data["session_id"], rt.store.session_id());
        assert!(
            rt.execute_tool(
                &task.id,
                "web",
                &settings,
                "file_write",
                &json!({"path":"../outside.md","content":"No"}),
                &cancel
            )
            .await
            .is_ok()
        );
        assert!(
            rt.workspace_open(&task.id, &json!({"kind":"file","path":"missing.md"}))
                .is_err()
        );
        assert_eq!(
            std::fs::read_to_string(dir.path().join("outside.md"))?,
            "No"
        );
        assert_eq!(
            rt.store
                .recent_bus_events(20)?
                .iter()
                .filter(|e| e.topic == "workspace.open")
                .count(),
            1
        );
        assert!(
            rt.workspace_open(&task.id, &json!({"kind":"file","path":"../outside.md"}))
                .is_err()
        );
        let before = rt.store.recent_bus_events(1)?.pop().unwrap().seq;
        rt.workspace_open(
            &task.id,
            &json!({"kind":"file","path":"report.md","page":2}),
        )?;
        let events = rt.store.bus_events(before, 100)?;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data["page"], 2);
        assert!(
            rt.workspace_open(
                &task.id,
                &json!({"kind":"file","path":"report.md","page":0})
            )
            .is_err()
        );
        rt.shutdown().await?;
        Ok(())
    }
}
