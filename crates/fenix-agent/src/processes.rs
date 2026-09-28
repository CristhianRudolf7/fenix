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

#[derive(Debug, Serialize)]
pub(crate) struct BusyProcess {
    pub code: &'static str,
    pub retryable: bool,
    pub executed: bool,
    pub processes: Value,
    pub instruction: &'static str,
}
impl std::fmt::Display for BusyProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.instruction)
    }
}
impl std::error::Error for BusyProcess {}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct ProcessOptions {
    pub tty: bool,
    pub persistent: bool,
    #[serde(default)]
    pub read_paths: Vec<PathBuf>,
    #[serde(default)]
    pub workspace: Option<PathBuf>,
    #[serde(default)]
    pub resource_paths: Vec<PathBuf>,
    /// Fixed image-owned PDF/OCR worker; never accepted from command_run arguments.
    #[serde(skip)]
    pub document_request: Option<Value>,
}
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ProcessInfo {
    pub process_id: String,
    pub owner: String,
    pub cwd: PathBuf,
    pub tty: bool,
    pub persistent: bool,
    #[serde(default)]
    pub read_paths: Vec<PathBuf>,
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
    execution: Option<Arc<fenix_execution::Lease>>,
    started: tokio::time::Instant,
}
struct ProcessState {
    status: String,
    exit_code: Option<i32>,
    log: tokio::fs::File,
    bytes: u64,
    cursors: HashMap<String, u64>,
    log_complete: bool,
    output_error: Option<String>,
    elapsed_ms: Option<u64>,
}
pub(crate) struct Processes {
    items: Mutex<HashMap<String, Arc<Process>>>,
    directory: PathBuf,
    /// Standalone account home, captured once rather than inferred from command cwd.
    home: PathBuf,
    redactor: Arc<Redactor>,
    stop: CancellationToken,
    executor: Option<fenix_execution::Executor>,
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
    pub async fn progress_signature(&self, owner: &str) -> String {
        let items = self.items.lock().await;
        let mut result = Vec::new();
        for p in items.values().filter(|p| p.info.owner == owner) {
            let state = p.state.lock().await;
            result.push(format!(
                "{}:{}:{}:{:?}",
                p.info.process_id, state.status, state.bytes, state.exit_code
            ));
        }
        result.sort();
        result.join(";")
    }
    pub async fn wait_foreground(&self, owner: &str, cancel: &CancellationToken) -> Result<()> {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
        loop {
            if !self.has_running(owner).await {
                return Ok(());
            }
            if tokio::time::Instant::now() >= deadline {
                let processes = self
                    .list()
                    .await
                    .into_iter()
                    .filter(|p| p.owner == owner && !p.persistent && p.status == "running")
                    .collect::<Vec<_>>();
                return Err(BusyProcess {
                    code: "process_busy", retryable: true, executed: false,
                    processes: json!(processes),
                    instruction: "An earlier command is still running. Use command_status with its process_id and wait_ms. Do not cancel a progressing installation merely to inspect it. This action has NOT executed.",
                }.into());
            }
            tokio::select! { _ = cancel.cancelled() => bail!("Task cancelled while waiting for command"), _ = tokio::time::sleep(Duration::from_millis(100)) => {} }
        }
    }
    pub async fn wait_effect(
        &self,
        owner: &str,
        writes: Option<&[PathBuf]>,
        cancel: &CancellationToken,
    ) -> Result<()> {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
        loop {
            let conflicts: Vec<_> = self
                .list()
                .await
                .into_iter()
                .filter(|p| {
                    if p.status != "running" || p.persistent {
                        return false;
                    }
                    if p.read_paths.is_empty() {
                        return p.owner == owner;
                    }
                    writes.is_none_or(|writes| {
                        use std::os::unix::fs::MetadataExt;
                        writes
                            .iter()
                            .any(|write| std::fs::metadata(write).is_ok_and(|m| m.nlink() > 1))
                            || p.read_paths.iter().any(|read| {
                                writes
                                    .iter()
                                    .any(|write| read.starts_with(write) || write.starts_with(read))
                            })
                    })
                })
                .collect();
            if conflicts.is_empty() {
                return Ok(());
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(BusyProcess { code:"process_busy",retryable:true,executed:false,processes:json!(conflicts),instruction:"A process conflicts with these resources. The requested action did NOT execute. Inspect its output/status or use an independent output path. Read-only inputs stay locked until their process exits." }.into());
            }
            tokio::select! { _=cancel.cancelled()=>bail!("Task cancelled while waiting for resources"), _=tokio::time::sleep(Duration::from_millis(100))=>{} }
        }
    }
    pub fn new(
        directory: PathBuf,
        redactor: Arc<Redactor>,
        stop: CancellationToken,
    ) -> Result<Self> {
        std::fs::create_dir_all(&directory)?;
        let fallback_home = directory.canonicalize()?.join("home");
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute() && path.is_dir())
            .unwrap_or(fallback_home);
        std::fs::create_dir_all(&home)?;
        Ok(Self {
            items: Mutex::new(HashMap::new()),
            directory,
            home,
            redactor,
            stop,
            executor: fenix_execution::Executor::from_environment()?,
        })
    }
    fn paths(&self, id: &str) -> Result<(PathBuf, PathBuf)> {
        let legacy = id.len() == 36
            && id.bytes().enumerate().all(|(i, b)| {
                if matches!(i, 8 | 13 | 18 | 23) {
                    b == b'-'
                } else {
                    b.is_ascii_hexdigit()
                }
            });
        let compact = id
            .strip_prefix("p_")
            .is_some_and(|v| v.len() == 16 && v.bytes().all(|b| b.is_ascii_hexdigit()));
        if !legacy && !compact {
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
        if command.is_empty() || command.chars().count() > 16_000 {
            bail!("Command must contain 1–16000 characters");
        }
        if !(1..=604_800).contains(&timeout) {
            bail!("Process timeout must be between one second and seven days");
        }
        if options.document_request.is_some()
            && (self.executor.is_none() || options.tty || options.persistent)
        {
            bail!("Document extraction requires the isolated execution service; no host fallback");
        }
        // Keep the supervising shell alive for ordinary background jobs as well
        // as foreground services. Group cleanup still runs on timeout/cancel.
        let supervised = if options.persistent {
            format!("{command}\n_fenix_command_status=$?\nwait\nexit \"$_fenix_command_status\"")
        } else {
            command.to_owned()
        };
        let command = supervised.as_str();
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
        let id = format!("p_{}", &fenix_storage::id().replace('-', "")[..16]);
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
            read_paths: options.read_paths.clone(),
            status: "running".into(),
            exit_code: None,
            log_complete: true,
            output_error: None,
        };
        crate::write_new(&manifest, &serde_json::to_string(&info)?)?;
        let remote = if let Some(executor) = &self.executor {
            let argv = if let Some(request) = &options.document_request {
                vec![
                    "/opt/fenix-ocr/bin/python".into(),
                    "-I".into(),
                    "-u".into(),
                    "/usr/local/lib/fenix-ocr/worker.py".into(),
                    serde_json::to_string(request)?,
                ]
            } else if options.read_paths.is_empty() {
                vec![
                    "/bin/bash".into(),
                    "--noprofile".into(),
                    "--norc".into(),
                    "-c".into(),
                    command.into(),
                ]
            } else {
                vec![
                    "/usr/local/bin/fenix".into(),
                    "internal-read-only".into(),
                    command.into(),
                    cwd.to_string_lossy().into_owned(),
                    serde_json::to_string(&options.read_paths)?,
                ]
            };
            let mut spec = fenix_execution::Specification::new(
                task,
                if options.document_request.is_some() {
                    "document"
                } else {
                    "shell"
                },
                argv,
                &cwd,
                options.workspace.as_deref().unwrap_or(&cwd),
                timeout,
            );
            spec.tty = options.tty;
            spec.read_paths = options.read_paths.clone();
            spec.resource_paths = options.resource_paths.clone();
            Some(Arc::new(executor.create(spec).await?))
        } else {
            None
        };
        let timeout = remote.as_ref().map_or(timeout, |lease| lease.timeout);
        // The remote executor supplies its own tenant-scoped home. These
        // allowlisted defaults are used only by standalone shell processes.
        let environment = fenix_execution::local_environment(&self.home);
        if task_cancel.is_cancelled() || self.stop.is_cancelled() {
            bail!("Task cancelled before starting command");
        }
        let (mut child, input, readers): (_, _, Vec<Box<dyn AsyncRead + Send + Unpin>>) =
            if options.tty {
                let (pty, pts) = pty_process::open()?;
                pty.resize(pty_process::Size::new(24, 100))?;
                let builder = if let Some(remote) = &remote {
                    pty_process::Command::new(std::env::current_exe()?)
                        .arg("internal-runner")
                        .env_clear()
                        .envs(remote.relay_environment())
                } else {
                    pty_process::Command::new("/bin/bash")
                        .args(["--noprofile", "--norc", "-c", command])
                        .env_clear()
                        .envs(&environment)
                }
                .current_dir(&cwd)
                .kill_on_drop(true);
                let child = builder.spawn(pts)?;
                let (read, write) = pty.into_split();
                (child, Input::Terminal(write), vec![Box::new(read)])
            } else {
                let mut builder = if let Some(remote) = &remote {
                    remote.command()?
                } else {
                    let mut builder = Command::new("/bin/bash");
                    builder
                        .args(["--noprofile", "--norc", "-c", command])
                        .env_clear()
                        .envs(&environment);
                    builder
                };
                builder
                    .current_dir(&cwd)
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .kill_on_drop(true)
                    .process_group(0);
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
            started: tokio::time::Instant::now(),
            info,
            state: Mutex::new(ProcessState {
                status: "running".into(),
                exit_code: None,
                log,
                bytes: 0,
                cursors: HashMap::new(),
                log_complete: true,
                output_error: None,
                elapsed_ms: None,
            }),
            input: Mutex::new(Some(input)),
            done: Notify::new(),
            cancel: if options.persistent {
                self.stop.child_token()
            } else {
                task_cancel.child_token()
            },
            log_path,
            execution: remote,
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
            if let Some(execution) = &process.execution
                && execution.cancel().await.is_err()
            {
                process
                    .output_failure("Remote cleanup is pending; execution service will retry")
                    .await;
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
            state.elapsed_ms =
                Some(process.started.elapsed().as_millis().min(u64::MAX as u128) as u64);
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
    pub async fn initial_wait(&self, owner: &str, cwd: &Path, base: u64, interactive: bool) -> u64 {
        if interactive {
            return base.min(100);
        }
        if base == 0 {
            return 0;
        }
        let items = self.items.lock().await;
        let mut estimate = base;
        for process in items
            .values()
            .filter(|p| p.info.owner == owner && p.info.cwd == cwd && !p.info.persistent)
        {
            let state = process.state.lock().await;
            if state.exit_code == Some(0)
                && let Some(ms) = state.elapsed_ms
                && ms <= 10_000
            {
                estimate = estimate.max(ms.saturating_add(200).min(10_000));
            }
        }
        estimate.min(30_000)
    }
    /// Wait on process completion without model polling. Silence is advisory, not failure.
    pub async fn wait_event(
        &self,
        owner: &str,
        id: &str,
        cancel: &CancellationToken,
        total: Duration,
        idle: Duration,
    ) -> Result<Value> {
        let process = self.owned(owner, id).await?;
        if process.info.tty || process.info.persistent {
            return Ok(json!({"reason":"interactive_or_service"}));
        }
        let started = tokio::time::Instant::now();
        let mut last_activity = started;
        let mut bytes = process.state.lock().await.bytes;
        let notified = process.done.notified();
        tokio::pin!(notified);
        notified.as_mut().enable();
        let reason = loop {
            let state = process.state.lock().await;
            if state.status != "running" {
                break "process_ended";
            }
            if state.bytes != bytes {
                bytes = state.bytes;
                last_activity = tokio::time::Instant::now();
            }
            drop(state);
            if started.elapsed() >= total {
                break "review_due";
            }
            if last_activity.elapsed() >= idle {
                break "quiet_process";
            }
            tokio::select! {
                _ = cancel.cancelled() => bail!("Command wait cancelled"),
                _ = self.stop.cancelled() => bail!("Runtime stopped during command wait"),
                _ = &mut notified => break "process_ended",
                _ = tokio::time::sleep(Duration::from_secs(2).min(total.saturating_sub(started.elapsed())).min(idle.saturating_sub(last_activity.elapsed()))) => {}
            }
        };
        let mut event = json!({"reason":reason,"waited_ms":started.elapsed().as_millis(),
            "process_elapsed_ms":process.started.elapsed().as_millis(),
            "instruction":"Silence alone does not establish a hang. Inspect recent output for an explicit input prompt before waiting again. Respond only with task-authorized settings; do not guess credentials, consent or owner intent. Without such evidence, continue waiting as appropriate. This advisory never sends input or kills the process; timeout and cancellation remain enforced."});
        if reason == "quiet_process" {
            // Incremental status may now be empty although the last output was a
            // prompt. Reuse the redacted log without advancing any reader cursor.
            let mut state = process.state.lock().await;
            state.log.flush().await?;
            let end = state.bytes;
            drop(state);
            let (text, _, _) = output::read_page(
                &process.log_path,
                end.saturating_sub(1200),
                end,
                1200,
                false,
            )
            .await?;
            if !text.is_empty() {
                event["recent_output"] = json!(text);
                event["recent_output_is_replayed"] = json!(true);
            }
        }
        Ok(event)
    }
    pub async fn status(&self, owner: &str, reader: &str, id: &str, args: &Value) -> Result<Value> {
        if let Some(port) = args["ready_port"].as_u64()
            && (port == 0 || port > 65535)
        {
            bail!("ready_port must be between 1 and 65535");
        }
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
        let mut result = json!({"process_id":id,"status":state.status,"exit_code":state.exit_code,"output":text,"cursor":start,"next_cursor":next,"omitted_bytes":omitted,"truncated":omitted>0,"log_complete":state.log_complete,"output_error":state.output_error,"persistent":process.info.persistent,"read_paths":process.info.read_paths,"descendant_policy":if process.info.persistent {"supervised_until_cancel_or_timeout"} else {"reaped_when_shell_exits"}});
        result["execution_mode"] = json!(if process.info.read_paths.is_empty() {
            "workspace"
        } else {
            "read_only"
        });
        if state.exit_code.is_some_and(|code| code != 0) {
            result["failure"] = json!({"code":"command_exit_nonzero","category":"program_exit","permissions_relaxed":false,"instruction":"Inspect the output; nonzero exit alone does not prove a permission denial or a program bug."});
            if crate::execution_state::missing_dependency_diagnostic(
                result["output"].as_str().unwrap_or(""),
            ) {
                result["failure"]["category"] = json!("missing_dependency");
                result["failure"]["instruction"] = json!(
                    "A required executable/module is missing. Inspect the build metadata and install the necessary dependency within existing authorization, then resume the failed step. An APT _apt download warning is not a denial of the requested build. This diagnostic does not grant permissions."
                );
            }
        }
        drop(state);
        if let Some(port) = args["ready_port"].as_u64() {
            let connected = if let Some(execution) = &process.execution {
                tokio::time::timeout(Duration::from_secs(5), execution.connect(port as u16))
                    .await
                    .is_ok_and(|v| v.is_ok())
            } else {
                tokio::time::timeout(
                    Duration::from_millis(500),
                    tokio::net::TcpStream::connect((std::net::Ipv4Addr::LOCALHOST, port as u16)),
                )
                .await
                .is_ok_and(|v| v.is_ok())
            };
            result["readiness"] = json!({"port":port,"tcp_accepting":connected,"process_running":process.state.lock().await.status == "running","scope":"TCP availability only; verify application behavior separately"});
        }
        Ok(result)
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
            if let Some(remote) = &process.execution {
                remote.resize(rows, columns).await?;
            }
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
    #[tokio::test]
    async fn legacy_uuid_logs_remain_readable_and_owner_checked() {
        let (_dir, processes) = fixture();
        let id = "11111111-2222-4333-8444-555555555555";
        let (manifest, log) = processes.paths(id).unwrap();
        std::fs::write(manifest, json!({"process_id":id,"owner":"original-task","cwd":"/tmp","tty":false,"persistent":false,"status":"completed","exit_code":0,"log_complete":true}).to_string()).unwrap();
        std::fs::write(log, "legacy output").unwrap();
        let output = processes
            .output("original-task", id, &json!({}))
            .await
            .unwrap();
        assert_eq!(output["output"], "legacy output");
        assert!(
            processes
                .output("another-task", id, &json!({}))
                .await
                .is_err()
        );
        assert!(
            processes
                .paths("../11111111-2222-4333-8444-555555555555")
                .is_err()
        );
    }

    fn fixture() -> (tempfile::TempDir, Processes) {
        let dir = tempfile::tempdir().unwrap();
        let redactor = Arc::new(Redactor::default());
        redactor.add("private-test-value");
        let processes =
            Processes::new(dir.path().join("logs"), redactor, CancellationToken::new()).unwrap();
        (dir, processes)
    }
    #[tokio::test]
    async fn git_identity_survives_directory_changes_in_pipe_and_terminal() {
        let (dir, mut processes) = fixture();
        let home = dir.path().join("account-home");
        std::fs::create_dir(&home).unwrap();
        std::fs::write(
            home.join(".gitconfig"),
            "[user]\nname = Test Account\nemail = test@example.invalid\n",
        )
        .unwrap();
        processes.home = home;
        for tty in [false, true] {
            let cwd = dir
                .path()
                .join(if tty { "terminal-repo" } else { "pipe-repo" });
            std::fs::create_dir_all(cwd.join("nested")).unwrap();
            let broken = std::process::Command::new("git")
                .args(["var", "GIT_COMMITTER_IDENT"])
                .current_dir(&cwd)
                .env_clear()
                .envs(fenix_execution::local_environment(&cwd))
                .output()
                .unwrap();
            assert!(
                !broken.status.success(),
                "cwd must not supply account identity"
            );
            let run = processes.start(
                "task",
                "git init -q -b main && git commit --allow-empty -qm 'Check account identity' && cd nested && git var GIT_COMMITTER_IDENT && git status --porcelain",
                &cwd, 10, &CancellationToken::new(), None,
                ProcessOptions { tty, ..Default::default() },
            ).await.unwrap();
            let result = complete(&processes, run["process_id"].as_str().unwrap()).await;
            assert_eq!(result["exit_code"], 0, "{result}");
            assert!(
                result["output"]
                    .as_str()
                    .unwrap()
                    .contains("Test Account <test@example.invalid>"),
                "{result}"
            );
            assert!(!cwd.join(".gitconfig").exists());
        }
    }
    #[tokio::test]
    async fn quiet_wait_replays_the_prompt_without_consuming_output_or_answering() {
        let (dir, processes) = fixture();
        let cancel = CancellationToken::new();
        let run = processes
            .start(
                "task",
                "printf 'Choose a region: '; read -r answer; printf 'selected=%s\\n' \"$answer\"",
                dir.path(),
                10,
                &cancel,
                None,
                ProcessOptions::default(),
            )
            .await
            .unwrap();
        let id = run["process_id"].as_str().unwrap();
        // Consume the prompt through the normal incremental reader first.
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                let page = processes
                    .status("task", "task", id, &json!({"wait_ms":0}))
                    .await
                    .unwrap();
                if page["output"]
                    .as_str()
                    .unwrap()
                    .contains("Choose a region:")
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        for _ in 0..2 {
            let event = processes
                .wait_event(
                    "task",
                    id,
                    &cancel,
                    Duration::from_secs(1),
                    Duration::from_millis(30),
                )
                .await
                .unwrap();
            assert_eq!(event["reason"], "quiet_process");
            assert_eq!(event["recent_output_is_replayed"], true);
            assert!(
                event["recent_output"]
                    .as_str()
                    .unwrap()
                    .contains("Choose a region:")
            );
            let page = processes
                .status("task", "task", id, &json!({"wait_ms":0}))
                .await
                .unwrap();
            assert_eq!(page["status"], "running");
            assert_eq!(page["output"], "");
        }
        assert!(
            processes
                .wait_event(
                    "other-task",
                    id,
                    &cancel,
                    Duration::from_secs(1),
                    Duration::from_millis(30)
                )
                .await
                .is_err()
        );
        processes
            .input("task", id, &json!({"text":"known-setting\n"}), &cancel)
            .await
            .unwrap();
        let result = complete(&processes, id).await;
        assert_eq!(result["exit_code"], 0);
        assert!(
            result["output"]
                .as_str()
                .unwrap()
                .contains("selected=known-setting")
        );
    }

    #[tokio::test]
    async fn event_wait_finishes_or_returns_advisory_without_killing() {
        let (dir, processes) = fixture();
        let cancel = CancellationToken::new();
        let run = processes
            .start(
                "task",
                "sleep 0.1; printf done",
                dir.path(),
                10,
                &cancel,
                None,
                ProcessOptions::default(),
            )
            .await
            .unwrap();
        let id = run["process_id"].as_str().unwrap();
        let event = processes
            .wait_event(
                "task",
                id,
                &cancel,
                Duration::from_secs(2),
                Duration::from_secs(1),
            )
            .await
            .unwrap();
        assert_eq!(event["reason"], "process_ended");
        assert!(
            processes
                .status("task", "task", id, &json!({"wait_ms":0}))
                .await
                .unwrap()["output"]
                .as_str()
                .unwrap()
                .contains("done")
        );
        let run = processes
            .start(
                "task",
                "sleep 10",
                dir.path(),
                20,
                &cancel,
                None,
                ProcessOptions::default(),
            )
            .await
            .unwrap();
        let id = run["process_id"].as_str().unwrap();
        let event = processes
            .wait_event(
                "task",
                id,
                &cancel,
                Duration::from_secs(1),
                Duration::from_millis(30),
            )
            .await
            .unwrap();
        assert_eq!(event["reason"], "quiet_process");
        assert_eq!(
            processes
                .status("task", "task", id, &json!({"wait_ms":0}))
                .await
                .unwrap()["status"],
            "running"
        );
        let event = processes
            .wait_event(
                "task",
                id,
                &cancel,
                Duration::from_millis(30),
                Duration::from_secs(1),
            )
            .await
            .unwrap();
        assert_eq!(event["reason"], "review_due");
        let stopped = CancellationToken::new();
        stopped.cancel();
        assert!(
            processes
                .wait_event(
                    "task",
                    id,
                    &stopped,
                    Duration::from_secs(1),
                    Duration::from_secs(1)
                )
                .await
                .is_err()
        );
        assert!(
            processes
                .wait_event(
                    "another-task",
                    id,
                    &cancel,
                    Duration::from_secs(1),
                    Duration::from_secs(1)
                )
                .await
                .is_err()
        );
        processes.cancel("task", id).await.unwrap();
    }
    async fn complete(processes: &Processes, id: &str) -> Value {
        processes
            .status("task", "task", id, &json!({"wait_ms":5000}))
            .await
            .unwrap()
    }
    #[tokio::test]
    async fn persistent_background_job_survives_shell_launch_and_is_cancelled() {
        let (dir, processes) = fixture();
        let cancel = CancellationToken::new();
        let run = processes
            .start(
                "task",
                "sleep 60 & printf '%s\\n' $!",
                dir.path(),
                60,
                &cancel,
                None,
                ProcessOptions {
                    tty: false,
                    persistent: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let id = run["process_id"].as_str().unwrap();
        let status = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let status = processes
                    .status("task", "task", id, &json!({"wait_ms":2000}))
                    .await
                    .unwrap();
                if !status["output"].as_str().unwrap().trim().is_empty() {
                    break status;
                }
            }
        })
        .await
        .unwrap();
        assert_eq!(status["status"], "running");
        let child: u32 = status["output"].as_str().unwrap().trim().parse().unwrap();
        assert!(std::path::Path::new(&format!("/proc/{child}")).exists());
        cancel.cancel();
        processes.cancel_foreground("task").await;
        assert_eq!(processes.info(id).await.unwrap().status, "running");
        processes.cancel_all().await;
        assert_eq!(processes.info(id).await.unwrap().status, "cancelled");
        // A reparented dead child may briefly remain a zombie; it is not executing.
        if let Ok(stat) = std::fs::read_to_string(format!("/proc/{child}/stat")) {
            assert!(
                stat.split(')')
                    .nth(1)
                    .unwrap()
                    .trim_start()
                    .starts_with('Z')
            );
        }
    }
    #[tokio::test]
    async fn effects_wait_for_foreground_completion_without_cancelling_it() {
        let (dir, processes) = fixture();
        let cancel = CancellationToken::new();
        let run = processes
            .start(
                "task",
                "sleep 0.2; printf done",
                dir.path(),
                10,
                &cancel,
                None,
                ProcessOptions::default(),
            )
            .await
            .unwrap();
        processes.wait_foreground("task", &cancel).await.unwrap();
        let result = complete(&processes, run["process_id"].as_str().unwrap()).await;
        assert_eq!(result["exit_code"], 0);
        assert_eq!(result["output"], "done");
    }
    #[tokio::test]
    async fn readiness_is_explicit_tcp_evidence_not_application_success() {
        let (dir, processes) = fixture();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let run = processes
            .start(
                "task",
                "sleep 60",
                dir.path(),
                60,
                &CancellationToken::new(),
                None,
                ProcessOptions {
                    tty: false,
                    persistent: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let id = run["process_id"].as_str().unwrap();
        let ready = processes
            .status("task", "task", id, &json!({"wait_ms":0,"ready_port":port}))
            .await
            .unwrap();
        assert_eq!(ready["readiness"]["tcp_accepting"], true);
        drop(listener);
        let absent = processes
            .status("task", "task", id, &json!({"wait_ms":0,"ready_port":port}))
            .await
            .unwrap();
        assert_eq!(absent["readiness"]["tcp_accepting"], false);
        processes.cancel_all().await;
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
                        ..Default::default()
                    },
                )
                .await
                .unwrap();
            let id = run["process_id"].as_str().unwrap();
            if tty {
                let preview = processes
                    .status("task", "task", id, &json!({"wait_ms":2000}))
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
    async fn invalid_utf8_does_not_discard_a_buffered_secret_label() {
        let (dir, processes) = fixture();
        let run = processes
            .start(
                "task",
                "printf '\\377password: '; sleep 0.1; printf 'synthetic-private-value\\n'",
                dir.path(),
                10,
                &CancellationToken::new(),
                None,
                ProcessOptions::default(),
            )
            .await
            .unwrap();
        let result = complete(&processes, run["process_id"].as_str().unwrap()).await;
        assert_eq!(result["exit_code"], 0);
        assert!(
            !result["output"]
                .as_str()
                .unwrap()
                .contains("synthetic-private-value")
        );
        assert!(result["output"].as_str().unwrap().contains("[REDACTED]"));
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
                    ..Default::default()
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
        let output = tokio::time::timeout(Duration::from_secs(5), async {
            let mut output = String::new();
            loop {
                let result = processes
                    .status("task", "later-turn", id, &json!({"wait_ms":100}))
                    .await
                    .unwrap();
                output.push_str(result["output"].as_str().unwrap());
                if output.contains("alive") {
                    break output;
                }
            }
        })
        .await
        .unwrap();
        assert!(output.contains("alive"));
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

#[cfg(test)]
mod resource_tests {
    use super::*;
    #[tokio::test]
    async fn declared_readers_allow_independent_writes_and_hold_conflicts() {
        let dir = tempfile::tempdir().unwrap();
        let stop = CancellationToken::new();
        let processes = Processes::new(
            dir.path().join("logs"),
            Arc::new(Redactor::default()),
            stop.clone(),
        );
        // The kernel helper is tested through the real CLI; this test isolates scheduling.
        let processes = processes.unwrap();
        let input = dir.path().join("input.db");
        std::fs::write(&input, "data").unwrap();
        let started = processes
            .start(
                "owner",
                "sleep 2",
                dir.path(),
                10,
                &stop,
                None,
                ProcessOptions {
                    read_paths: vec![input.clone()],
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        processes
            .wait_effect("owner", Some(&[dir.path().join("answer.sql")]), &stop)
            .await
            .unwrap();
        assert!(
            tokio::time::timeout(
                Duration::from_millis(50),
                processes.wait_effect("other", Some(&[input]), &stop)
            )
            .await
            .is_err()
        );
        processes
            .cancel("owner", started["process_id"].as_str().unwrap())
            .await
            .unwrap();
    }
}
