//! Tests execute actual native children; no providers or model calls.
use fenix_process::{Input, Shell, local_environment, spawn_pipe, spawn_shell};
use std::{
    io::{BufRead, Write},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};

#[test]
fn child_fixture() {
    let Ok(role) = std::env::var("FENIX_PROCESS_TEST_ROLE") else {
        return;
    };
    match role.as_str() {
        "echo" => {
            println!("stdout ✓");
            eprintln!("stderr ✓");
            std::process::exit(7);
        }
        "input" => {
            let mut s = String::new();
            std::io::stdin().lock().read_line(&mut s).unwrap();
            println!("received={}", s.trim());
        }
        "tree" => {
            let mut child = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "child_fixture", "--nocapture"])
                .env("FENIX_PROCESS_TEST_ROLE", "sleep")
                .spawn()
                .unwrap();
            println!("descendant={}", child.id());
            std::io::stdout().flush().unwrap();
            child.wait().unwrap();
        }
        "shutdown" => {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mut shutdown = fenix_process::shutdown::Shutdown::listen().unwrap();
                println!("shutdown_ready");
                std::io::stdout().flush().unwrap();
                shutdown.wait().await;
            });
        }
        "sleep" => std::thread::sleep(Duration::from_secs(60)),
        _ => panic!("Unknown fixture"),
    }
}
fn helper(role: &str, cwd: &std::path::Path) -> Command {
    let mut c = Command::new(std::env::current_exe().unwrap());
    c.args(["--exact", "child_fixture", "--nocapture"])
        .current_dir(cwd)
        .env_clear()
        .envs(local_environment(cwd))
        .env("FENIX_PROCESS_TEST_ROLE", role);
    c
}
#[tokio::test]
async fn direct_arguments_capture_both_streams_and_exit_code() {
    let dir = tempfile::tempdir().unwrap();
    let mut p = spawn_pipe(helper("echo", dir.path())).unwrap();
    drop(p.input);
    let mut streams = Vec::new();
    for mut reader in p.readers {
        streams.push(tokio::spawn(async move {
            let mut s = String::new();
            reader.read_to_string(&mut s).await.unwrap();
            s
        }));
    }
    assert_eq!(p.child.wait().await.unwrap(), Some(7));
    p.tree.terminate();
    let mut text = String::new();
    for s in streams {
        text.push_str(&s.await.unwrap());
    }
    assert!(text.contains("stdout ✓") && text.contains("stderr ✓"));
}
#[tokio::test]
async fn pipe_input_eof_and_cwd_with_spaces() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path().join("folder with spaces");
    std::fs::create_dir(&cwd).unwrap();
    let mut p = spawn_pipe(helper("input", &cwd)).unwrap();
    if let Input::Pipe(ref mut stdin) = p.input {
        stdin.write_all(b"hello\n").await.unwrap();
    }
    drop(p.input);
    assert_eq!(p.child.wait().await.unwrap(), Some(0));
    p.tree.terminate();
    let mut output = String::new();
    p.readers[0].read_to_string(&mut output).await.unwrap();
    assert!(output.contains("received=hello"));
}
#[tokio::test]
async fn native_shell_reports_failures_and_success() {
    let dir = tempfile::tempdir().unwrap();
    let shell = Shell::detect().await.unwrap();
    for (command, code) in [
        ("exit 9", 9),
        (
            if cfg!(windows) {
                "Write-Output 'hello'"
            } else {
                "printf hello"
            },
            0,
        ),
    ] {
        let mut p = spawn_shell(&shell, command, false, false, dir.path(), dir.path()).unwrap();
        assert_eq!(p.child.wait().await.unwrap(), Some(code));
        p.tree.terminate();
    }
}
#[tokio::test]
async fn terminal_resize_input_and_completion_event() {
    let dir = tempfile::tempdir().unwrap();
    let command = if cfg!(windows) {
        "$line = [Console]::ReadLine(); Write-Output ('received=' + $line)"
    } else {
        "read line; printf 'received=%s\\n' \"$line\""
    };
    let mut p = spawn_shell(
        &Shell::detect().await.unwrap(),
        command,
        false,
        true,
        dir.path(),
        dir.path(),
    )
    .unwrap();
    if let Input::Terminal(ref terminal) = p.input {
        terminal.resize(40, 120).unwrap();
        terminal.write(b"hello\n".to_vec(), false).await.unwrap();
    }
    let mut reader = p.readers.remove(0);
    let output = tokio::spawn(async move {
        let mut s = String::new();
        reader.read_to_string(&mut s).await.unwrap();
        s
    });
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(10), p.child.wait())
            .await
            .unwrap()
            .unwrap(),
        Some(0)
    );
    p.tree.terminate();
    drop(p.input);
    assert!(output.await.unwrap().contains("received=hello"));
}
#[tokio::test]
async fn cancelling_parent_stops_its_descendant() {
    let dir = tempfile::tempdir().unwrap();
    let mut p = spawn_pipe(helper("tree", dir.path())).unwrap();
    let mut output = String::new();
    let mut byte = [0];
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            p.readers[0].read_exact(&mut byte).await.unwrap();
            output.push(byte[0] as char);
            if output.contains("descendant=") && byte[0] == b'\n' {
                break;
            }
        }
    })
    .await
    .unwrap();
    let pid: u32 = output
        .lines()
        .find_map(|l| l.strip_prefix("descendant="))
        .unwrap()
        .parse()
        .unwrap();
    p.tree.terminate();
    let _ = p.child.kill();
    let _ = p.child.wait().await;
    let mut remainder = String::new();
    tokio::time::timeout(
        Duration::from_secs(5),
        p.readers[0].read_to_string(&mut remainder),
    )
    .await
    .expect("Descendant still holds pipe open")
    .unwrap();
    #[cfg(target_os = "linux")]
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let stat = std::fs::read_to_string(format!("/proc/{pid}/stat"));
            if stat.is_err()
                || stat.as_ref().is_ok_and(|s| {
                    s.rsplit_once(") ").unwrap().1.split_whitespace().next() == Some("Z")
                })
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("Descendant survived cancellation");
    #[cfg(not(target_os = "linux"))]
    let _ = pid;
}
#[test]
fn shell_builders_keep_script_as_one_argument_and_environment_is_filtered() {
    for kind in ["bash", "powershell"] {
        let s = Shell {
            program: kind.into(),
            kind,
        };
        let script = "echo 'a b'; echo c";
        assert_eq!(s.args(script).last(), Some(&script));
        assert!(s.script(script, true).contains(script));
    }
    let env = local_environment(std::path::Path::new("test-home"));
    for key in [
        "OPENAI_API_KEY",
        "BASH_ENV",
        "LD_PRELOAD",
        "PYTHONPATH",
        "HTTP_PROXY",
    ] {
        assert!(!env.contains_key(key));
    }
}

#[tokio::test]
async fn cancelled_terminal_wait_can_be_resumed_and_reaped() {
    let dir = tempfile::tempdir().unwrap();
    let command = if cfg!(windows) {
        "Start-Sleep -Seconds 60"
    } else {
        "sleep 60"
    };
    let mut p = spawn_shell(
        &Shell::detect().await.unwrap(),
        command,
        false,
        true,
        dir.path(),
        dir.path(),
    )
    .unwrap();
    assert!(
        tokio::time::timeout(Duration::from_millis(50), p.child.wait())
            .await
            .is_err()
    );
    p.tree.terminate();
    let _ = p.child.kill();
    tokio::time::timeout(Duration::from_secs(5), p.child.wait())
        .await
        .unwrap()
        .unwrap();
    // Repeating completion reads must not poll an already-consumed oneshot.
    p.child.wait().await.unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn terminal_signal_exit_keeps_the_native_missing_exit_code() {
    let dir = tempfile::tempdir().unwrap();
    let mut p = spawn_shell(
        &Shell::detect().await.unwrap(),
        "kill -TERM $$",
        false,
        true,
        dir.path(),
        dir.path(),
    )
    .unwrap();
    assert_eq!(p.child.wait().await.unwrap(), None);
    assert_eq!(p.child.wait().await.unwrap(), None);
    p.tree.terminate();
}

#[cfg(all(unix, not(any(target_os = "android", target_os = "ios"))))]
#[tokio::test]
async fn terminate_signal_requests_a_clean_exit_instead_of_killing_the_child() {
    use tokio::io::{AsyncBufReadExt, BufReader};
    let directory = tempfile::tempdir().unwrap();
    let mut command = helper("shutdown", directory.path());
    command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    let (mut child, _tree) = fenix_process::spawn_program(command).unwrap();
    let pid = child.id().unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    tokio::time::timeout(Duration::from_secs(5), async {
        while let Some(line) = lines.next_line().await.unwrap() {
            if line == "shutdown_ready" {
                return;
            }
        }
        panic!("Shutdown fixture ended before readiness");
    })
    .await
    .unwrap();
    nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(pid as i32),
        nix::sys::signal::Signal::SIGTERM,
    )
    .unwrap();
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(5), child.wait())
            .await
            .unwrap()
            .unwrap()
            .code(),
        Some(0)
    );
}
