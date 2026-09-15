//! Interactive shell sessions and explicitly persistent services. No LLM calls.
mod output;
use crate::redaction::Redactor;
use anyhow::{Context, Result, bail};
use nix::{
    sys::signal::{Signal, killpg},
    unistd::Pid,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    process::Command,
    sync::{Mutex, Notify},
};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct ProcessOptions {
    pub tty: bool,
    pub persistent: bool,
}
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ProcessInfo {
    pub process_id: String,
    pub owner: String,
    pub cwd: PathBuf,
    pub tty: bool,
    pub persistent: bool,
    pub status: String,
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub log_complete: bool,
    #[serde(default)]
    pub output_error: Option<String>,
}
enum Input {
    Pipe(tokio::process::ChildStdin),
    Terminal(pty_process::OwnedWritePty),
}
struct Process {
    info: ProcessInfo,
    state: Mutex<ProcessState>,
    input: Mutex<Option<Input>>,
    done: Notify,
    cancel: CancellationToken,
    log_path: PathBuf,
}
struct ProcessState {
    status: String,
    exit_code: Option<i32>,
    log: tokio::fs::File,
    bytes: u64,
    cursors: HashMap<String, u64>,
    log_complete: bool,
    output_error: Option<String>,
}
pub(crate) struct Processes {
    items: Mutex<HashMap<String, Arc<Process>>>,
    directory: PathBuf,
    redactor: Arc<Redactor>,
    stop: CancellationToken,
}
impl Process {
    async fn output_failure(&self, error: &str) {
        let mut state = self.state.lock().await;
        state.log_complete = false;
        state.output_error = Some(error.to_owned());
        self.cancel.cancel();
    }
}
impl Processes {
    pub fn new(
        directory: PathBuf,
        redactor: Arc<Redactor>,
        stop: CancellationToken,
    ) -> Result<Self> {
        std::fs::create_dir_all(&directory)?;
        Ok(Self {
            items: Mutex::new(HashMap::new()),
            directory,
            redactor,
            stop,
        })
    }
    fn paths(&self, id: &str) -> Result<(PathBuf, PathBuf)> {
        if id.len() != 36 || !id.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-') {
            bail!("Invalid process ID");
        }
        Ok((
            self.directory.join(format!("{id}.json")),
            self.directory.join(format!("{id}.log")),
        ))
    }
    pub async fn info(&self, id: &str) -> Result<ProcessInfo> {
        let items = self.items.lock().await;
        if let Some(process) = items.get(id) {
            let mut info = process.info.clone();
            let state = process.state.lock().await;
            info.status = state.status.clone();
            info.exit_code = state.exit_code;
            info.log_complete = state.log_complete;
            info.output_error = state.output_error.clone();
            return Ok(info);
        }
        drop(items);
        let mut info: ProcessInfo = serde_json::from_slice(
            &tokio::fs::read(self.paths(id)?.0)
                .await
                .context("Process not found")?,
        )?;
        if info.status == "running" {
            info.status = "interrupted".into();
            info.log_complete = false;
            info.output_error = Some("Process was interrupted; final output may be missing".into());
        }
        Ok(info)
    }
    pub async fn list(&self) -> Vec<ProcessInfo> {
        let ids = self.items.lock().await.keys().cloned().collect::<Vec<_>>();
        let mut result = Vec::new();
        for id in ids {
            if let Ok(info) = self.info(&id).await {
                result.push(info);
            }
        }
        result
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn start(
        &self,
        task: &str,
        command: &str,
        cwd: &Path,
        timeout: u64,
        task_cancel: &CancellationToken,
        lease: Option<tokio::sync::OwnedMutexGuard<()>>,
        options: ProcessOptions,
    ) -> Result<Value> {
        if command.is_empty() || command.len() > 16_000 {
            bail!("Command must contain 1–16000 bytes");
        }
        if !(1..=604_800).contains(&timeout) {
            bail!("Process timeout must be between one second and seven days");
        }
        let cwd = std::fs::canonicalize(cwd)?;
        if !cwd.is_dir() {
            bail!("Working directory is not a directory");
        }
        let mut items = self.items.lock().await;
        if items.len() >= 500 {
            items.retain(|_, p| p.state.try_lock().map_or(true, |s| s.status == "running"));
        }
        if items
            .values()
            .filter(|p| p.state.try_lock().map_or(true, |s| s.status == "running"))
            .count()
            >= 16
        {
            bail!("At most sixteen processes may run simultaneously");
        }
        let id = fenix_storage::id();
        let (manifest, log_path) = self.paths(&id)?;
        let log = tokio::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(&log_path)
            .await?;
        let info = ProcessInfo {
            process_id: id.clone(),
            owner: task.into(),
            cwd: cwd.clone(),
            tty: options.tty,
            persistent: options.persistent,
            status: "running".into(),
            exit_code: None,
            log_complete: true,
            output_error: None,
        };
        crate::write_new(&manifest, &serde_json::to_string(&info)?)?;
        let private_env = std::env::vars_os()
            .filter_map(|(key, _)| {
                let name = key.to_string_lossy().to_uppercase();
                (name.contains("API_KEY") || name.contains("TOKEN") || name.contains("SECRET"))
                    .then_some(key)
            })
            .collect::<Vec<_>>();
        if task_cancel.is_cancelled() || self.stop.is_cancelled() {
            bail!("Task cancelled before starting command");
        }
        let (mut child, input, readers): (_, _, Vec<Box<dyn AsyncRead + Send + Unpin>>) =
            if options.tty {
                let (pty, pts) = pty_process::open()?;
                pty.resize(pty_process::Size::new(24, 100))?;
                let mut builder = pty_process::Command::new("/bin/bash")
                    .args(["-c", command])
                    .current_dir(&cwd)
                    .kill_on_drop(true)
                    .env("TERM", "xterm-256color");
                for key in &private_env {
                    builder = builder.env_remove(key);
                }
                let child = builder.spawn(pts)?;
                let (read, write) = pty.into_split();
                (child, Input::Terminal(write), vec![Box::new(read)])
            } else {
                let mut builder = Command::new("/bin/bash");
                builder
                    .args(["-c", command])
                    .current_dir(&cwd)
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .kill_on_drop(true)
                    .process_group(0);
                for key in &private_env {
                    builder.env_remove(key);
                }
                let mut child = builder.spawn().context("Could not start command")?;
                let input = Input::Pipe(child.stdin.take().context("Missing stdin")?);
                let readers: Vec<Box<dyn AsyncRead + Send + Unpin>> = vec![
                    Box::new(child.stdout.take().context("Missing stdout")?),
                    Box::new(child.stderr.take().context("Missing stderr")?),
                ];
                (child, input, readers)
            };
        let pid = child.id().context("Missing process ID")?;
        let process = Arc::new(Process {
            info,
            state: Mutex::new(ProcessState {
                status: "running".into(),
                exit_code: None,
                log,
                bytes: 0,
                cursors: HashMap::new(),
                log_complete: true,
                output_error: None,
            }),
            input: Mutex::new(Some(input)),
            done: Notify::new(),
            cancel: if options.persistent {
                self.stop.child_token()
            } else {
                task_cancel.child_token()
            },
            log_path,
        });
        items.insert(id.clone(), process.clone());
        drop(items);
        let mut captures = Vec::new();
        for reader in readers {
            captures.push(tokio::spawn(output::capture(
                reader,
                process.clone(),
                self.redactor.clone(),
            )));
        }
        // Persistent sessions intentionally overlap later work and release the foreground lease.
        let lease = if options.persistent {
            drop(lease);
            None
        } else {
            lease
        };
        tokio::spawn(async move {
            let (status, exit_code) = tokio::select! {
                result=child.wait() => ("completed", result.ok().and_then(|s|s.code())),
                _=process.cancel.cancelled() => ("cancelled",None),
                _=tokio::time::sleep(Duration::from_secs(timeout)) => ("timed_out",None),
            };
            // Interactive shells put foreground jobs into separate process groups.
            // Stop members of the PTY session as well as the original group.
            if process.info.tty {
                let _ = tokio::task::spawn_blocking(move || kill_terminal_session(pid)).await;
            }
            let _ = killpg(Pid::from_raw(pid as i32), Signal::SIGKILL);
            let _ = child.wait().await;
            process.input.lock().await.take();
            for mut capture in captures {
                if tokio::time::timeout(Duration::from_secs(2), &mut capture)
                    .await
                    .is_err()
                {
                    capture.abort();
                    let _ = capture.await;
                    process
                        .output_failure("Output capture did not close; log may be incomplete")
                        .await;
                }
            }
            drop(lease);
            let mut state = process.state.lock().await;
            if let Err(error) = state.log.flush().await {
                state.output_error = Some(error.to_string());
                state.log_complete = false;
            }
            state.status = if state.output_error.is_some() {
                "failed".into()
            } else {
                status.into()
            };
            state.exit_code = exit_code;
            let mut info = process.info.clone();
            info.status = state.status.clone();
            info.exit_code = exit_code;
            let mut encoded = serde_json::to_value(info).unwrap_or_default();
            encoded["log_complete"] = json!(state.log_complete);
            encoded["output_error"] = json!(state.output_error);
            let _ = tokio::fs::write(manifest, encoded.to_string()).await;
            drop(state);
            process.done.notify_waiters();
        });
        Ok(
            json!({"process_id":id,"status":"running","tty":options.tty,"persistent":options.persistent,"timeout_seconds":timeout,"instruction":"Read incremental output with command_status. Persistent sessions survive successful turns until cancelled, expired, or the service stops."}),
        )
    }
    async fn owned(&self, owner: &str, id: &str) -> Result<Arc<Process>> {
        let process = self
            .items
            .lock()
            .await
            .get(id)
            .cloned()
            .context("Process is no longer live; command_output can recover its saved log")?;
        if process.info.owner != owner {
            bail!("Process belongs to another task");
        }
        Ok(process)
    }
    pub async fn status(&self, owner: &str, reader: &str, id: &str, args: &Value) -> Result<Value> {
        let process = self.owned(owner, id).await?;
        let notified = process.done.notified();
        tokio::pin!(notified);
        notified.as_mut().enable();
        let wait_ms = args["wait_ms"].as_u64().unwrap_or(1000).min(30_000);
        if process.state.lock().await.status == "running" && wait_ms > 0 {
            let _ = tokio::time::timeout(Duration::from_millis(wait_ms), notified).await;
        }
        let mut state = process.state.lock().await;
        state.log.flush().await?;
        let start = args["cursor"]
            .as_u64()
            .unwrap_or_else(|| *state.cursors.get(reader).unwrap_or(&0));
        let limit = args["limit"]
            .as_u64()
            .unwrap_or(output::PAGE_LIMIT as u64)
            .clamp(256, output::PAGE_LIMIT as u64) as usize;
        let (text, next, omitted) =
            output::read_page(&process.log_path, start, state.bytes, limit, true).await?;
        state.cursors.insert(reader.into(), next);
        Ok(
            json!({"process_id":id,"status":state.status,"exit_code":state.exit_code,"output":text,"cursor":start,"next_cursor":next,"omitted_bytes":omitted,"truncated":omitted>0,"log_complete":state.log_complete,"output_error":state.output_error,"persistent":process.info.persistent}),
        )
    }
    pub async fn output(&self, owner: &str, id: &str, args: &Value) -> Result<Value> {
        let info = self.info(id).await?;
        if info.owner != owner {
            bail!("Process belongs to another task");
        }
        let path = self.paths(id)?.1;
        let end = tokio::fs::metadata(&path).await?.len();
        let start = args["offset"].as_u64().unwrap_or(0);
        let limit = args["limit"]
            .as_u64()
            .unwrap_or(output::PAGE_LIMIT as u64)
            .clamp(256, output::PAGE_LIMIT as u64) as usize;
        let (text, next, _) = output::read_page(&path, start, end, limit, false).await?;
        Ok(
            json!({"process_id":id,"output":text,"offset":start,"next_offset":next,"has_more":next<end,"bytes":end,"status":info.status,"log_complete":info.log_complete,"output_error":info.output_error}),
        )
    }
    pub async fn input(
        &self,
        owner: &str,
        id: &str,
        args: &Value,
        cancel: &CancellationToken,
    ) -> Result<Value> {
        let process = self.owned(owner, id).await?;
        let text = args["text"].as_str().unwrap_or("");
        if text.len() > 16_000 {
            bail!("Input is limited to 16000 bytes");
        }
        let mut input = process.input.lock().await;
        let writer = input.as_mut().context("Process stdin is closed")?;
        if let Input::Terminal(pty) = writer
            && (args.get("rows").is_some() || args.get("columns").is_some())
        {
            let rows = args["rows"].as_u64().unwrap_or(24).clamp(1, 500) as u16;
            let columns = args["columns"].as_u64().unwrap_or(100).clamp(1, 500) as u16;
            pty.resize(pty_process::Size::new(rows, columns))?;
        }
        tokio::select! {
            biased;
            _=cancel.cancelled()=>bail!("Task cancelled before sending input"),
            _=process.cancel.cancelled()=>bail!("Process cancelled"),
            result=tokio::time::timeout(Duration::from_secs(5), async {
                match writer {
                    Input::Pipe(pipe)=> { pipe.write_all(text.as_bytes()).await?; pipe.flush().await?; }
                    Input::Terminal(pty)=> { pty.write_all(text.as_bytes()).await?; if args["eof"]==true {pty.write_all(b"\x04").await?;} pty.flush().await?; }
                } Ok::<_,std::io::Error>(())
            })=>result.context("Input write timed out; some bytes may have been sent; inspect before retrying")??,
        }
        if args["eof"] == true && matches!(input.as_ref(), Some(Input::Pipe(_))) {
            input.take();
        }
        Ok(json!({"process_id":id,"bytes_sent":text.len(),"eof":args["eof"]==true}))
    }
    pub async fn cancel(&self, owner: &str, id: &str) -> Result<Value> {
        let process = self.owned(owner, id).await?;
        process.cancel.cancel();
        wait_finished(&process).await;
        Ok(json!({"process_id":id,"cancellation_requested":true}))
    }
    pub async fn cancel_task(&self, owner: &str) {
        self.cancel_matching(Some(owner), false).await;
    }
    pub async fn cancel_foreground(&self, owner: &str) {
        self.cancel_matching(Some(owner), true).await;
    }
    pub async fn cancel_all(&self) {
        self.cancel_matching(None, false).await;
    }
    async fn cancel_matching(&self, owner: Option<&str>, foreground_only: bool) {
        let processes = self
            .items
            .lock()
            .await
            .values()
            .filter(|p| {
                owner.is_none_or(|t| p.info.owner == t) && (!foreground_only || !p.info.persistent)
            })
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
        for p in self
            .items
            .lock()
            .await
            .values()
            .filter(|p| p.info.owner == task && !p.info.persistent)
        {
            if p.state.lock().await.status == "running" {
                return true;
            }
        }
        false
    }
}
async fn wait_finished(process: &Process) {
    loop {
        let notified = process.done.notified();
        tokio::pin!(notified);
        notified.as_mut().enable();
        if process.state.lock().await.status != "running" {
            return;
        }
        notified.await;
    }
}

fn kill_terminal_session(session: u32) {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return;
    };
    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<i32>().ok())
        else {
            continue;
        };
        let Ok(stat) = std::fs::read_to_string(entry.path().join("stat")) else {
            continue;
        };
        let Some((_, fields)) = stat.rsplit_once(") ") else {
            continue;
        };
        // Fields after comm: state, ppid, pgrp, session.
        if fields
            .split_whitespace()
            .nth(3)
            .and_then(|s| s.parse::<u32>().ok())
            == Some(session)
        {
            let _ = nix::sys::signal::kill(Pid::from_raw(pid), Signal::SIGKILL);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (tempfile::TempDir, Processes) {
        let dir = tempfile::tempdir().unwrap();
        let redactor = Arc::new(Redactor::default());
        redactor.add("private-test-value");
        let processes =
            Processes::new(dir.path().join("logs"), redactor, CancellationToken::new()).unwrap();
        (dir, processes)
    }
    async fn complete(processes: &Processes, id: &str) -> Value {
        processes
            .status("task", "task", id, &json!({"wait_ms":5000}))
            .await
            .unwrap()
    }
    #[tokio::test]
    async fn pipe_and_pty_accept_input_and_eof() {
        let (dir, processes) = fixture();
        for tty in [false, true] {
            let command = if tty {
                "test -t 0 || exit 7; stty -echo; printf 'Name?'; read name; printf '\nHello %s\n' \"$name\""
            } else {
                "read name; printf 'Hello %s\n' \"$name\"; cat; printf 'EOF reached\n'"
            };
            let run = processes
                .start(
                    "task",
                    command,
                    dir.path(),
                    10,
                    &CancellationToken::new(),
                    None,
                    ProcessOptions {
                        tty,
                        persistent: false,
                    },
                )
                .await
                .unwrap();
            let id = run["process_id"].as_str().unwrap();
            if tty {
                let preview = processes
                    .status("task", "task", id, &json!({"wait_ms":100}))
                    .await
                    .unwrap();
                assert!(preview["output"].as_str().unwrap().contains("Name?"));
            }
            processes
                .input(
                    "task",
                    id,
                    &json!({"text":"Rudy\n","rows":40,"columns":120}),
                    &CancellationToken::new(),
                )
                .await
                .unwrap();
            if !tty {
                processes
                    .input("task", id, &json!({"eof":true}), &CancellationToken::new())
                    .await
                    .unwrap();
            }
            let result = complete(&processes, id).await;
            assert_eq!(result["exit_code"], 0, "{result}");
            assert!(result["output"].as_str().unwrap().contains("Hello Rudy"));
            assert!(
                processes
                    .input(
                        "other",
                        id,
                        &json!({"text":"wrong"}),
                        &CancellationToken::new()
                    )
                    .await
                    .is_err()
            );
        }
    }
    #[tokio::test]
    async fn incremental_logs_preserve_tail_redact_chunks_and_support_replay() {
        let (dir, processes) = fixture();
        let run=processes.start("task","printf 'private-test-'; sleep 0.1; printf 'value\n'; head -c 200000 /dev/zero; printf '\nFINAL ERROR\n'",dir.path(),10,&CancellationToken::new(),None,ProcessOptions::default()).await.unwrap();
        let id = run["process_id"].as_str().unwrap();
        let result = complete(&processes, id).await;
        assert_eq!(result["exit_code"], 0);
        let output = result["output"].as_str().unwrap();
        assert!(output.contains("FINAL ERROR"));
        assert!(output.contains("[REDACTED]"));
        assert!(!output.contains("private-test"));
        assert_eq!(result["truncated"], true);
        assert_eq!(complete(&processes, id).await["output"], "");
        let log = processes
            .output("task", id, &json!({"offset":0,"limit":256}))
            .await
            .unwrap();
        assert!(log["output"].as_str().unwrap().starts_with("[REDACTED]"));
        assert_eq!(log["has_more"], true);
        let raw = std::fs::read_to_string(processes.paths(id).unwrap().1).unwrap();
        assert!(raw.len() > 200000);
        assert!(!raw.contains("private-test"));
        let reopened = Processes::new(
            processes.directory.clone(),
            Arc::new(Redactor::default()),
            CancellationToken::new(),
        )
        .unwrap();
        let recovered = reopened.output("task", id, &json!({})).await.unwrap();
        assert_eq!(recovered["status"], "completed");
        assert_eq!(recovered["log_complete"], true);
        assert!(recovered["output_error"].is_null());
    }
    #[tokio::test]
    async fn later_master_turns_can_read_foreground_logs_without_controlling_the_process() {
        let dir = tempfile::tempdir().unwrap();
        let runtime =
            crate::Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
                .await
                .unwrap();
        let settings = crate::config::Settings::initial(dir.path().to_owned());
        let binding = fenix_storage::TaskBinding::default();
        let owner = runtime
            .store
            .enqueue_bound("Run a check", "web", None, &binding)
            .unwrap();
        let reader = runtime
            .store
            .enqueue_bound("Read its log", "web", None, &binding)
            .unwrap();
        let cancel = CancellationToken::new();
        let process = runtime
            .processes
            .start(
                &owner.id,
                "printf 'saved evidence'",
                dir.path(),
                10,
                &cancel,
                None,
                ProcessOptions::default(),
            )
            .await
            .unwrap();
        let id = process["process_id"].as_str().unwrap();
        runtime
            .processes
            .status(&owner.id, &owner.id, id, &json!({"wait_ms":5000}))
            .await
            .unwrap();
        let args = json!({"process_id":id});
        let recovered = runtime
            .execute_tool(
                &reader.id,
                &reader.id,
                &settings,
                "command_output",
                &args,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(recovered["output"], "saved evidence");
        assert_eq!(recovered["log_complete"], true);
        assert!(
            runtime
                .execute_tool(
                    &reader.id,
                    &reader.id,
                    &settings,
                    "command_input",
                    &args,
                    &cancel
                )
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn persistent_sessions_survive_turn_cancellation_but_stop_on_shutdown() {
        let (dir, processes) = fixture();
        let task_cancel = CancellationToken::new();
        let effects = Arc::new(Mutex::new(()));
        let run = processes
            .start(
                "task",
                "cat",
                dir.path(),
                60,
                &task_cancel,
                Some(effects.clone().lock_owned().await),
                ProcessOptions {
                    tty: false,
                    persistent: true,
                },
            )
            .await
            .unwrap();
        let id = run["process_id"].as_str().unwrap();
        task_cancel.cancel();
        processes.cancel_foreground("task").await;
        assert_eq!(processes.info(id).await.unwrap().status, "running");
        assert!(effects.try_lock().is_ok());
        processes
            .input(
                "task",
                id,
                &json!({"text":"alive\n"}),
                &CancellationToken::new(),
            )
            .await
            .unwrap();
        let result = processes
            .status("task", "later-turn", id, &json!({"wait_ms":100}))
            .await
            .unwrap();
        assert!(result["output"].as_str().unwrap().contains("alive"));
        processes.cancel_all().await;
        assert_eq!(processes.info(id).await.unwrap().status, "cancelled");
    }
    #[tokio::test]
    async fn timeouts_and_cancellation_release_effects_and_reap_descendants() {
        let (dir, processes) = fixture();
        let effects = Arc::new(Mutex::new(()));
        let run = processes
            .start(
                "task",
                "sleep 60 & wait",
                dir.path(),
                1,
                &CancellationToken::new(),
                Some(effects.clone().lock_owned().await),
                ProcessOptions::default(),
            )
            .await
            .unwrap();
        let id = run["process_id"].as_str().unwrap();
        assert!(effects.try_lock().is_err());
        let status = complete(&processes, id).await;
        assert_eq!(status["status"], "timed_out");
        assert!(effects.try_lock().is_ok());
        assert!(processes.cancel("other", id).await.is_err());
    }
}
