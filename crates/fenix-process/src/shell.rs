use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::process::Command;

#[derive(Clone, Debug, Serialize)]
pub struct Shell {
    pub program: PathBuf,
    pub kind: &'static str,
}
#[derive(Clone, Debug, Serialize)]
pub struct ShellInfo {
    pub shell: Shell,
    pub version: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct ShellDiscovery {
    pub selected: ShellInfo,
    pub available: Vec<ShellInfo>,
}
impl Shell {
    pub async fn detect() -> Result<Self> {
        Ok(Self::discover().await?.selected.shell)
    }
    /// Inspect installed interpreters once per runtime; SHELL/COMSPEC are hints, not evidence.
    pub async fn discover() -> Result<ShellDiscovery> {
        crate::require_external_processes()?;
        let configured = std::env::var_os("FENIX_SHELL");
        let mut candidates = Vec::new();
        if let Some(value) = &configured {
            let path = PathBuf::from(value);
            let kind = shell_kind(&path).context(
                "FENIX_SHELL must name bash, zsh, sh, dash, ash, pwsh, powershell or cmd",
            )?;
            candidates.push(Shell {
                program: resolve(&path).context("Configured shell was not found")?,
                kind,
            });
        }
        let names: &[&str] = if cfg!(target_os = "android") {
            &["sh", "bash", "zsh"]
        } else if cfg!(windows) {
            &["pwsh", "powershell", "cmd"]
        } else if cfg!(target_os = "macos") {
            &["zsh", "bash", "sh"]
        } else {
            &["bash", "zsh", "sh", "dash", "ash"]
        };
        for name in names {
            if let Some(program) = resolve(Path::new(name)) {
                if !candidates
                    .iter()
                    .any(|candidate| candidate.program == program)
                {
                    candidates.push(Shell {
                        program,
                        kind: shell_kind(Path::new(name)).unwrap(),
                    });
                }
            }
        }
        discover_candidates(candidates, configured.is_some()).await
    }
    pub fn args<'a>(&self, script: &'a str) -> Vec<&'a str> {
        match self.kind {
            "powershell" => vec![
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                script,
            ],
            "cmd" => vec!["/D", "/S", "/C", script],
            "bash" => vec!["--noprofile", "--norc", "-c", script],
            "zsh" => vec!["-f", "-c", script],
            _ => vec!["-c", script],
        }
    }
    pub fn script(&self, command: &str, persistent: bool) -> String {
        if self.kind == "powershell" {
            // Distinguish PowerShell errors from native executable exit status.
            // Explicit exit remains authoritative. Jobs stay supervised for persistent sessions.
            format!(
                "$ErrorActionPreference = 'Stop'; $OutputEncoding = [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false); $global:LASTEXITCODE = 0; try {{\n{command}\n$fenixSucceeded = $?; $fenixCode = $global:LASTEXITCODE; {}\nif (-not $fenixSucceeded -and $fenixCode -eq 0) {{ exit 1 }}; exit $fenixCode\n}} catch {{ [Console]::Error.WriteLine($_.ToString()); exit 1 }}",
                if persistent {
                    "Get-Job | Wait-Job | Receive-Job"
                } else {
                    ""
                }
            )
        } else if self.kind == "cmd" {
            command.to_owned()
        } else if persistent {
            format!("{command}\n_fenix_command_status=$?\nwait\nexit \"$_fenix_command_status\"")
        } else {
            command.to_owned()
        }
    }
    pub fn command(&self, script: &str) -> Command {
        let mut c = Command::new(&self.program);
        c.args(self.args(script));
        c
    }
}

fn shell_kind(path: &Path) -> Option<&'static str> {
    let name = path.file_name()?.to_str()?.to_ascii_lowercase();
    match name.strip_suffix(".exe").unwrap_or(&name) {
        "bash" => Some("bash"),
        "zsh" => Some("zsh"),
        "sh" | "dash" | "ash" => Some("sh"),
        "pwsh" | "powershell" => Some("powershell"),
        "cmd" => Some("cmd"),
        _ => None,
    }
}
pub fn resolve(path: &Path) -> Option<PathBuf> {
    if path.components().count() > 1 || path.is_absolute() {
        return path
            .is_file()
            .then(|| std::path::absolute(path).ok())
            .flatten();
    }
    let mut directories: Vec<PathBuf> =
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
            .filter(|p| p.is_absolute())
            .collect();
    if cfg!(windows) {
        let root =
            PathBuf::from(std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:\Windows".into()));
        directories.push(root.join("System32/WindowsPowerShell/v1.0"));
        directories.push(root.join("System32"));
    } else if cfg!(target_os = "android") {
        directories.push("/system/bin".into());
    } else {
        directories.extend([PathBuf::from("/bin"), PathBuf::from("/usr/bin")]);
    }
    for directory in directories {
        let executable = directory.join(path);
        if executable.is_file() {
            return Some(executable);
        }
        if cfg!(windows) && path.extension().is_none() {
            let executable = executable.with_extension("exe");
            if executable.is_file() {
                return Some(executable);
            }
        }
    }
    None
}
async fn discover_candidates(candidates: Vec<Shell>, explicit: bool) -> Result<ShellDiscovery> {
    let mut available = Vec::new();
    for (index, shell) in candidates.into_iter().enumerate() {
        match probe(&shell).await {
            Ok(version) => available.push(ShellInfo { shell, version }),
            Err(error) if explicit && index == 0 => {
                return Err(error
                    .context("Configured shell failed verification; no automatic substitution"));
            }
            Err(_) => {}
        }
    }
    let selected = available
        .first()
        .cloned()
        .context("No working supported shell is available; install one or set FENIX_SHELL")?;
    Ok(ShellDiscovery {
        selected,
        available,
    })
}
async fn probe(shell: &Shell) -> Result<String> {
    let script = match shell.kind {
        "bash" => "test -n \"$BASH_VERSION\" && printf 'FENIX_SHELL:bash:%s' \"$BASH_VERSION\"",
        "zsh" => "test -n \"$ZSH_VERSION\" && printf 'FENIX_SHELL:zsh:%s' \"$ZSH_VERSION\"",
        "powershell" => {
            "[Console]::Write('FENIX_SHELL:powershell:' + $PSVersionTable.PSVersion.ToString())"
        }
        "cmd" => "echo FENIX_SHELL:cmd: & ver",
        _ => "printf 'FENIX_SHELL:sh:POSIX-compatible shell; version not advertised'",
    };
    let home = std::env::home_dir().unwrap_or_else(std::env::temp_dir);
    let mut command = shell.command(script);
    command
        .env_clear()
        .envs(local_environment(&home))
        .current_dir(&home)
        .stdin(std::process::Stdio::null())
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    let output = tokio::time::timeout(Duration::from_secs(2), command.output())
        .await
        .context("Shell verification timed out")??;
    ensure!(output.status.success(), "Shell verification failed");
    let text = String::from_utf8_lossy(&output.stdout);
    let marker = format!("FENIX_SHELL:{}:", shell.kind);
    let version = text
        .trim()
        .strip_prefix(&marker)
        .context("Interpreter did not identify itself")?
        .trim();
    ensure!(
        !version.is_empty(),
        "Interpreter returned no version information"
    );
    Ok(version.chars().take(200).collect())
}
/// Copy executable-search and OS essentials only; never provider tokens/startup hooks.
pub fn local_environment(home: &Path) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    for name in [
        "PATH",
        "SystemRoot",
        "SystemDrive",
        "COMSPEC",
        "APPDATA",
        "LOCALAPPDATA",
        "WINDIR",
        "PATHEXT",
        "TEMP",
        "TMP",
        "TMPDIR",
    ] {
        if let Some(value) = std::env::var_os(name) {
            env.insert(name.into(), value.to_string_lossy().into_owned());
        }
    }
    env.entry("PATH".into()).or_insert_with(|| {
        if cfg!(windows) {
            "C:\\Windows\\System32".into()
        } else if cfg!(target_os = "android") {
            "/system/bin:/system/xbin".into()
        } else {
            "/usr/local/bin:/usr/bin:/bin".into()
        }
    });
    env.insert("HOME".into(), home.to_string_lossy().into_owned());
    #[cfg(windows)]
    env.insert("USERPROFILE".into(), home.to_string_lossy().into_owned());
    env.insert("LANG".into(), "en_US.UTF-8".into());
    env.insert("TERM".into(), "xterm-256color".into());
    env
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arguments_match_each_interpreter_without_splitting_the_command() {
        for (kind, prefix) in [
            ("bash", vec!["--noprofile", "--norc", "-c"]),
            ("zsh", vec!["-f", "-c"]),
            ("sh", vec!["-c"]),
            ("cmd", vec!["/D", "/S", "/C"]),
        ] {
            let shell = Shell {
                program: kind.into(),
                kind,
            };
            let command = "echo a & echo b";
            let args = shell.args(command);
            assert_eq!(&args[..args.len() - 1], prefix.as_slice());
            assert_eq!(args.last(), Some(&command));
        }
    }
    #[tokio::test]
    async fn discovery_verifies_the_selected_shell_and_records_alternatives() {
        let discovery = Shell::discover().await.unwrap();
        assert!(!discovery.available.is_empty());
        assert!(!discovery.selected.version.is_empty());
        assert!(discovery.selected.shell.program.is_absolute());
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn automatic_fallback_does_not_override_an_explicit_broken_shell() {
        let candidates = vec![
            Shell {
                program: "/fenix-nonexistent-shell".into(),
                kind: "bash",
            },
            Shell {
                program: "/bin/sh".into(),
                kind: "sh",
            },
        ];
        assert_eq!(
            discover_candidates(candidates.clone(), false)
                .await
                .unwrap()
                .selected
                .shell
                .kind,
            "sh"
        );
        assert!(discover_candidates(candidates, true).await.is_err());
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn probes_have_a_deadline_and_do_not_accept_a_wrong_interpreter() {
        let wrong = Shell {
            program: "/bin/echo".into(),
            kind: "bash",
        };
        assert!(probe(&wrong).await.is_err());
        let directory = tempfile::tempdir().unwrap();
        let program = directory.path().join("sh");
        std::fs::write(&program, "#!/bin/sh\nexec sleep 30\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
        let start = std::time::Instant::now();
        assert!(
            probe(&Shell {
                program,
                kind: "sh"
            })
            .await
            .is_err()
        );
        assert!(start.elapsed() < Duration::from_secs(5));
    }
}
