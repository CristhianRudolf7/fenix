use crate::redaction::Redactor;
use anyhow::{Context, Result, bail};
use nix::{
    sys::signal::{Signal, killpg},
    unistd::Pid,
};
use serde_json::{Value, json};
use std::{collections::HashMap, path::Path, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::Command,
    sync::{Mutex, Notify},
};
use tokio_util::sync::CancellationToken;

const OUTPUT_LIMIT: usize = 128 * 1024;
struct Process {
    owner: String,
    state: Mutex<ProcessState>,
    done: Notify,
    cancel: CancellationToken,
}
struct ProcessState {
    status: String,
    output: Vec<u8>,
    truncated: bool,
    exit_code: Option<i32>,
}
#[derive(Default)]
pub(crate) struct Processes {
    items: Mutex<HashMap<String, Arc<Process>>>,
}

impl Processes {
    pub async fn run(
        &self,
        task: &str,
        command: &str,
        cwd: &Path,
        timeout: u64,
        task_cancel: &CancellationToken,
        effect_lease: Option<tokio::sync::OwnedMutexGuard<()>>,
    ) -> Result<Value> {
        if command.is_empty() || command.len() > 16_000 {
            bail!("Command must contain 1–16000 bytes");
        }
        let cwd = std::fs::canonicalize(cwd)?;
        if !cwd.is_dir() {
            bail!("Working directory is not a directory");
        }
        let mut items = self.items.lock().await;
        if items.len() >= 500 {
            items.retain(|_, p| p.state.try_lock().is_ok_and(|s| s.status == "running"));
        }
        if items
            .values()
            .filter(|p| p.state.try_lock().map_or(true, |s| s.status == "running"))
            .count()
            >= 8
        {
            bail!("At most eight managed processes may run simultaneously");
        }
        let mut builder = Command::new("/bin/bash");
        builder
            .arg("-c")
            .arg(command)
            .current_dir(cwd)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .process_group(0);
        // Do not hand backend API keys to children through the environment.
        for (key, _) in std::env::vars_os() {
            let name = key.to_string_lossy().to_uppercase();
            if name.contains("API_KEY") || name.contains("TOKEN") || name.contains("SECRET") {
                builder.env_remove(key);
            }
        }
        let mut child = builder.spawn().context("Could not start command")?;
        let pid = child.id().context("Command did not receive a process ID")?;
        let stdout = child.stdout.take().context("Missing stdout")?;
        let stderr = child.stderr.take().context("Missing stderr")?;
        let id = fenix_storage::id();
        let process = Arc::new(Process {
            owner: task.into(),
            state: Mutex::new(ProcessState {
                status: "running".into(),
                output: vec![],
                truncated: false,
                exit_code: None,
            }),
            done: Notify::new(),
            cancel: task_cancel.child_token(),
        });
        items.insert(id.clone(), process.clone());
        drop(items);
        let out = process.clone();
        let mut stdout = tokio::spawn(async move {
            capture(stdout, out).await;
        });
        let out = process.clone();
        let mut stderr = tokio::spawn(async move {
            capture(stderr, out).await;
        });
        tokio::spawn(async move {
            // Shell commands can affect arbitrary paths. Serialize effects until
            // the process group is gone, even if the calling future is dropped.
            let (status, exit) = tokio::select! {
                result=child.wait()=> ("completed",result.ok().and_then(|s|s.code())),
                _=process.cancel.cancelled()=> ("cancelled",None),
                _=tokio::time::sleep(Duration::from_secs(timeout.clamp(1,600)))=> ("timed_out",None),
            };
            // Also close descendant processes that inherited stdout/stderr after shell exit.
            let _ = killpg(Pid::from_raw(pid as i32), Signal::SIGKILL);
            let _ = child.wait().await;
            if tokio::time::timeout(Duration::from_secs(2), &mut stdout)
                .await
                .is_err()
            {
                stdout.abort();
                let _ = stdout.await;
            }
            if tokio::time::timeout(Duration::from_secs(2), &mut stderr)
                .await
                .is_err()
            {
                stderr.abort();
                let _ = stderr.await;
            }
            // Observing a terminal status must also mean the process group has
            // been reaped, output capture has stopped and effects can proceed.
            drop(effect_lease);
            let mut state = process.state.lock().await;
            state.status = status.into();
            state.exit_code = exit;
            drop(state);
            process.done.notify_waiters();
        });
        Ok(json!({"process_id":id,"status":"running"}))
    }

    pub async fn status(
        &self,
        task: &str,
        id: &str,
        wait_ms: u64,
        redactor: &Redactor,
    ) -> Result<Value> {
        let process = self
            .items
            .lock()
            .await
            .get(id)
            .cloned()
            .context("Managed process not found; processes do not resume after restart")?;
        if process.owner != task {
            bail!("This process belongs to another task");
        }
        let notified = process.done.notified();
        if process.state.lock().await.status == "running" && wait_ms > 0 {
            let _ =
                tokio::time::timeout(Duration::from_millis(wait_ms.min(10_000)), notified).await;
        }
        let state = process.state.lock().await;
        Ok(
            json!({"process_id":id,"status":state.status,"exit_code":state.exit_code,"output":redactor.text(&String::from_utf8_lossy(&state.output)),"truncated":state.truncated}),
        )
    }
    pub async fn cancel(&self, task: &str, id: &str) -> Result<Value> {
        let process = self
            .items
            .lock()
            .await
            .get(id)
            .cloned()
            .context("Managed process not found")?;
        if process.owner != task {
            bail!("This process belongs to another task");
        }
        process.cancel.cancel();
        wait_finished(&process).await;
        Ok(json!({"process_id":id,"cancellation_requested":true}))
    }
    pub async fn cancel_task(&self, task: &str) {
        let processes = self
            .items
            .lock()
            .await
            .values()
            .filter(|process| process.owner == task)
            .cloned()
            .collect::<Vec<_>>();
        for process in &processes {
            process.cancel.cancel();
        }
        for process in processes {
            wait_finished(&process).await;
        }
    }
    pub async fn has_running(&self, task: &str) -> bool {
        for p in self.items.lock().await.values().filter(|p| p.owner == task) {
            if p.state.lock().await.status == "running" {
                return true;
            }
        }
        false
    }
    pub async fn cancel_all(&self) {
        let processes = self
            .items
            .lock()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for process in &processes {
            process.cancel.cancel();
        }
        for process in processes {
            wait_finished(&process).await;
        }
    }
}

async fn wait_finished(process: &Process) {
    loop {
        // Register before inspecting status so completion cannot fall between
        // the status check and waiting for its notification.
        let notified = process.done.notified();
        tokio::pin!(notified);
        notified.as_mut().enable();
        if process.state.lock().await.status != "running" {
            return;
        }
        notified.await;
    }
}
async fn capture(mut reader: impl AsyncRead + Unpin, process: Arc<Process>) {
    let mut buffer = [0u8; 4096];
    while let Ok(count) = reader.read(&mut buffer).await {
        if count == 0 {
            break;
        }
        let mut state = process.state.lock().await;
        let remaining = OUTPUT_LIMIT.saturating_sub(state.output.len());
        state
            .output
            .extend_from_slice(&buffer[..count.min(remaining)]);
        state.truncated |= count > remaining;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn commands_accept_arbitrary_cwd_and_redact_output() {
        let dir = tempfile::tempdir().unwrap();
        let processes = Processes::default();
        let cancel = CancellationToken::new();
        let redactor = Redactor::default();
        redactor.add("private-test-value");
        let result = processes
            .run(
                "task",
                "pwd; printf 'private-test-value'",
                dir.path(),
                5,
                &cancel,
                None,
            )
            .await
            .unwrap();
        let id = result["process_id"].as_str().unwrap();
        let status = processes.status("task", id, 5000, &redactor).await.unwrap();
        assert_eq!(status["exit_code"], 0);
        let output = status["output"].as_str().unwrap();
        assert!(output.contains(&dir.path().to_string_lossy().to_string()));
        assert!(output.contains("[REDACTED]"));
        assert!(!output.contains("private-test-value"));
        assert!(processes.cancel("another-task", id).await.is_err());
    }
    #[tokio::test]
    async fn timeouts_terminate_managed_commands_and_bound_output() {
        let dir = tempfile::tempdir().unwrap();
        let processes = Processes::default();
        let cancel = CancellationToken::new();
        let redactor = Redactor::default();
        let result = processes
            .run(
                "task",
                "head -c 200000 /dev/zero; sleep 20",
                dir.path(),
                1,
                &cancel,
                None,
            )
            .await
            .unwrap();
        let status = processes
            .status(
                "task",
                result["process_id"].as_str().unwrap(),
                5000,
                &redactor,
            )
            .await
            .unwrap();
        assert_eq!(status["status"], "timed_out");
        assert_eq!(status["truncated"], true);
        assert!(status["output"].as_str().unwrap().len() <= OUTPUT_LIMIT);
    }

    #[tokio::test]
    async fn cancellation_waits_for_process_cleanup_and_releases_effects() {
        let dir = tempfile::tempdir().unwrap();
        let processes = Processes::default();
        let redactor = Redactor::default();
        let effects = Arc::new(Mutex::new(()));
        for cancel_all in [false, true] {
            let task_cancel = CancellationToken::new();
            let result = processes
                .run(
                    "task",
                    "sleep 60 & wait",
                    dir.path(),
                    60,
                    &task_cancel,
                    Some(effects.clone().lock_owned().await),
                )
                .await
                .unwrap();
            assert!(processes.has_running("task").await);
            assert!(effects.try_lock().is_err());
            tokio::time::timeout(Duration::from_secs(5), async {
                if cancel_all {
                    processes.cancel_all().await;
                } else {
                    processes.cancel_task("task").await;
                }
            })
            .await
            .expect("Cancellation must finish managed process cleanup");
            assert!(!processes.has_running("task").await);
            assert!(effects.try_lock().is_ok());
            let status = processes
                .status("task", result["process_id"].as_str().unwrap(), 0, &redactor)
                .await
                .unwrap();
            assert_eq!(status["status"], "cancelled");
        }
    }
}
