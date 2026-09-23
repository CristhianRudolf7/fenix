use fenix_agent::{Runtime, config::Settings};
use fenix_storage::{PlanNodeSpec, TaskBinding};
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use wiremock::{
    Mock, MockServer, Request, ResponseTemplate,
    matchers::{method, path},
};

fn call(id: &str, name: &str, args: Value) -> Value {
    json!({"id":"fixture","status":"completed","output":[{"type":"function_call","call_id":id,"name":name,"arguments":args.to_string()}],"usage":{"input_tokens":10,"output_tokens":10,"total_tokens":20}})
}
fn answer() -> Value {
    json!({"id":"fixture","status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":"The chat remains available"}]}],"usage":{"input_tokens":10,"output_tokens":10,"total_tokens":20}})
}
fn output<'a>(body: &'a Value, id: &str) -> Option<&'a str> {
    body["input"]
        .as_array()?
        .iter()
        .rev()
        .find(|i| i["type"] == "function_call_output" && i["call_id"] == id)?["output"]
        .as_str()
}
async fn fixture() -> (tempfile::TempDir, Arc<Runtime>, MockServer) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("report.txt"), "Monthly report, 42 rows").unwrap();
    let runtime = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(answer()))
        .mount(&server)
        .await;
    let mut settings = Settings::initial(dir.path().into());
    settings.base_url = Some(server.uri());
    runtime
        .configure(settings, Some("fixture-api-key".into()))
        .await
        .unwrap();
    server.reset().await;
    (dir, runtime, server)
}
async fn wait(runtime: &Runtime, id: &str, state: &str) {
    tokio::time::timeout(Duration::from_secs(15), async {
        while runtime.store.task(id).unwrap().status != state {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap_or_else(|_| {
        panic!(
            "Expected {state}, got {:?}; events: {:?}",
            runtime.store.task(id),
            runtime.store.events(id, 0, 30)
        )
    });
}
async fn stop(runtime: &Runtime, handles: Vec<tokio::task::JoinHandle<()>>) {
    runtime.shutdown().await.unwrap();
    for handle in handles {
        tokio::time::timeout(Duration::from_secs(5), handle)
            .await
            .unwrap()
            .unwrap();
    }
}

#[tokio::test]
async fn auditor_asks_without_blocking_chat_checks_real_files_and_rejects_stale_observations() {
    let (dir, runtime, server) = fixture().await;
    let parent = runtime
        .store
        .enqueue_bound(
            "Prepare the monthly report",
            "web",
            None,
            &TaskBinding::default(),
        )
        .unwrap();
    runtime.store.claim_next().unwrap();
    runtime
        .store
        .set_task_destination(&parent.id, "telegram", Some("audit-fixture:42"))
        .unwrap();
    let plan=runtime.plan_manage(&parent.id,&json!({"action":"create","objective":"Prepare report","audit_required":true,"nodes":[{"id":"report","objective":"Produce report.txt","expected_output":"A report with 42 rows"}]})).await.unwrap()["plan"]["id"].as_str().unwrap().to_owned();
    runtime
        .store
        .set_task_status(&parent.id, "completed", None)
        .unwrap();
    let store = runtime.store.clone();
    let plan_id = plan.clone();
    let stale_rejected = Arc::new(AtomicBool::new(false));
    let saw_stale = stale_rejected.clone();
    Mock::given(method("POST")).and(path("/responses")).respond_with(move |request:&Request| {
        let body:Value=serde_json::from_slice(&request.body).unwrap();
        let tools=body["tools"].as_array().unwrap();
        let auditor=tools.iter().any(|t|t["name"]=="completion_audit_read");
        let result=if auditor {
            assert!(tools.iter().all(|t|!matches!(t["name"].as_str(),Some("command_run"|"file_write"|"subagent_spawn"|"plan_manage"|"connector_call"))));
            if output(&body,"forbidden").is_none() {
                call("forbidden","file_write",json!({"path":"must-not-exist.txt","content":"bad"}))
            } else if output(&body,"read").is_none() {
                assert!(output(&body,"forbidden").unwrap().contains("reviewers"));
                call("read","file_read",json!({"path":"report.txt"}))
            } else if output(&body,"question").is_none() {
                call("question","ask_user",json!({"question":"¿El informe corresponde a septiembre?"}))
            } else if output(&body,"finish").is_some_and(|s|s.contains("Observed file changed")) && output(&body,"reread").is_none() {
                saw_stale.store(true,Ordering::SeqCst);
                call("reread","file_read",json!({"path":"report.txt"}))
            } else {
                let plan=store.plan(&plan_id).unwrap();
                let read_id=if output(&body,"reread").is_some(){"reread"}else{"read"};
                let checks=store.plan_evidence(&plan_id,plan.version).unwrap().iter().map(|e|json!({"entry_id":e.input.id,"state":"verified","detail":"Compared report against current owner request and answer","call_ids":[read_id]})).collect::<Vec<_>>();
                call(if read_id=="reread"{"finish-current"}else{"finish"},"subagent_finish",json!({"status":"completed","summary":"Report checked","completion_audit":{"verdict":"verified","summary":"Actual report inspected","checks":checks}}))
            }
        } else if tools.iter().any(|t|t["name"]=="subagent_finish") {
            call("executor-finish","subagent_finish",json!({"status":"completed","summary":"Report ready","artifacts":"report.txt"}))
        } else {answer()};
        ResponseTemplate::new(200).set_body_json(result)
    }).mount(&server).await;
    let handles = runtime.start();
    let auditor = tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            if let Some(id) = runtime
                .store
                .completion_audits(&plan)
                .unwrap()
                .first()
                .and_then(|a| a.worker_id.clone())
            {
                break id;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    wait(&runtime, &auditor, "waiting").await;
    assert_eq!(runtime.store.plan(&plan).unwrap().status, "running");
    let quick = runtime.submit("Hello", "web").await.unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    wait(&runtime, &quick, "completed").await;
    std::fs::write(dir.path().join("report.txt"), "September report, 42 rows").unwrap();
    runtime
        .submit_referenced("Sí, septiembre", "web", Some(&auditor))
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(15), async {
        while runtime.store.plan(&plan).unwrap().status != "completed" {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    assert!(stale_rejected.load(Ordering::SeqCst));
    assert!(!dir.path().join("must-not-exist.txt").exists());
    let audits = runtime.store.completion_audits(&plan).unwrap();
    assert_eq!(
        audits.len(),
        1,
        "single-stage tasks need only one final audit"
    );
    assert_eq!(audits[0].status, "passed");
    assert_eq!(
        runtime.store.task_destination(&auditor).unwrap(),
        Some(("telegram".into(), Some("audit-fixture:42".into())))
    );
    stop(&runtime, handles).await;
}

#[tokio::test]
async fn ledger_doubts_are_persisted_before_asking_and_answers_do_not_self_verify() {
    let (_dir, runtime, server) = fixture().await;
    let owner = runtime
        .store
        .enqueue_bound("Produce a report", "web", None, &TaskBinding::default())
        .unwrap();
    runtime.store.claim_next().unwrap();
    let plan = runtime
        .store
        .create_plan(
            &owner.id,
            "Produce report",
            "",
            vec![PlanNodeSpec {
                profile: "general".into(),
                id: "report".into(),
                objective: "Produce report".into(),
                expected_output: "A report".into(),
                skills: vec![],
                depends_on: vec![],
            }],
        )
        .unwrap();
    runtime
        .store
        .set_task_status(&owner.id, "completed", None)
        .unwrap();
    let id = plan.id.clone();
    Mock::given(method("POST")).and(path("/responses")).respond_with(move |request:&Request| {
        let body:Value=serde_json::from_slice(&request.body).unwrap();
        let result=if !body["tools"].as_array().unwrap().iter().any(|t|t["name"]=="subagent_finish") {answer()} else if output(&body,"record").and_then(|r|serde_json::from_str::<Value>(r).ok()).is_some_and(|r|r.get("answer").is_some()) {call("done","subagent_finish",json!({"status":"completed","summary":"Format recorded"}))} else {
            call("record","plan_evidence",json!({"action":"record","plan_id":id,"expected_version":1,"entry":{"id":"note:format","kind":"requirement","state":"doubtful","statement":"Output format needs confirmation","source":"Owner did not specify a format","question":"¿Prefieres CSV o PDF?"}}))
        };
        ResponseTemplate::new(200).set_body_json(result)
    }).mount(&server).await;
    let handles = runtime.start();
    let task = tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            if let Some(id) = runtime.store.plan(&plan.id).unwrap().nodes[0]
                .worker_id
                .clone()
            {
                break id;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    wait(&runtime, &task, "waiting").await;
    let entry = runtime
        .store
        .plan_evidence(&plan.id, 1)
        .unwrap()
        .into_iter()
        .find(|e| e.input.id == "note:format")
        .unwrap();
    assert_eq!(entry.input.state, fenix_storage::EvidenceState::Doubtful);
    runtime
        .submit_referenced("PDF", "web", Some(&task))
        .await
        .unwrap();
    wait(&runtime, &task, "completed").await;
    assert_eq!(
        runtime
            .store
            .plan_evidence(&plan.id, 1)
            .unwrap()
            .into_iter()
            .find(|e| e.input.id == "note:format")
            .unwrap()
            .input
            .state,
        fenix_storage::EvidenceState::Doubtful,
        "An answer must not self-certify execution"
    );
    assert!(
        runtime
            .store
            .saved_questions(Some(&task))
            .unwrap()
            .iter()
            .any(|q| q["status"] == "answered")
    );
    stop(&runtime, handles).await;
}
