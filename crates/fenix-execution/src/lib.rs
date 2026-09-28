//! Authenticated client for the host execution service. No Docker socket or keys
//! are exposed to workload processes. Standalone mode is explicitly separate.
pub mod fs;
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::UnixStream,
    sync::Mutex,
};
use zeroize::Zeroizing;

const MAX_FRAME: usize = 65_536;
const MAX_CONTROL: usize = 1_048_576;

#[derive(Clone)]
pub struct Executor {
    socket: PathBuf,
    account: String,
    credential: Zeroizing<String>,
    auth_epoch: u64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Specification {
    pub task: String,
    pub kind: String,
    pub argv: Vec<String>,
    pub cwd: PathBuf,
    pub workspace: PathBuf,
    pub timeout: u64,
    pub tty: bool,
    pub read_paths: Vec<PathBuf>,
    #[serde(default)]
    pub resource_paths: Vec<PathBuf>,
    pub environment: BTreeMap<String, String>,
    pub profile: String,
}

impl Specification {
    pub fn new(
        task: &str,
        kind: &str,
        argv: Vec<String>,
        cwd: &Path,
        workspace: &Path,
        timeout: u64,
    ) -> Self {
        Self {
            task: task.into(),
            kind: kind.into(),
            argv,
            cwd: cwd.into(),
            workspace: workspace.into(),
            timeout,
            tty: false,
            read_paths: vec![],
            resource_paths: vec![],
            environment: BTreeMap::new(),
            profile: "development".into(),
        }
    }
}

/// Lifetime ownership is explicit: dropping the lease cancels the whole remote
/// container; the service also kills it on attachment loss, expiry or revocation.
pub struct Lease {
    executor: Executor,
    pub id: String,
    capability: Zeroizing<String>,
    pub timeout: u64,
    suspended: AtomicBool,
}

impl Executor {
    pub fn from_environment() -> Result<Option<Self>> {
        let required = std::env::var("FENIX_CONTAINER").as_deref() == Ok("1")
            || std::env::var("FENIX_EXECUTION_REQUIRED").as_deref() == Ok("1");
        let mut configured = Self::configured(
            std::env::var_os("FENIX_EXECUTION_SOCKET").map(PathBuf::from),
            std::env::var("FENIX_USER_ID").ok(),
            std::env::var("FENIX_UPSTREAM_TOKEN").ok(),
            required,
        )?;
        if let Some(executor) = configured.as_mut() {
            executor.auth_epoch = std::env::var("FENIX_AUTH_EPOCH")
                .context("Account authorization revision is missing")?
                .parse()?;
        }
        Ok(configured)
    }

    pub fn configured(
        socket: Option<PathBuf>,
        account: Option<String>,
        credential: Option<String>,
        required: bool,
    ) -> Result<Option<Self>> {
        let Some(socket) = socket else {
            ensure!(
                !required,
                "Multi-user execution requires the isolated execution service; host fallback is disabled"
            );
            return Ok(None);
        };
        ensure!(
            socket.is_absolute(),
            "Execution socket must be an absolute path"
        );
        let account = account.context("Execution account identity is missing")?;
        let credential = credential.context("Execution credential is missing")?;
        ensure!(
            account.len() == 36 && account.bytes().all(|c| c.is_ascii_hexdigit() || c == b'-'),
            "Invalid execution account"
        );
        ensure!(
            (32..=256).contains(&credential.len())
                && credential
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-'),
            "Invalid execution credential"
        );
        Ok(Some(Self {
            socket,
            account,
            credential: Zeroizing::new(credential),
            auth_epoch: 0,
        }))
    }

    async fn open(&self, mut request: Value) -> Result<(UnixStream, Value)> {
        request["account_id"] = json!(self.account);
        request["credential"] = json!(self.credential.as_str());
        request["auth_epoch"] = json!(self.auth_epoch);
        let deadline = if request["operation"] == "browser_suspend" {
            240
        } else {
            90
        };
        let result = tokio::time::timeout(Duration::from_secs(deadline), async {
            let mut stream = UnixStream::connect(&self.socket)
                .await
                .context("Isolated execution service unavailable; no host fallback")?;
            write_json(&mut stream, &request).await?;
            let response = read_json(&mut stream).await?;
            ensure!(
                response["ok"] == true,
                "Isolated execution rejected: {}",
                response["error"].as_str().unwrap_or("policy unavailable")
            );
            Ok::<_, anyhow::Error>((stream, response))
        })
        .await
        .context("Execution service timed out")??;
        Ok(result)
    }

    pub async fn request(&self, request: Value) -> Result<Value> {
        Ok(self.open(request).await?.1)
    }

    pub async fn health(&self) -> Result<Value> {
        self.request(json!({"operation":"health"})).await
    }

    async fn admitted(&self, request: Value) -> Result<Lease> {
        let mut response = self.request(request).await?;
        let mut waiting = QueueTicket {
            executor: self.clone(),
            ticket: response["ticket"].as_str().map(str::to_owned),
        };
        while response["queued"] == true {
            let ticket = waiting
                .ticket
                .as_deref()
                .context("Missing admission ticket")?;
            tokio::time::sleep(Duration::from_millis(1000)).await;
            response = self
                .request(json!({"operation":"admission_poll","ticket":ticket}))
                .await?;
        }
        waiting.ticket = None;
        let id = response["id"]
            .as_str()
            .context("Missing execution identity")?
            .to_owned();
        let capability = response["capability"]
            .as_str()
            .context("Missing execution capability")?
            .to_owned();
        ensure!(
            id.len() == 36 && capability.len() >= 32,
            "Invalid execution grant"
        );
        Ok(Lease {
            executor: self.clone(),
            id,
            capability: Zeroizing::new(capability),
            timeout: response["timeout"]
                .as_u64()
                .context("Missing execution deadline")?,
            suspended: AtomicBool::new(false),
        })
    }

    pub async fn create(&self, spec: Specification) -> Result<Lease> {
        self.admitted(json!({"operation":"create","spec":spec,"enqueue":true}))
            .await
    }

    pub async fn resume_browser(&self, id: &str) -> Result<Lease> {
        self.admitted(json!({"operation":"browser_resume","id":id}))
            .await
    }

    pub async fn cancel_browser(&self, id: &str) -> Result<()> {
        self.request(json!({"operation":"cancel","id":id})).await?;
        Ok(())
    }
}

struct QueueTicket {
    executor: Executor,
    ticket: Option<String>,
}
impl Drop for QueueTicket {
    fn drop(&mut self) {
        if let Some(ticket) = self.ticket.take() {
            let executor = self.executor.clone();
            if let Ok(runtime) = tokio::runtime::Handle::try_current() {
                runtime.spawn(async move {
                    let _ = executor
                        .request(json!({"operation":"admission_cancel","ticket":ticket}))
                        .await;
                });
            }
        }
    }
}

impl Lease {
    pub async fn suspend_browser(&self) -> Result<()> {
        let result = self
            .executor
            .request(json!({"operation":"browser_suspend","id":self.id}))
            .await;
        // Cancellation must not delete a successfully committed checkpoint.
        if result.is_ok()
            || self
                .executor
                .request(json!({"operation":"status","id":self.id}))
                .await
                .is_ok_and(|v| v["status"] == "suspended")
        {
            self.suspended.store(true, Ordering::Release);
        }
        result?;
        Ok(())
    }

    pub fn is_suspended(&self) -> bool {
        self.suspended.load(Ordering::Acquire)
    }

    pub fn command(&self) -> Result<tokio::process::Command> {
        let mut command = tokio::process::Command::new(std::env::current_exe()?);
        command
            .arg("internal-runner")
            .env_clear()
            .env("FENIX_RUNNER_SOCKET", &self.executor.socket)
            .env("FENIX_RUNNER_ID", &self.id)
            .env("FENIX_RUNNER_CAPABILITY", self.capability.as_str());
        Ok(command)
    }

    pub fn relay_environment(&self) -> [(String, String); 3] {
        [
            (
                "FENIX_RUNNER_SOCKET".into(),
                self.executor.socket.to_string_lossy().into_owned(),
            ),
            ("FENIX_RUNNER_ID".into(), self.id.clone()),
            (
                "FENIX_RUNNER_CAPABILITY".into(),
                self.capability.to_string(),
            ),
        ]
    }

    pub async fn connect(&self, port: u16) -> Result<UnixStream> {
        ensure!(port >= 1024, "Privileged runner ports are not exposed");
        Ok(self
            .executor
            .open(json!({"operation":"connect", "id":self.id, "port":port}))
            .await?
            .0)
    }

    pub async fn cancel(&self) -> Result<()> {
        self.executor
            .request(json!({"operation":"cancel", "id":self.id}))
            .await?;
        Ok(())
    }

    pub async fn resize(&self, rows: u16, columns: u16) -> Result<()> {
        self.executor
            .request(json!({"operation":"resize", "id":self.id, "rows":rows, "columns":columns}))
            .await?;
        Ok(())
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        if self.is_suspended() {
            return;
        }
        let executor = self.executor.clone();
        let id = self.id.clone();
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn(async move {
                let _ = executor
                    .request(json!({"operation":"release", "id":id}))
                    .await;
            });
        }
    }
}

/// Only non-sensitive, fixed execution defaults. In particular no BASH_ENV,
/// LD_PRELOAD, language startup hooks, cloud identity, proxy or provider tokens.
pub fn local_environment(home: &Path) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("PATH".into(), "/usr/local/bin:/usr/bin:/bin".into()),
        ("HOME".into(), home.to_string_lossy().into_owned()),
        ("LANG".into(), "C.UTF-8".into()),
        ("TERM".into(), "xterm-256color".into()),
    ])
}

async fn write_json(stream: &mut (impl AsyncWrite + Unpin), value: &Value) -> Result<()> {
    let mut bytes = serde_json::to_vec(value)?;
    ensure!(bytes.len() < MAX_CONTROL, "Execution request too large");
    bytes.push(b'\n');
    stream.write_all(&bytes).await?;
    Ok(())
}

async fn read_json(stream: &mut (impl AsyncRead + Unpin)) -> Result<Value> {
    let mut bytes = Vec::new();
    loop {
        let byte = stream
            .read_u8()
            .await
            .context("Execution service disconnected")?;
        if byte == b'\n' {
            break;
        }
        ensure!(bytes.len() < MAX_CONTROL, "Execution response too large");
        bytes.push(byte);
    }
    Ok(serde_json::from_slice(&bytes)?)
}

async fn frame(writer: &mut (impl AsyncWrite + Unpin), kind: u8, bytes: &[u8]) -> Result<()> {
    ensure!(bytes.len() <= MAX_FRAME, "Execution frame too large");
    writer.write_u8(kind).await?;
    writer.write_u32(bytes.len() as u32).await?;
    writer.write_all(bytes).await?;
    writer.flush().await?;
    Ok(())
}

/// Internal fixed-function relay. It never executes a workload on the controller.
pub async fn relay() -> Result<i32> {
    let socket = std::env::var("FENIX_RUNNER_SOCKET").context("Missing relay socket")?;
    let id = std::env::var("FENIX_RUNNER_ID").context("Missing relay identity")?;
    let capability = Zeroizing::new(
        std::env::var("FENIX_RUNNER_CAPABILITY").context("Missing relay capability")?,
    );
    let mut stream = UnixStream::connect(socket).await?;
    write_json(
        &mut stream,
        &json!({"operation":"attach", "id":id, "capability":capability.as_str()}),
    )
    .await?;
    let reply = tokio::time::timeout(Duration::from_secs(90), read_json(&mut stream)).await??;
    ensure!(reply["ok"] == true, "Runner attachment rejected");
    let (mut read, write) = stream.into_split();
    let write = Arc::new(Mutex::new(write));
    let input = write.clone();
    let stdin = tokio::spawn(async move {
        let mut stdin = tokio::io::stdin();
        let mut bytes = vec![0; 16384];
        loop {
            let n = stdin.read(&mut bytes).await?;
            frame(
                &mut *input.lock().await,
                if n == 0 { 5 } else { 0 },
                &bytes[..n],
            )
            .await?;
            if n == 0 {
                return Ok::<_, anyhow::Error>(());
            }
        }
    });
    let heartbeat = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;
            frame(&mut *write.lock().await, 9, &[]).await?;
        }
        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(())
    });
    let result = async {
        let mut stdout = tokio::io::stdout();
        let mut stderr = tokio::io::stderr();
        loop {
            let kind = read.read_u8().await?;
            let length = read.read_u32().await? as usize;
            ensure!(length <= MAX_FRAME, "Runner output exceeds frame limit");
            let mut bytes = vec![0; length];
            read.read_exact(&mut bytes).await?;
            match kind {
                1 => {
                    stdout.write_all(&bytes).await?;
                    stdout.flush().await?;
                }
                2 => {
                    stderr.write_all(&bytes).await?;
                    stderr.flush().await?;
                }
                3 => return Ok(serde_json::from_slice::<i32>(&bytes)?),
                4 => bail!("Isolated execution interrupted by the service"),
                _ => bail!("Invalid runner output frame"),
            }
        }
    }
    .await;
    stdin.abort();
    heartbeat.abort();
    result
}

#[cfg(test)]
mod lifecycle_tests;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn required_execution_never_falls_back_to_host() {
        assert!(Executor::configured(None, None, None, true).is_err());
        assert!(
            Executor::configured(None, None, None, false)
                .unwrap()
                .is_none()
        );
        assert!(Executor::configured(Some("relative.sock".into()), None, None, false).is_err());
    }
    #[test]
    fn child_environment_is_an_allowlist() {
        let env = local_environment(Path::new("/work"));
        assert_eq!(env.len(), 4);
        for key in [
            "TOKEN",
            "AWS_SECRET_ACCESS_KEY",
            "BASH_ENV",
            "LD_PRELOAD",
            "PYTHONPATH",
            "HTTP_PROXY",
        ] {
            assert!(!env.contains_key(key));
        }
    }
    #[tokio::test]
    async fn unavailable_service_does_not_launch_a_process() {
        let dir = tempfile::tempdir().unwrap();
        let executor = Executor::configured(
            Some(dir.path().join("missing.sock")),
            Some("00000000-0000-4000-8000-000000000001".into()),
            Some("a".repeat(64)),
            true,
        )
        .unwrap()
        .unwrap();
        assert!(executor.health().await.is_err());
    }
}
