use fenix_agent::{
    Runtime,
    config::{ActionPolicy, Settings},
    workflows::{Run, WorkflowInput},
};
use serde_json::{Value, json};
use std::{path::Path, sync::Arc};
use tokio_util::sync::CancellationToken;

async fn runtime(root: &Path) -> Arc<Runtime> {
    let workspace = root.join("workspace");
    std::fs::create_dir_all(&workspace).unwrap();
    let rt = Runtime::open(&root.join("state"), Some(&root.join("skills")))
        .await
        .unwrap();
    let mut settings = Settings::initial(workspace);
    settings.file_write_policy = ActionPolicy::Allow;
    settings.command_policy = ActionPolicy::Allow;
    rt.store.put("settings", "active", &settings).unwrap();
    rt
}
fn flow(nodes: Value) -> WorkflowInput {
    serde_json::from_value(json!({"name":"Fixture workflow","nodes":nodes})).unwrap()
}
fn step(id: &str, kind: &str, arguments: Value, dependencies: &[&str]) -> Value {
    json!({"id":id,"label":id,"kind":kind,"arguments":arguments,"depends_on":dependencies})
}

#[test]
fn graph_validation_rejects_cycles_duplicates_unknown_references_and_nested_jobs() {
    let a = step("a", "notify", json!({"text":"hello"}), &["b"]);
    let b = step("b", "notify", json!({"text":"hi"}), &["a"]);
    assert!(flow(json!([a, b])).validate().is_err());
    let a = step("a", "notify", json!({"text":"hello"}), &[]);
    assert!(flow(json!([a, a])).validate().is_err());
    assert!(
        flow(json!([step(
            "a",
            "notify",
            json!({"text":{"$ref":"missing/text"}}),
            &[]
        )]))
        .validate()
        .is_err()
    );
    assert!(flow(json!([{ "id":"nested","label":"Nested","kind":"tool","tool":"workflow_manage","arguments":{},"depends_on":[] }])).validate().is_err());
}

#[tokio::test]
async fn execution_pins_versions_resolves_outputs_and_skips_the_other_branch() {
    let directory = tempfile::tempdir().unwrap();
    let rt = runtime(directory.path()).await;
    std::fs::write(directory.path().join("workspace/input.txt"), "Evidence").unwrap();
    let nodes = json!([
        {"id":"read","label":"Read input","kind":"tool","tool":"file_read","arguments":{"path":"input.txt"},"depends_on":[]},
        step("check","condition",json!({"reference":"read","operator":"exists"}),&["read"]),
        {"id":"write","label":"Write report","kind":"tool","tool":"file_write","arguments":{"path":"report.txt","content":"Completed"},"depends_on":["check"],"when":{"reference":"check/matches","operator":"equals","value":true}},
        {"id":"unused","label":"Unused branch","kind":"tool","tool":"file_write","arguments":{"path":"unwanted.txt","content":"Wrong branch"},"depends_on":["check"],"when":{"reference":"check/matches","operator":"equals","value":false}}
    ]);
    let saved = rt
        .save_workflow(None, None, flow(nodes.clone()))
        .await
        .unwrap();
    let run = rt.enqueue_workflow(&saved.id, false, None).await.unwrap();
    assert!(
        rt.store.claim_next().unwrap().is_none(),
        "Conversational worker must not claim workflow parents"
    );
    let mut changed = flow(nodes);
    changed.name = "Updated flow".into();
    assert!(
        rt.save_workflow(Some(&saved.id), Some(0), changed.clone())
            .await
            .is_err()
    );
    rt.save_workflow(Some(&saved.id), Some(1), changed)
        .await
        .unwrap();
    rt.run_workflow(run.clone(), CancellationToken::new())
        .await
        .unwrap();
    let result = rt.workflow_run(&run.id).unwrap();
    assert_eq!(result.status, "completed", "{:?}", result.error);
    assert_eq!(result.workflow.version, 1);
    assert_eq!(result.nodes["unused"].status, "skipped");
    assert_eq!(
        std::fs::read_to_string(directory.path().join("workspace/report.txt")).unwrap(),
        "Completed"
    );
    assert!(!directory.path().join("workspace/unwanted.txt").exists());
}

#[tokio::test]
async fn preview_stops_before_effects_and_resume_preserves_finished_steps() {
    let directory = tempfile::tempdir().unwrap();
    let rt = runtime(directory.path()).await;
    std::fs::write(directory.path().join("workspace/input.txt"), "input").unwrap();
    let saved=rt.save_workflow(None,None,flow(json!([
        {"id":"read","label":"Read","kind":"tool","tool":"file_read","arguments":{"path":"input.txt"}},
        {"id":"write","label":"Write","kind":"tool","tool":"file_write","arguments":{"path":"report.txt","content":"done"},"depends_on":["read"]}
    ]))).await.unwrap();
    let run = rt.enqueue_workflow(&saved.id, true, None).await.unwrap();
    rt.run_workflow(run.clone(), CancellationToken::new())
        .await
        .unwrap();
    let paused = rt.workflow_run(&run.id).unwrap();
    assert_eq!(paused.status, "paused");
    assert_eq!(paused.nodes["read"].attempts, 1);
    assert!(!directory.path().join("workspace/report.txt").exists());
    let other = rt.enqueue_workflow(&saved.id, false, None).await.unwrap();
    assert!(
        rt.resume_workflow(&run.id).await.is_err(),
        "Resuming a preview must not overlap another execution"
    );
    rt.cancel_workflow(&other.id).await.unwrap();
    let next = rt.resume_workflow(&run.id).await.unwrap();
    rt.run_workflow(next, CancellationToken::new())
        .await
        .unwrap();
    let done = rt.workflow_run(&run.id).unwrap();
    assert_eq!(done.status, "completed", "{:?}", done.error);
    assert_eq!(done.nodes["read"].attempts, 1);
}

#[tokio::test]
async fn connectors_work_in_flows_without_an_llm_and_other_accounts_cannot_read_them() {
    let directory = tempfile::tempdir().unwrap();
    let a = runtime(&directory.path().join("a")).await;
    let b = runtime(&directory.path().join("b")).await;
    let path = directory.path().join("a/workspace/reports.sqlite");
    let db = rusqlite::Connection::open(path).unwrap();
    db.execute_batch("CREATE TABLE reports(name TEXT); INSERT INTO reports VALUES ('weekly');")
        .unwrap();
    drop(db);
    let c = a
        .connectors
        .save(
            serde_json::from_value(
                json!({"provider":"sqlite","name":"My data","settings":{"path":"reports.sqlite"}}),
            )
            .unwrap(),
        )
        .unwrap();
    let saved=a.save_workflow(None,None,flow(json!([step("data","connector",json!({"connection":c["id"],"operation":"query","arguments":{"sql":"SELECT name FROM reports"}}),&[]) ]))).await.unwrap();
    let run = a.enqueue_workflow(&saved.id, false, None).await.unwrap();
    assert!(a.enqueue_workflow(&saved.id, false, None).await.is_err());
    assert!(b.workflow(&saved.id).is_err());
    assert!(b.workflow_run(&run.id).is_err());
    a.run_workflow(run.clone(), CancellationToken::new())
        .await
        .unwrap();
    let result = a.workflow_run(&run.id).unwrap();
    assert_eq!(result.status, "completed", "{:?}", result.error);
    assert_eq!(result.nodes["data"].output["rows"][0]["name"], "weekly");
}

#[tokio::test]
async fn invalid_step_failure_is_durable_and_effects_are_not_retried() {
    let directory = tempfile::tempdir().unwrap();
    let rt = runtime(directory.path()).await;
    let saved=rt.save_workflow(None,None,flow(json!([{"id":"write","label":"Invalid write","kind":"tool","tool":"file_write","attempts":3,"arguments":{"path":"../private.txt","content":"not allowed"}}]))).await.unwrap();
    let run = rt.enqueue_workflow(&saved.id, false, None).await.unwrap();
    rt.run_workflow(run.clone(), CancellationToken::new())
        .await
        .unwrap();
    let failed: Run = rt.workflow_run(&run.id).unwrap();
    assert_eq!(failed.status, "failed");
    assert_eq!(failed.nodes["write"].attempts, 1);
    assert!(
        rt.enqueue_workflow(&saved.id, false, None).await.is_err(),
        "An uncertain write must block automatic duplicate execution"
    );
    let resumed = rt.resume_workflow(&run.id).await.unwrap();
    assert_eq!(resumed.nodes["write"].status, "needs_confirmation");
}

#[tokio::test]
async fn cron_dispatches_a_native_flow_without_a_model_and_does_not_overlap() {
    let directory = tempfile::tempdir().unwrap();
    let rt = runtime(directory.path()).await;
    let saved = rt
        .save_workflow(
            None,
            None,
            flow(json!([step(
                "notify",
                "notify",
                json!({"text":"Scheduled"}),
                &[]
            )])),
        )
        .await
        .unwrap();
    let schedule=rt.schedule_manage(json!({"action":"create","name":"Daily workflow","content":"Daily notice","mode":"workflow","kind":"cron","cron":"0 9 * * *","workflow_id":saved.id})).await.unwrap();
    let mut schedule: fenix_agent::schedules::Schedule = serde_json::from_value(schedule).unwrap();
    schedule.next_run = Some(chrono::Utc::now() - chrono::Duration::seconds(1));
    rt.store.put("schedules", &schedule.id, &schedule).unwrap();
    rt.tick_schedules().await.unwrap();
    assert_eq!(rt.workflow_runs(Some(&saved.id)).unwrap().len(), 1);
    rt.tick_schedules().await.unwrap();
    assert_eq!(rt.workflow_runs(Some(&saved.id)).unwrap().len(), 1);
    assert!(rt.store.claim_next().unwrap().is_none());
}

#[tokio::test]
async fn restart_keeps_completed_checkpoints_and_flags_an_uncertain_step() {
    let directory = tempfile::tempdir().unwrap();
    let rt = runtime(directory.path()).await;
    let saved = rt
        .save_workflow(
            None,
            None,
            flow(json!([
                step("first", "notify", json!({"text":"first"}), &[]),
                step("second", "notify", json!({"text":"second"}), &["first"])
            ])),
        )
        .await
        .unwrap();
    let mut run = rt.enqueue_workflow(&saved.id, false, None).await.unwrap();
    run.status = "running".into();
    run.nodes.get_mut("first").unwrap().status = "completed".into();
    run.nodes.get_mut("second").unwrap().status = "running".into();
    rt.store.put("workflow_runs", &run.id, &run).unwrap();
    drop(rt);
    let reopened = Runtime::open(
        &directory.path().join("state"),
        Some(&directory.path().join("skills")),
    )
    .await
    .unwrap();
    let recovered = reopened.workflow_run(&run.id).unwrap();
    assert_eq!(recovered.status, "interrupted");
    assert_eq!(recovered.nodes["first"].status, "completed");
    assert_eq!(recovered.nodes["second"].status, "interrupted");
    assert!(
        reopened
            .enqueue_workflow(&saved.id, false, None)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn a_question_waits_for_a_text_reply_without_blocking_the_conversation() {
    let directory = tempfile::tempdir().unwrap();
    let rt = runtime(directory.path()).await;
    let saved = rt
        .save_workflow(
            None,
            None,
            flow(json!([step(
                "question",
                "wait",
                json!({"question":"Which report should be used?"}),
                &[]
            )])),
        )
        .await
        .unwrap();
    let run = rt.enqueue_workflow(&saved.id, false, None).await.unwrap();
    let worker = rt.clone();
    let execution = run.clone();
    let handle = tokio::spawn(async move {
        worker
            .run_workflow(execution, CancellationToken::new())
            .await
    });
    let question = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let status = rt.status().await.unwrap();
            if let Some(id) = status["pending_requests"][0]["id"].as_str() {
                break id.to_owned();
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(rt.workflow_run(&run.id).unwrap().status, "waiting");
    rt.submit(&format!("responder {question} The weekly report"), "web")
        .await
        .unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), handle)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(
        rt.workflow_run(&run.id).unwrap().nodes["question"].output["answer"],
        "The weekly report"
    );
}

#[tokio::test]
async fn a_script_waits_for_its_exit_status() {
    let directory = tempfile::tempdir().unwrap();
    let rt = runtime(directory.path()).await;
    let saved=rt.save_workflow(None,None,flow(json!([step("script","script",json!({"command":"printf 'workflow fixture'","cwd":directory.path().join("workspace")}),&[]) ]))).await.unwrap();
    let run = rt.enqueue_workflow(&saved.id, false, None).await.unwrap();
    tokio::time::timeout(
        std::time::Duration::from_secs(15),
        rt.run_workflow(run.clone(), CancellationToken::new()),
    )
    .await
    .unwrap()
    .unwrap();
    let result = rt.workflow_run(&run.id).unwrap();
    assert_eq!(result.status, "completed", "{:?}", result.error);
    assert_eq!(result.nodes["script"].output["exit_code"], 0);
}
