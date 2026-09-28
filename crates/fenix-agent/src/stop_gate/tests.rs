use super::*;
use std::sync::Arc;
async fn fixture() -> (tempfile::TempDir, Arc<Runtime>, String) {
    let dir = tempfile::tempdir().unwrap();
    let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let task = rt
        .store
        .enqueue("Build and verify the requested package", "web", None)
        .unwrap();
    (dir, rt, task.id)
}
fn command(rt: &Runtime, task: &str, call: &str, cmd: &str, code: i64, text: &str) {
    rt.record_tool_outcome(
        task,
        call,
        "command_run",
        &json!({"command":cmd,"cwd":"/app"}),
        Ok(json!({"exit_code":code,"output":text})),
    )
    .unwrap();
}
fn stop(reason: &str, call: &str) -> Value {
    json!({"status":"failed","stop_reason":reason,"blocker_call_id":call,"blockers":"claimed blocker"})
}
#[tokio::test]
async fn successful_preparation_and_prose_cannot_justify_abandonment() {
    let (_d, rt, t) = fixture().await;
    command(&rt, &t, "clone", "git clone repository", 0, "cloned");
    assert!(
        rt.validate_task_stop(&t, &stop("external_dependency", "clone"))
            .is_err()
    );
    assert!(
        rt.validate_task_stop(&t, &stop("recovery_exhausted", "invented"))
            .is_err()
    );
    let pause = rt
        .validate_task_stop(&t, &stop("recovery_exhausted", "invented"))
        .unwrap()
        .unwrap();
    assert_eq!(pause["stop_reason"], "runtime_stagnation");
    assert!(
        rt.store
            .events(&t, 0, 100)
            .unwrap()
            .iter()
            .any(|e| e["kind"] == "execution_stalled")
    );
    rt.shutdown().await.unwrap();
}
#[tokio::test]
async fn missing_dependency_requires_repair_and_unchanged_retries_do_not_count() {
    let (_d, rt, t) = fixture().await;
    for i in 0..3 {
        command(
            &rt,
            &t,
            &format!("test-{i}"),
            "python -m pytest tests",
            1,
            "No module named pytest",
        );
    }
    let state = rt
        .store
        .get::<StopState>("task_stop_state", &t)
        .unwrap()
        .unwrap();
    assert_eq!(
        state
            .observations
            .iter()
            .map(|o| &o.attempt)
            .collect::<HashSet<_>>()
            .len(),
        1
    );
    assert!(rt.repeated_failure_pause(&t).unwrap().is_some());
    let pause = rt
        .validate_task_stop(&t, &stop("recovery_exhausted", "test-2"))
        .unwrap()
        .unwrap();
    assert_eq!(pause["stop_reason"], "runtime_stagnation"); // pause, never accept claimed exhaustion
    rt.shutdown().await.unwrap();
}
#[tokio::test]
async fn distinct_repairs_allow_honest_exhaustion_and_success_resolves_failure() {
    let (_d, rt, t) = fixture().await;
    for i in 0..3 {
        if i > 0 {
            rt.record_tool_outcome(
                &t,
                &format!("repair-{i}"),
                "file_patch",
                &json!({"patch":format!("different repair {i}")}),
                Ok(json!({"verified":true})),
            )
            .unwrap();
        }
        command(
            &rt,
            &t,
            &format!("check-{i}"),
            "python -m pytest tests",
            1,
            "1 failed, 17 passed",
        );
    }
    assert!(
        rt.validate_task_stop(&t, &stop("recovery_exhausted", "check-2"))
            .unwrap()
            .is_none()
    );
    command(&rt, &t, "fixed", "python -m pytest tests", 0, "18 passed");
    assert!(
        rt.validate_task_stop(&t, &stop("recovery_exhausted", "check-2"))
            .is_err()
    );
    rt.shutdown().await.unwrap();
}
#[tokio::test]
async fn observed_denial_and_external_failure_are_scoped_to_the_task() {
    let (_d, rt, t) = fixture().await;
    rt.record_tool_outcome(
        &t,
        "denial",
        "request_approval",
        &json!({}),
        Ok(json!({"approved":false})),
    )
    .unwrap();
    assert!(
        rt.validate_task_stop(&t, &stop("permission_denied", "denial"))
            .unwrap()
            .is_none()
    );
    command(
        &rt,
        &t,
        "offline",
        "curl https://required.example",
        6,
        "curl: (6) Could not resolve host: required.example",
    );
    assert!(
        rt.validate_task_stop(&t, &stop("external_dependency", "offline"))
            .unwrap()
            .is_none()
    );
    let other = rt.store.enqueue("Other project", "web", None).unwrap();
    assert!(
        rt.validate_task_stop(&other.id, &stop("external_dependency", "offline"))
            .is_err()
    );
    rt.shutdown().await.unwrap();
}
#[tokio::test]
async fn repeated_async_reads_are_not_recovery_attempts() {
    let (_d, rt, t) = fixture().await;
    rt.record_tool_outcome(
        &t,
        "start",
        "command_run",
        &json!({"command":"make","cwd":"/app"}),
        Ok(json!({"process_id":"p_1","exit_code":null,"output":"building"})),
    )
    .unwrap();
    for i in 0..4 {
        rt.record_tool_outcome(
            &t,
            &format!("poll-{i}"),
            "command_status",
            &json!({"process_id":"p_1"}),
            Ok(json!({"process_id":"p_1","exit_code":1,"output":"compiler error"})),
        )
        .unwrap();
    }
    let state = rt
        .store
        .get::<StopState>("task_stop_state", &t)
        .unwrap()
        .unwrap();
    assert_eq!(state.observations.len(), 1);
    assert!(
        rt.validate_task_stop(&t, &stop("recovery_exhausted", "poll-0"))
            .is_err()
    );
    rt.shutdown().await.unwrap();
}

#[tokio::test]
async fn successful_shell_repairs_break_an_unchanged_failure_streak() {
    let (_d, rt, t) = fixture().await;
    for i in 0..2 {
        command(&rt, &t, &format!("old-{i}"), "make", 1, "compiler error");
    }
    command(
        &rt,
        &t,
        "repair",
        "python -c 'path.write_text(fixed_source)'",
        0,
        "",
    );
    command(&rt, &t, "new", "make", 1, "compiler error");
    assert!(rt.repeated_failure_pause(&t).unwrap().is_none());
    // Repeating the same failed build cannot be presented as a new repair.
    let state = rt
        .store
        .get::<StopState>("task_stop_state", &t)
        .unwrap()
        .unwrap();
    assert_eq!(
        state
            .observations
            .iter()
            .map(|o| &o.attempt)
            .collect::<HashSet<_>>()
            .len(),
        2
    );
    rt.shutdown().await.unwrap();
}

#[tokio::test]
async fn cosmetic_patch_variations_are_the_same_stalled_edit() {
    let (_d, rt, t) = fixture().await;
    for i in 0..3 {
        rt.record_tool_outcome(&t,&format!("patch-{i}"),"file_patch",&json!({"patch":format!("*** Begin Patch\n*** Update File: /app/example.py\n@@\n-old\n+new # variation {i}\n*** End Patch")}),Err(anyhow::anyhow!("Hunk context must match exactly once; read the file and provide more context"))).unwrap();
    }
    let state = rt
        .store
        .get::<StopState>("task_stop_state", &t)
        .unwrap()
        .unwrap();
    assert_eq!(state.progress, 1);
    assert_eq!(
        state
            .observations
            .iter()
            .map(|o| &o.failure_key)
            .collect::<HashSet<_>>()
            .len(),
        1
    );
    assert!(rt.repeated_failure_pause(&t).unwrap().is_some());
    rt.record_tool_outcome(
        &t,
        "repair",
        "file_edit",
        &json!({"path":"/app/example.py","old":"specific original","new":"fixed"}),
        Ok(json!({"verified":true})),
    )
    .unwrap();
    assert!(rt.repeated_failure_pause(&t).unwrap().is_none());
    rt.shutdown().await.unwrap();
}

#[tokio::test]
async fn apt_warning_does_not_turn_a_missing_extractor_into_permission_denial() {
    let (_d, rt, t) = fixture().await;
    command(
        &rt,
        &t,
        "source",
        "apt-get source package",
        100,
        "sh: 1: dpkg-source: not found\nW: Download is performed unsandboxed as root; _apt (13: Permission denied)\nE: Unpack command failed.",
    );
    assert_eq!(
        rt.stop_context(&t).unwrap()["unresolved"][0]["category"],
        "missing_dependency"
    );
    assert!(
        rt.validate_task_stop(&t, &stop("permission_denied", "source"))
            .is_err()
    );
    // An actual runtime denial always wins over text that resembles a missing tool.
    rt.record_tool_outcome(
        &t,
        "denied",
        "command_run",
        &json!({"command":"install dependency"}),
        Ok(json!({"denied":true,"executed":false,"output":"sh: 1: dpkg-source: not found"})),
    )
    .unwrap();
    assert!(
        rt.validate_task_stop(&t, &stop("permission_denied", "denied"))
            .unwrap()
            .is_none()
    );
    assert!(!crate::execution_state::missing_dependency_diagnostic(
        "open: Permission denied"
    ));
    rt.shutdown().await.unwrap();
}

#[tokio::test]
async fn recovery_hint_requires_a_failed_repair_and_clears_after_success() {
    let (_d, rt, t) = fixture().await;
    for i in 0..2 {
        command(
            &rt,
            &t,
            &format!("unchanged-{i}"),
            "python check.py",
            1,
            "wrong value",
        );
    }
    assert!(rt.stop_context(&t).unwrap()["recovery_hint"].is_null());
    rt.record_tool_outcome(
        &t,
        "repair",
        "file_patch",
        &json!({"patch":"observed correction"}),
        Ok(json!({"verified":true})),
    )
    .unwrap();
    command(&rt, &t, "recheck", "python check.py", 1, "wrong value");
    assert!(
        rt.stop_context(&t).unwrap()["recovery_hint"]
            .as_str()
            .unwrap()
            .contains("tiny reproducer")
    );
    command(&rt, &t, "passed", "python check.py", 0, "correct value");
    assert!(rt.stop_context(&t).unwrap()["recovery_hint"].is_null());
    rt.record_tool_outcome(
        &t,
        "denied",
        "request_approval",
        &json!({}),
        Ok(json!({"denied":true})),
    )
    .unwrap();
    assert!(rt.stop_context(&t).unwrap()["recovery_hint"].is_null());
    rt.shutdown().await.unwrap();
}

#[tokio::test]
async fn owner_unavailable_cannot_override_an_explicit_admission_that_work_can_continue() {
    let (_d, rt, t) = fixture().await;
    rt.record_tool_outcome(
        &t,
        "owner",
        "ask_user",
        &json!({}),
        Err(anyhow::anyhow!(
            "noninteractive_input_unavailable: missing owner"
        )),
    )
    .unwrap();
    assert!(rt.validate_task_stop(&t, &json!({"status":"blocked","stop_reason":"owner_input_unavailable","blocker_call_id":"owner", "blockers":"The question was unnecessary; the interface is available."})).is_err());
    assert!(rt.validate_task_stop(&t, &json!({"status":"blocked","stop_reason":"owner_input_unavailable","blocker_call_id":"owner", "blockers":"Falta una elección real del propietario, no indicada en la tarea."})).unwrap().is_none());
    rt.shutdown().await.unwrap();
}

#[tokio::test]
async fn long_shared_warning_prefix_does_not_hide_new_failure_evidence() {
    let (_d, rt, task) = fixture().await;
    let prefix = "warning: repeated compiler diagnostic\n".repeat(200);
    for index in 0..3 {
        let diagnostic = format!("{prefix}fatal: unresolved symbol target_{index}");
        command(
            &rt,
            &task,
            &format!("changed-{index}"),
            "make",
            1,
            &diagnostic,
        );
        assert!(rt.repeated_failure_pause(&task).unwrap().is_none());
    }
    let context = rt.stop_context(&task).unwrap();
    assert!(
        context["unresolved"][0]["diagnostic"]
            .as_str()
            .unwrap()
            .ends_with("target_2")
    );
    assert!(
        context["unresolved"][0]["diagnostic"]
            .as_str()
            .unwrap()
            .len()
            <= 800
    );
    // Still catch genuinely unchanged failures, including a long output prefix.
    for index in 0..2 {
        command(
            &rt,
            &task,
            &format!("same-{index}"),
            "make",
            1,
            &format!("{prefix}fatal: unresolved symbol target_2"),
        );
    }
    let pause = rt.repeated_failure_pause(&task).unwrap().unwrap();
    assert_eq!(pause["stop_reason"], "runtime_stagnation");
    rt.shutdown().await.unwrap();
}

#[tokio::test]
async fn unchanged_native_edit_is_neither_repair_nor_stale_verification() {
    let (_d, rt, task) = fixture().await;
    let hash = "a".repeat(64);
    let same = json!({"verified":true,"before_sha256":hash,"sha256":hash});
    for index in 0..2 {
        command(
            &rt,
            &task,
            &format!("failure-{index}"),
            "make",
            1,
            "fatal error",
        );
    }
    rt.record_tool_outcome(
        &task,
        "noop",
        "file_edit",
        &json!({"path":"/app/source.c","expected":"old","replacement":"old"}),
        Ok(same.clone()),
    )
    .unwrap();
    command(&rt, &task, "failure-2", "make", 1, "fatal error");
    assert!(rt.repeated_failure_pause(&task).unwrap().is_some());
    command(&rt, &task, "fixed", "make", 0, "built");
    command(
        &rt,
        &task,
        "tests",
        "python -m pytest tests/",
        0,
        "18 passed",
    );
    rt.validate_task_completion(&task).unwrap();
    rt.record_tool_outcome(&task, "noop-again", "file_edit", &json!({}), Ok(same))
        .unwrap();
    rt.validate_task_completion(&task).unwrap();
    rt.record_tool_outcome(
        &task,
        "real-edit",
        "file_edit",
        &json!({}),
        Ok(json!({"verified":true,"before_sha256":hash,"sha256":"b".repeat(64)})),
    )
    .unwrap();
    assert!(rt.validate_task_completion(&task).is_err());
    rt.shutdown().await.unwrap();
}
