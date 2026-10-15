use anyhow::{Context, Result};
use tokio::process::Child;
/// A lifecycle owner, never an isolation or permission boundary.
pub struct ProcessTree {
    #[cfg(unix)]
    pid: u32,
    #[cfg(unix)]
    terminal: bool,
    #[cfg(windows)]
    job: Option<win32job::Job>,
}
impl ProcessTree {
    pub(crate) fn pipe(child: &Child) -> Result<Self> {
        #[cfg(unix)]
        {
            Ok(Self {
                pid: child.id().context("Missing process ID")?,
                terminal: false,
            })
        }
        #[cfg(windows)]
        {
            Self::job(child.raw_handle().context("Missing process handle")? as isize)
        }
    }
    pub(crate) fn terminal(child: &dyn portable_pty::Child) -> Result<Self> {
        #[cfg(unix)]
        {
            Ok(Self {
                pid: child.process_id().context("Missing terminal process ID")?,
                terminal: true,
            })
        }
        #[cfg(windows)]
        {
            Self::job(
                child
                    .as_raw_handle()
                    .context("Missing terminal process handle")? as isize,
            )
        }
    }
    #[cfg(windows)]
    fn job(handle: isize) -> Result<Self> {
        let job = win32job::Job::create()?;
        let mut info = job.query_extended_limit_info()?;
        info.limit_kill_on_job_close();
        job.set_extended_limit_info(&info)?;
        job.assign_process(handle)?;
        Ok(Self { job: Some(job) })
    }
    /// A Windows shell can exit while Start-Process descendants are still alive.
    /// Keep explicitly persistent sessions owned until their job is empty.
    pub async fn wait_for_descendants(&self) -> Result<()> {
        #[cfg(windows)]
        if let Some(job) = &self.job {
            while !job.query_process_id_list()?.is_empty() {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
        Ok(())
    }
    pub fn terminate(&mut self) {
        #[cfg(windows)]
        {
            self.job.take();
        }
        #[cfg(unix)]
        {
            use nix::{
                sys::signal::{Signal, kill, killpg},
                unistd::{Pid, getsid},
            };
            if self.pid == 0 {
                return;
            }
            if self.terminal {
                // PTY foreground jobs may have their own group. Match session IDs,
                // including jobs reparented after the shell exits. No Linux /proc requirement.
                if let Ok(output) = std::process::Command::new("/bin/ps")
                    .args(["-axo", "pid="])
                    .output()
                {
                    for pid in String::from_utf8_lossy(&output.stdout)
                        .split_whitespace()
                        .filter_map(|v| v.parse::<i32>().ok())
                    {
                        if getsid(Some(Pid::from_raw(pid)))
                            .is_ok_and(|sid| sid.as_raw() == self.pid as i32)
                        {
                            let _ = kill(Pid::from_raw(pid), Signal::SIGKILL);
                        }
                    }
                }
            }
            let _ = killpg(Pid::from_raw(self.pid as i32), Signal::SIGKILL);
            self.pid = 0;
        }
    }
}
impl Drop for ProcessTree {
    fn drop(&mut self) {
        self.terminate();
    }
}
