//! End-to-end checks use a fresh helper process, never restrict the test runner.
use std::{fs, os::unix::fs::PermissionsExt, process::Command};

#[test]
fn readonly_helper_blocks_mutations_and_network_or_fails_closed() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.txt");
    fs::write(&input, "original").unwrap();
    let run = |command: &str| {
        Command::new(env!("CARGO_BIN_EXE_fenix"))
            .args([
                "internal-read-only",
                command,
                dir.path().to_str().unwrap(),
                &serde_json::json!([input]).to_string(),
            ])
            .output()
            .unwrap()
    };
    let read = run("cat input.txt");
    if !read.status.success() {
        let error = String::from_utf8_lossy(&read.stderr);
        assert!(
            error.contains("Landlock")
                || error.contains("landlock")
                || error.contains("Operation not permitted"),
            "Unexpected read failure: {error}"
        );
        assert!(!run("touch created.txt").status.success());
        assert!(!dir.path().join("created.txt").exists());
        eprintln!("Kernel sandbox unavailable; fail-closed path checked: {error}");
        return;
    }
    assert_eq!(read.stdout, b"original");
    let before = fs::metadata(&input).unwrap().permissions().mode();
    for command in [
        "echo changed > input.txt",
        "touch created.txt",
        "chmod 777 input.txt",
        "rm input.txt",
        "ln input.txt duplicate",
        "truncate -s 0 input.txt",
        "bash -c 'echo changed > input.txt'",
        "cat /proc/self/environ",
        "python3 -c 'import socket; socket.socket()'",
    ] {
        let result = run(command);
        assert!(
            !result.status.success(),
            "Forbidden operation succeeded: {command}"
        );
    }
    assert_eq!(fs::read_to_string(&input).unwrap(), "original");
    assert_eq!(fs::metadata(&input).unwrap().permissions().mode(), before);
    assert!(!dir.path().join("created.txt").exists());
}
