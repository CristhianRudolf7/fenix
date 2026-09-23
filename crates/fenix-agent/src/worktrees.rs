//! Engine-owned Git worktrees. Workers never merge into the source checkout.
use crate::{Runtime, config::ActionPolicy};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[derive(Default)]
pub(crate) struct Worktrees {
    pub git: Mutex<()>,
    locks: std::sync::Mutex<HashMap<String, Arc<Mutex<()>>>>,
}
impl Worktrees {
    pub fn effect_lock(&self, task: &str) -> Arc<Mutex<()>> {
        self.locks
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .entry(task.into())
            .or_default()
            .clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Worktree {
    pub task_id: String,
    pub source: PathBuf,
    pub path: PathBuf,
    pub branch: String,
    pub base: String,
    pub target_branch: String,
    pub status: String,
    pub head: Option<String>,
    pub integrated_head: Option<String>,
}

// No shell interpolation, credentials, hooks, signing, fsmonitor or prompts.
async fn git_output(path: &Path, args: &[&str]) -> Result<std::process::Output> {
    let mut command = tokio::process::Command::new("git");
    command
        .current_dir(path)
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("HOME", "/nonexistent")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_AUTHOR_NAME", "Fenix")
        .env("GIT_AUTHOR_EMAIL", "agent@fenix.local")
        .env("GIT_COMMITTER_NAME", "Fenix")
        .env("GIT_COMMITTER_EMAIL", "agent@fenix.local")
        .args([
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "commit.gpgSign=false",
            "-c",
            "merge.gpgSign=false",
            "-c",
            "gc.auto=0",
        ])
        .args(args)
        .kill_on_drop(true);
    tokio::time::timeout(std::time::Duration::from_secs(90), command.output())
        .await
        .context("Git operation timed out; inspect the preserved workspace before retrying")?
        .context("Git is required for isolated coding workers")
}
async fn git(path: &Path, args: &[&str]) -> Result<String> {
    let output = git_output(path, args).await?;
    ensure!(
        output.status.success(),
        "Git {} failed: {}",
        args.first().unwrap_or(&""),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}
async fn no_custom_git_drivers(path: &Path) -> Result<()> {
    let config = git_output(
        path,
        &[
            "config",
            "--get-regexp",
            r"^(filter\..*\.(clean|smudge|process)|merge\..*\.driver)$",
        ],
    )
    .await?;
    ensure!(
        config.status.code() == Some(1),
        "Repositories with custom Git filter or merge commands require manual integration; engine-managed Git does not execute repository-defined programs"
    );
    Ok(())
}

async fn clean(path: &Path) -> Result<()> {
    ensure!(
        git(
            path,
            &["status", "--porcelain=v1", "--untracked-files=normal"]
        )
        .await?
        .is_empty(),
        "Checkout has uncommitted files. Preserve/commit them before starting or integrating isolated work; no files were overwritten"
    );
    for state in [
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "REVERT_HEAD",
        "rebase-merge",
        "rebase-apply",
    ] {
        let resolved = git(path, &["rev-parse", "--git-path", state]).await?;
        ensure!(
            !path.join(resolved).exists(),
            "Checkout has an unfinished Git operation"
        );
    }
    Ok(())
}

impl Runtime {
    pub(crate) fn worktree(&self, task: &str) -> Result<Option<Worktree>> {
        self.store.get("task_worktrees", task)
    }

    fn validate_worktree_path(&self, path: &Path) -> Result<PathBuf> {
        let path = std::fs::canonicalize(path)?;
        ensure!(
            !path.starts_with(&self.data_dir),
            "Private runtime state cannot be a worktree"
        );
        if let Some(root) = &self.tenant_root {
            ensure!(
                path.starts_with(root.join("projects")) || path.starts_with(root.join("workspace")),
                "Worktree must remain inside this user's projects or workspace"
            );
        }
        Ok(path)
    }

    async fn check_worktree_identity(&self, record: &Worktree) -> Result<()> {
        ensure!(
            self.validate_worktree_path(&record.path)? == record.path,
            "Worker path was redirected"
        );
        ensure!(
            self.validate_worktree_path(&record.source)? == record.source,
            "Source path was redirected"
        );
        let source_common = git(
            &record.source,
            &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        )
        .await?;
        let worker_common = git(
            &record.path,
            &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        )
        .await?;
        ensure!(
            self.validate_worktree_path(Path::new(&source_common))?
                == self.validate_worktree_path(Path::new(&worker_common))?,
            "Worker is no longer attached to its assigned repository"
        );
        ensure!(
            git(&record.path, &["rev-parse", "--show-toplevel"]).await?
                == record.path.to_string_lossy(),
            "Worker root was replaced"
        );
        Ok(())
    }

    async fn require_source_idle(&self, source: &Path) -> Result<()> {
        ensure!(
            !self
                .processes
                .list()
                .await
                .iter()
                .any(|p| p.status == "running" && p.cwd.starts_with(source)),
            "A command is still using the source checkout; finish it before provisioning or integrating worktrees"
        );
        Ok(())
    }

    /// Called before building context, including workers created by the planner.
    pub(crate) async fn prepare_worktree(&self, task: &str, fallback: &Path) -> Result<()> {
        if self.store.review_for_worker(task)?.is_some()
            || self.store.completion_audit_for_worker(task)?.is_some()
        {
            return Ok(());
        }
        let _git = self.worktrees.git.lock().await;
        if let Some(record) = self.worktree(task)? {
            self.check_worktree_identity(&record).await?;
            ensure!(
                record.status != "provisioning",
                "Worktree provisioning was interrupted; inspect the retained workspace"
            );
            ensure!(
                git(&record.path, &["symbolic-ref", "--short", "HEAD"]).await? == record.branch,
                "Worker branch changed outside its assignment"
            );
            return Ok(());
        }
        let binding = self.store.task_binding(task)?;
        let source = self.validate_worktree_path(
            binding
                .workspace
                .as_deref()
                .map(Path::new)
                .unwrap_or(fallback),
        )?;
        // Plain directories keep the existing execution behavior. A broken Git
        // checkout is an error, never a silent fallback to shared writes.
        if !source.ancestors().any(|p| p.join(".git").exists()) {
            return Ok(());
        }
        let probe = git_output(&source, &["rev-parse", "--show-toplevel"]).await?;
        ensure!(
            probe.status.success(),
            "Cannot inspect this Git checkout; shared execution fallback is disabled"
        );
        let top =
            self.validate_worktree_path(Path::new(String::from_utf8(probe.stdout)?.trim()))?;
        ensure!(
            top == source,
            "Set the project workspace to its repository root before delegating Git modifications"
        );
        let common = git(
            &source,
            &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        )
        .await?;
        self.validate_worktree_path(Path::new(&common))?;
        let _effects = self.effects.clone().lock_owned().await;
        self.require_source_idle(&source).await?;
        no_custom_git_drivers(&source).await?;
        clean(&source).await?;
        let base = git(&source, &["rev-parse", "HEAD"])
            .await
            .context("Repository needs an initial commit")?;
        let target_branch = git(&source, &["symbolic-ref", "--short", "HEAD"])
            .await
            .context("Repository must be on a named branch")?;
        ensure!(
            task.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'),
            "Invalid task identifier"
        );
        use sha2::{Digest, Sha256};
        let key = format!(
            "{:x}",
            Sha256::digest(source.as_os_str().as_encoded_bytes())
        );
        let root = self
            .tenant_root
            .as_ref()
            .map(|p| p.join("projects"))
            .unwrap_or_else(|| source.parent().unwrap_or(&source).to_owned());
        let directory = root.join(".fenix-worktrees").join(&key[..16]);
        self.validate_worktree_path(&root)?;
        let marker = directory.join(".fenix-owner");
        if !marker.exists() {
            fenix_execution::fs::create(&marker, self.data_dir.as_os_str().as_encoded_bytes())?;
        }
        ensure!(
            fenix_execution::fs::read(&marker, 4096)?
                == self.data_dir.as_os_str().as_encoded_bytes(),
            "Worktree directory belongs to another runtime"
        );
        let directory = self.validate_worktree_path(&directory)?;
        let path = directory.join(task);
        ensure!(
            !path.exists(),
            "Reserved worker workspace already exists; refusing to overwrite it"
        );
        let mut record = Worktree {
            task_id: task.into(),
            source,
            path,
            branch: format!("fenix/worker-{task}"),
            base,
            target_branch,
            status: "provisioning".into(),
            head: None,
            integrated_head: None,
        };
        self.store.put("task_worktrees", task, &record)?;
        git(
            &record.source,
            &[
                "worktree",
                "add",
                "-b",
                &record.branch,
                record.path.to_str().context("UTF-8 workspace required")?,
                &record.base,
            ],
        )
        .await?;
        record.status = "active".into();
        self.store.put("task_worktrees", task, &record)?;
        self.store.event(task, "worktree_created", &json!(record))?;
        Ok(())
    }

    pub(crate) async fn worktree_status(&self, task: &str) -> Result<Value> {
        let Some(record) = self.worktree(task)? else {
            return Ok(json!({"isolated":false}));
        };
        self.check_worktree_identity(&record).await?;
        let files = git(&record.path, &["status", "--short"]).await?;
        Ok(
            json!({"isolated":true,"worktree":record,"changes":crate::redaction::truncate(&files,8000)}),
        )
    }

    /// Merge only after tool effects have settled. This is local Git, never push.
    pub(crate) async fn integrate_worktree(
        &self,
        task: &str,
        cancel: &CancellationToken,
    ) -> Result<Value> {
        let Some(mut record) = self.worktree(task)? else {
            return Ok(json!({"isolated":false}));
        };
        // Wait for filesystem resources without holding the planner hostage.
        // Replanning can cancel this wait; the assignment is checked again once
        // all resources are owned, immediately before applying Git changes.
        let _git = tokio::select! {
            _ = cancel.cancelled() => bail!("Integration cancelled while waiting for Git"),
            _ = self.stop.cancelled() => bail!("Runtime stopped before integration"),
            lock = self.worktrees.git.lock() => lock,
        };
        let _global = tokio::select! {
            _ = cancel.cancelled() => bail!("Integration cancelled while waiting for source files"),
            _ = self.stop.cancelled() => bail!("Runtime stopped before integration"),
            lock = self.effects.clone().lock_owned() => lock,
        };
        let _worker = tokio::select! {
            _ = cancel.cancelled() => bail!("Integration cancelled while waiting for worker files"),
            _ = self.stop.cancelled() => bail!("Runtime stopped before integration"),
            lock = self.worktrees.effect_lock(task).lock_owned() => lock,
        };
        let _plan = tokio::select! {
            _ = cancel.cancelled() => bail!("Integration cancelled before checking plan"),
            _ = self.stop.cancelled() => bail!("Runtime stopped before integration"),
            lock = self.plan_lock.lock() => lock,
        };
        ensure!(
            !cancel.is_cancelled() && !self.stop.is_cancelled(),
            "Task cancelled before integration"
        );
        self.store.check_plan_worker(task)?;
        self.store.check_task_spec_changes(task)?;
        self.check_worktree_identity(&record).await?;
        self.require_source_idle(&record.source).await?;
        ensure!(
            record.status != "provisioning",
            "Incomplete worktree provisioning"
        );
        ensure!(
            git(&record.path, &["symbolic-ref", "--short", "HEAD"]).await? == record.branch,
            "Worker must retain its assigned branch"
        );
        ensure!(
            git(&record.source, &["symbolic-ref", "--short", "HEAD"]).await?
                == record.target_branch,
            "Source branch changed; integration requires explicit reconciliation"
        );
        no_custom_git_drivers(&record.source).await?;
        no_custom_git_drivers(&record.path).await?;
        clean(&record.source).await?;
        ensure!(
            git_output(
                &record.path,
                &["merge-base", "--is-ancestor", &record.base, "HEAD"]
            )
            .await?
            .status
            .success(),
            "Worker history no longer descends from its assigned base"
        );
        for state in [
            "MERGE_HEAD",
            "CHERRY_PICK_HEAD",
            "REVERT_HEAD",
            "rebase-merge",
            "rebase-apply",
        ] {
            let path = git(&record.path, &["rev-parse", "--git-path", state]).await?;
            ensure!(
                !record.path.join(path).exists(),
                "Worker has an unfinished Git operation"
            );
        }
        ensure!(
            git_output(
                &record.source,
                &["merge-base", "--is-ancestor", &record.base, "HEAD"]
            )
            .await?
            .status
            .success(),
            "Source history was rewritten after assignment; reconcile before integrating old work"
        );
        // Refuse unfinished Git operations; unstaged worker edits are expected.
        let conflicts = git(&record.path, &["diff", "--name-only", "--diff-filter=U"]).await?;
        ensure!(
            conflicts.is_empty(),
            "Resolve worker conflicts before integration"
        );
        git(&record.path, &["add", "--all", "--", "."]).await?;
        let changed = git_output(&record.path, &["diff", "--cached", "--quiet"]).await?;
        ensure!(
            matches!(changed.status.code(), Some(0 | 1)),
            "Could not inspect staged worker changes"
        );
        if changed.status.code() == Some(1) {
            git(
                &record.path,
                &["commit", "-m", &format!("Complete delegated task {task}")],
            )
            .await?;
        }
        let head = git(&record.path, &["rev-parse", "HEAD"]).await?;
        record.head = Some(head.clone());
        record.status = "ready".into();
        self.store.put("task_worktrees", task, &record)?;
        let before = git(&record.source, &["rev-parse", "HEAD"]).await?;
        if !git_output(
            &record.source,
            &["merge-base", "--is-ancestor", &head, &before],
        )
        .await?
        .status
        .success()
        {
            // Check conflicts without touching the source index or files.
            let merge = git_output(
                &record.source,
                &["merge-tree", "--write-tree", &before, &head],
            )
            .await?;
            if !merge.status.success() {
                record.status = "conflict".into();
                self.store.put("task_worktrees", task, &record)?;
                bail!(
                    "Worker changes conflict with the source checkout. Branch {} and workspace {} were preserved; source files are unchanged. Reconcile in an isolated workspace before retrying. Conflict details: {}",
                    record.branch,
                    record.path.display(),
                    crate::redaction::truncate(&String::from_utf8_lossy(&merge.stdout), 8000)
                );
            }
            ensure!(!cancel.is_cancelled(), "Task cancelled before merge");
            self.store.check_plan_worker(task)?;
            self.store.event(
                task,
                "worktree_integration_started",
                &json!({"source_head":before,"worker_head":head,"source":record.source}),
            )?;
            git(
                &record.source,
                &[
                    "merge",
                    "--no-ff",
                    "--no-edit",
                    "-m",
                    &format!("Integrate delegated task {task}"),
                    &head,
                ],
            )
            .await?;
        }
        record.status = "integrated".into();
        record.integrated_head = Some(git(&record.source, &["rev-parse", "HEAD"]).await?);
        self.store.put("task_worktrees", task, &record)?;
        self.store
            .event(task, "worktree_integrated", &json!(record))?;
        Ok(json!(record))
    }

    pub(crate) async fn finish_worktree(
        &self,
        task: &str,
        mut result: Value,
        cancel: &CancellationToken,
    ) -> Value {
        match self.worktree(task) {
            Ok(None) => return result,
            Err(error) => {
                result["status"] = json!("blocked");
                result["blockers"] = json!(
                    self.redactor
                        .text(&format!("Cannot verify worktree assignment: {error}"))
                );
                return result;
            }
            Ok(Some(_)) => {}
        }
        if result["status"] == "completed" {
            result["execution_completed"] = json!(true);
            let integration = async {
                if self.settings()?.is_some_and(|s| s.file_write_policy == ActionPolicy::Ask) {
                    let action = json!({"tool":"worktree_manage","arguments":{"action":"integrate","subagent_id":task}});
                    ensure!(self.approve(task, action, cancel).await?, "Worktree integration was not approved; changes were preserved");
                }
                self.integrate_worktree(task, cancel).await
            }.await;
            match integration {
                Ok(info) => result["worktree"] = info,
                Err(error) => {
                    result["status"] = json!("blocked");
                    result["blockers"] = json!(self.redactor.text(&error.to_string()));
                    result["worktree"] = self
                        .worktree(task)
                        .ok()
                        .flatten()
                        .map_or(Value::Null, |r| json!(r));
                    let _ = self.store.event(
                        task,
                        "worktree_integration_blocked",
                        &json!({"reason":result["blockers"]}),
                    );
                }
            }
        } else {
            result["worktree"] = self
                .worktree(task)
                .ok()
                .flatten()
                .map_or(Value::Null, |r| json!(r));
        }
        result
    }
}

#[cfg(test)]
mod tests;
