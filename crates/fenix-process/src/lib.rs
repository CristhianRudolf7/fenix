//! Shared native process/terminal execution. This is supervision, not a sandbox.
mod shell;
pub mod shutdown;
mod terminal;
pub mod transport;
mod tree;
use anyhow::{Context, Result};
pub use shell::{Shell, ShellDiscovery, ShellInfo, local_environment, resolve as resolve_program};
use std::path::Path;
pub use terminal::Terminal;
use tokio::{
    io::AsyncRead,
    process::{Child, ChildStdin, Command},
};
pub use tree::ProcessTree;

/// iOS applications execute bundled operations in-process, not OS child processes.
pub const fn external_processes_supported() -> bool {
    !cfg!(target_os = "ios")
}
pub fn require_external_processes() -> Result<()> {
    anyhow::ensure!(
        external_processes_supported(),
        "External processes and system shells are unavailable in an iOS application; use in-process native operations"
    );
    Ok(())
}

pub type Reader = Box<dyn AsyncRead + Send + Unpin>;
pub enum Input {
    Pipe(ChildStdin),
    Terminal(Terminal),
}
pub enum RunningChild {
    Pipe(Child),
    Terminal {
        done: tokio::sync::oneshot::Receiver<std::io::Result<portable_pty::ExitStatus>>,
        killer: Box<dyn portable_pty::ChildKiller + Send + Sync>,
        code: Option<std::result::Result<Option<i32>, String>>,
    },
}
pub struct Spawned {
    pub child: RunningChild,
    pub input: Input,
    pub readers: Vec<Reader>,
    pub tree: ProcessTree,
}
impl RunningChild {
    /// Cancellation-safe completion: a PTY wait happens once in a blocking worker.
    pub async fn wait(&mut self) -> Result<Option<i32>> {
        match self {
            Self::Pipe(child) => Ok(child.wait().await?.code()),
            Self::Terminal { done, code, .. } => {
                if code.is_none() {
                    *code = Some(match done.await {
                        Ok(Ok(status)) => Ok(if status.signal().is_some() {
                            None
                        } else {
                            Some(status.exit_code() as i32)
                        }),
                        Ok(Err(error)) => Err(error.to_string()),
                        Err(error) => Err(error.to_string()),
                    });
                }
                match code.as_ref().expect("Stored terminal outcome") {
                    Ok(code) => Ok(*code),
                    Err(error) => anyhow::bail!("Terminal wait failed: {error}"),
                }
            }
        }
    }
    pub fn kill(&mut self) -> Result<()> {
        match self {
            Self::Pipe(child) => child.start_kill()?,
            Self::Terminal {
                code: Some(Ok(_)), ..
            } => {}
            Self::Terminal { killer, .. } => killer.kill()?,
        }
        Ok(())
    }
}
impl Drop for RunningChild {
    fn drop(&mut self) {
        let _ = self.kill();
    }
}

/// Spawn an executable with separate arguments (no shell interpretation).
/// The caller supplies cwd, environment and stdio; this always supervises descendants.
pub fn spawn_program(mut command: Command) -> Result<(Child, ProcessTree)> {
    require_external_processes()?;
    command.kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x08000000); // CREATE_NO_WINDOW for background pipe processes.
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command.spawn().context("Could not start executable")?;
    let tree = match ProcessTree::pipe(&child) {
        Ok(tree) => tree,
        Err(error) => {
            let _ = child.start_kill();
            return Err(error);
        }
    };
    Ok((child, tree))
}

pub fn spawn_pipe(mut command: Command) -> Result<Spawned> {
    command
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let (mut child, tree) = spawn_program(command)?;
    let input = Input::Pipe(child.stdin.take().context("Missing stdin")?);
    let readers: Vec<Reader> = vec![
        Box::new(child.stdout.take().context("Missing stdout")?),
        Box::new(child.stderr.take().context("Missing stderr")?),
    ];
    Ok(Spawned {
        child: RunningChild::Pipe(child),
        input,
        readers,
        tree,
    })
}
pub fn spawn_shell(
    shell: &Shell,
    command: &str,
    persistent: bool,
    tty: bool,
    cwd: &Path,
    home: &Path,
) -> Result<Spawned> {
    require_external_processes()?;
    let script = shell.script(command, persistent);
    if tty {
        terminal::spawn(shell, &script, cwd, home)
    } else {
        let mut builder = shell.command(&script);
        builder
            .current_dir(cwd)
            .env_clear()
            .envs(local_environment(home));
        spawn_pipe(builder)
    }
}
