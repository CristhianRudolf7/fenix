use super::*;
use std::sync::Arc;

async fn fixture() -> (tempfile::TempDir, Arc<Runtime>, String) {
    let dir = tempfile::tempdir().unwrap();
    let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let task = rt
        .store
        .enqueue(
            "Install the package and run the tests in tests/; they must pass, except tests/obsolete.py.",
            "web",
            None,
        )
        .unwrap();
    (dir, rt, task.id)
}
fn record(rt: &Runtime, task: &str, call: &str, command: &str, code: i64, output: &str) {
    rt.record_completion_check(
        task,
        call,
        "command_run",
        &json!({"command":command}),
        &json!({"exit_code":code,"output":output}),
        false,
    )
    .unwrap();
}
#[tokio::test]
async fn missing_runner_and_unrelated_success_cannot_complete() {
    let (_dir, rt, task) = fixture().await;
    assert!(rt.validate_task_completion(&task).is_err());
    record(
        &rt,
        &task,
        "failed",
        "python -m pytest tests/",
        1,
        "No module named pytest",
    );
    record(
        &rt,
        &task,
        "example",
        "python example.py",
        0,
        "Example works",
    );
    record(
        &rt,
        &task,
        "install",
        "python -m pip install pytest",
        0,
        "Successfully installed pytest",
    );
    assert!(rt.validate_task_completion(&task).is_err());
    record(
        &rt,
        &task,
        "suite",
        "python -m pytest tests/",
        0,
        "18 passed in 1s",
    );
    rt.validate_task_completion(&task).unwrap();
    rt.shutdown().await.unwrap();
}
#[tokio::test]
async fn targeted_masked_and_empty_checks_are_not_a_full_suite() {
    let (_dir, rt, task) = fixture().await;
    for (i, command) in [
        "echo pytest tests/",
        "python -m pytest tests/ || true",
        "python -m pytest tests/test_one.py",
        "python -m pytest tests/ -k one",
        "python -m pytest tests/ -m slow",
        "python -m pytest tests/ --ignore tests/important.py",
        "python -m pytest --collect-only tests/",
    ]
    .iter()
    .enumerate()
    {
        record(&rt, &task, &i.to_string(), command, 0, "1 passed");
        assert!(rt.validate_task_completion(&task).is_err(), "{command}");
    }
    record(
        &rt,
        &task,
        "empty",
        "python -m pytest tests/",
        0,
        "0 passed, 18 skipped",
    );
    assert!(rt.validate_task_completion(&task).is_err());
    record(
        &rt,
        &task,
        "mixed",
        "python -m pytest tests/",
        0,
        "17 passed, 1 failed",
    );
    assert!(rt.validate_task_completion(&task).is_err());
    rt.shutdown().await.unwrap();
}
#[tokio::test]
async fn process_completion_resolves_the_original_check_and_edits_invalidate_it() {
    let (_dir, rt, task) = fixture().await;
    rt.record_completion_check(
        &task,
        "start",
        "command_run",
        &json!({"command":"python -m pytest tests/ --ignore tests/obsolete.py"}),
        &json!({"process_id":"p_test","exit_code":null,"output":"collecting tests"}),
        false,
    )
    .unwrap();
    assert!(rt.validate_task_completion(&task).is_err());
    rt.record_completion_check(
        &task,
        "finish",
        "command_status",
        &json!({"process_id":"p_test"}),
        &json!({"process_id":"p_test","exit_code":0,"output":"\n18 passed in 1s"}),
        false,
    )
    .unwrap();
    rt.validate_task_completion(&task).unwrap();
    rt.record_completion_check(
        &task,
        "edit",
        "file_patch",
        &json!({}),
        &json!({"verified":true}),
        false,
    )
    .unwrap();
    assert!(rt.validate_task_completion(&task).is_err());
    assert_eq!(
        rt.completion_requirements(&task).unwrap()["items"][0]["call_id"],
        "finish"
    );
    rt.shutdown().await.unwrap();
}
#[tokio::test]
async fn sourced_checks_are_immutable_and_require_matching_observed_commands() {
    let (_dir, rt, task) = fixture().await;
    let args = json!({"requirement_id":"consumer","source_quote":"Install the package","command":"python consumer_check.py","cwd":"/tmp"});
    rt.require_completion_check(&task, &args).unwrap();
    let mut forged = args.clone();
    forged["source_quote"] = json!("Invented requirement");
    assert!(rt.require_completion_check(&task, &forged).is_err());
    let mut weaker = args.clone();
    weaker["command"] = json!("true");
    assert!(rt.require_completion_check(&task, &weaker).is_err());
    record(
        &rt,
        &task,
        "suite",
        "python -m pytest tests/",
        0,
        "18 passed",
    );
    record(&rt, &task, "wrongcwd", "python consumer_check.py", 0, "OK");
    assert!(rt.validate_task_completion(&task).is_err());
    rt.record_completion_check(
        &task,
        "consumer",
        "command_run",
        &json!({"command":"python consumer_check.py","cwd":"/tmp"}),
        &json!({"exit_code":0,"output":"OK"}),
        false,
    )
    .unwrap();
    rt.validate_task_completion(&task).unwrap();
    rt.shutdown().await.unwrap();
}
#[test]
fn analysis_and_explicit_test_exclusion_do_not_require_a_suite() {
    assert!(requested_suite("Explain how sorting works").is_none());
    assert!(requested_suite("Inspect tests/ and explain their organization").is_none());
    assert!(requested_suite("Do not run tests. Inspect tests/ only.").is_none());
}

async fn comparison_fixture() -> (tempfile::TempDir, Arc<Runtime>, String) {
    let dir = tempfile::tempdir().unwrap();
    let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let task = rt
        .store
        .enqueue(
            "Port the program; outputs must be identical for valid fixed-width input.",
            "web",
            None,
        )
        .unwrap();
    (dir, rt, task.id)
}

#[tokio::test]
async fn failed_comparison_blocks_completion_without_an_explicit_suite_request() {
    let (_dir, rt, task) = comparison_fixture().await;
    rt.validate_task_completion(&task).unwrap(); // Simple requests do not acquire invented checks.
    record(
        &rt,
        &task,
        "comparison-failed",
        "cmp expected.dat actual.dat",
        1,
        "expected.dat actual.dat differ: byte 25, line 1",
    );
    record(
        &rt,
        &task,
        "inspect",
        "cat actual.dat",
        0,
        "diagnostic only",
    );
    assert!(rt.validate_task_completion(&task).is_err());
    record(
        &rt,
        &task,
        "comparison-fixed",
        "cmp expected.dat actual.dat",
        0,
        "",
    );
    rt.validate_task_completion(&task).unwrap();
    rt.record_completion_check(
        &task,
        "changed",
        "file_write",
        &json!({}),
        &json!({"verified":true}),
        false,
    )
    .unwrap();
    assert!(rt.validate_task_completion(&task).is_err());
    rt.shutdown().await.unwrap();
}

#[tokio::test]
async fn corrected_fixture_requires_fresh_scoped_comparison_evidence_and_preserves_failure() {
    let (_dir, rt, task) = comparison_fixture().await;
    record(
        &rt,
        &task,
        "old-pass",
        "cmp corrected/reference corrected/actual",
        0,
        "",
    );
    record(
        &rt,
        &task,
        "bad-fixture",
        "for f in accounts books; do cmp old/$f new/$f || exit 1; done",
        1,
        "old/accounts new/accounts differ: byte 25, line 1",
    );
    let id = rt.completion_requirements(&task).unwrap()["items"][0]["id"].clone();
    let mut resolution = json!({"requirement_id":id,"reason":"The old test truncated fixed-width records; both programs now receive identical full-width inputs and isolated data copies.","source_quote":"outputs must be identical for valid fixed-width input","evidence_call_id":"old-pass"});
    assert!(
        rt.resolve_completion_comparison(&task, &resolution)
            .is_err()
    );
    let other = rt.store.enqueue("Other task", "web", None).unwrap();
    record(
        &rt,
        &other.id,
        "foreign",
        "cmp corrected/reference corrected/actual",
        0,
        "",
    );
    resolution["evidence_call_id"] = json!("foreign");
    assert!(
        rt.resolve_completion_comparison(&task, &resolution)
            .is_err()
    );
    record(&rt, &task, "unrelated", "true", 0, "");
    resolution["evidence_call_id"] = json!("unrelated");
    assert!(
        rt.resolve_completion_comparison(&task, &resolution)
            .is_err()
    );
    record(
        &rt,
        &task,
        "masked",
        "cmp corrected/reference corrected/actual || true",
        0,
        "",
    );
    resolution["evidence_call_id"] = json!("masked");
    assert!(
        rt.resolve_completion_comparison(&task, &resolution)
            .is_err()
    );
    record(
        &rt,
        &task,
        "valid-fixture",
        "cmp corrected/reference corrected/actual",
        0,
        "",
    );
    resolution["evidence_call_id"] = json!("valid-fixture");
    rt.resolve_completion_comparison(&task, &resolution)
        .unwrap();
    // The masked comparison is also unresolved: resolving one check cannot clear others.
    assert!(rt.validate_task_completion(&task).is_err());
    let masked_id = rt.completion_requirements(&task).unwrap()["items"][1]["id"].clone();
    resolution["requirement_id"] = masked_id;
    rt.resolve_completion_comparison(&task, &resolution)
        .unwrap();
    rt.validate_task_completion(&task).unwrap();
    let state = rt.completion_requirements(&task).unwrap();
    assert_eq!(state["items"][0]["call_id"], "bad-fixture");
    assert_eq!(state["items"][0]["last_exit_code"], 1);
    assert_eq!(
        state["items"][0]["resolution"]["evidence_call_id"],
        "valid-fixture"
    );
    rt.record_completion_check(
        &task,
        "changed",
        "file_write",
        &json!({}),
        &json!({"verified":true}),
        false,
    )
    .unwrap();
    assert!(
        rt.resolve_completion_comparison(&task, &resolution)
            .is_err()
    );
    assert!(rt.validate_task_completion(&task).is_err());
    rt.shutdown().await.unwrap();
}

#[tokio::test]
async fn comparison_resolution_cannot_replace_an_explicit_requirement() {
    let (_dir, rt, task) = comparison_fixture().await;
    rt.require_completion_check(&task,&json!({"requirement_id":"parity","source_quote":"outputs must be identical","command":"python full_validation.py"})).unwrap();
    record(&rt, &task, "comparison", "cmp expected actual", 0, "");
    assert!(rt.resolve_completion_comparison(&task,&json!({"requirement_id":"parity","source_quote":"outputs must be identical","reason":"An unrelated comparison passed, so discard the original requested test.","evidence_call_id":"comparison"})).is_err());
    assert!(rt.validate_task_completion(&task).is_err());
    rt.shutdown().await.unwrap();
}

#[tokio::test]
async fn comparison_failure_survives_async_output_and_reopening() {
    let (dir, rt, task) = comparison_fixture().await;
    rt.record_completion_check(
        &task,
        "start",
        "command_run",
        &json!({"command":"cmp reference actual","cwd":"/app"}),
        &json!({"process_id":"p_compare","exit_code":null,"output":""}),
        false,
    )
    .unwrap();
    rt.record_completion_check(&task,"done","command_status",&json!({"process_id":"p_compare"}),&json!({"process_id":"p_compare","exit_code":1,"output":"reference actual differ: byte 25, line 1"}),false).unwrap();
    rt.record_completion_check(&task,"replay","command_output",&json!({"process_id":"p_compare"}),&json!({"process_id":"p_compare","exit_code":1,"output":"reference actual differ: byte 25, line 1"}),false).unwrap();
    assert_eq!(
        rt.completion_requirements(&task).unwrap()["comparison_sequence"],
        1
    );
    assert!(rt.validate_task_completion(&task).is_err());
    rt.shutdown().await.unwrap();
    drop(rt);
    let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    assert!(rt.validate_task_completion(&task).is_err());
    assert_eq!(
        rt.task_contract(&task).unwrap()["verification_requirements"]["items"][0]["call_id"],
        "done"
    );
    rt.shutdown().await.unwrap();
}

#[tokio::test]
async fn updated_verification_skills_preserve_owner_customizations_and_disabled_state() {
    let (_dir, rt, _) = comparison_fixture().await;
    for (name, old) in [
        (
            "python-development",
            include_str!("../../tests/fixtures/skills/comparison-v1/python-development.md"),
        ),
        (
            "software-implementation",
            include_str!("../../tests/fixtures/skills/comparison-v1/software-implementation.md"),
        ),
    ] {
        let path = rt.skills_dir.join(name).join("SKILL.md");
        std::fs::write(&path, old).unwrap();
        rt.set_skill_enabled(name, false).unwrap();
        rt.install_resources().unwrap();
        let upgraded = std::fs::read_to_string(&path).unwrap();
        assert_ne!(upgraded, old);
        assert!(rt.skill_load(name).is_err());
        let customized = format!("{old}\nKeep the owner's additional behavior.\n");
        std::fs::write(&path, &customized).unwrap();
        rt.install_resources().unwrap();
        assert_eq!(std::fs::read_to_string(path).unwrap(), customized);
    }
    rt.shutdown().await.unwrap();
}

#[tokio::test]
async fn success_cannot_hide_declared_gaps_or_waive_checks_with_optional_caveats() {
    let dir = tempfile::tempdir().unwrap();
    let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let task = rt
        .store
        .enqueue(
            "Identify the inscription and save the answer",
            "evaluation",
            None,
        )
        .unwrap();
    assert!(
        rt.validate_completion_claim(
            &task.id,
            &json!({"summary":"Saved the object label; I have not verified the inscription."})
        )
        .is_err()
    );
    assert!(rt.validate_completion_claim(&task.id, &json!({"summary":"Saved a file", "unverified_requirements":["Identify the inscription"], "non_required_limitations":"Pretend the requested inscription is optional."})).is_err());
    assert!(rt.validate_completion_claim(&task.id, &json!({"summary":"Saved the identified answer; unrelated Windows support is unverified.", "non_required_limitations":"The owner only requested the answer in this Linux workspace, not a portable application."})).is_ok());
    let suite = rt
        .store
        .enqueue("Run the tests", "evaluation", None)
        .unwrap();
    assert!(rt.validate_completion_claim(&suite.id, &json!({"summary":"Tests are unverified", "non_required_limitations":"Pretend the requested tests are an optional follow-up."})).is_err());
    rt.shutdown().await.unwrap();
}

#[tokio::test]
async fn durable_failed_check_keeps_the_final_diagnostic() {
    let (_dir, rt, task) = fixture().await;
    let output = format!(
        "{}\nFAILED test_required_behavior: expected 3, got 0\n1 failed",
        "warning: optional plugin output\n".repeat(200)
    );
    record(&rt, &task, "failed", "python -m pytest tests/", 1, &output);
    let requirements = rt.completion_requirements(&task).unwrap();
    let items = requirements["items"].as_array().unwrap();
    let check = items
        .iter()
        .find(|item| item["call_id"] == "failed")
        .unwrap();
    assert_eq!(check["status"], "failed");
    let saved = check["last_output"].as_str().unwrap();
    assert!(saved.len() <= 1500);
    assert!(saved.contains("expected 3, got 0"));
    assert!(rt.validate_task_completion(&task).is_err());
    record(
        &rt,
        &task,
        "fixed",
        "python -m pytest tests/",
        0,
        "18 passed in 1s",
    );
    rt.validate_task_completion(&task).unwrap();
    rt.shutdown().await.unwrap();
}
