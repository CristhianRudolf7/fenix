#[path = "support/completion.rs"]
mod completion;
use fenix_agent::{Runtime, config::Settings};
use fenix_storage::TaskBinding;
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

fn answer(text: &str) -> Value {
    json!({"id":"resp-test","status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":text}]}],"usage":{"input_tokens":12,"output_tokens":8,"total_tokens":20}})
}

fn call(id: &str, name: &str, args: Value) -> Value {
    json!({"id":"resp-tool","status":"completed","output":[{"type":"function_call","call_id":id,"name":name,"arguments":args.to_string()}],"usage":{"input_tokens":12,"output_tokens":8,"total_tokens":20}})
}

fn finish(summary: &str) -> Value {
    call(
        "finish",
        "subagent_finish",
        json!({"status":"completed","summary":summary,"artifacts":"","evidence":"The fixture supplied a deterministic result.","blockers":"","review":{"verdict":"pass","findings":[],"specialists":[]}}),
    )
}

fn child_request(body: &Value) -> bool {
    body["input"].as_array().is_some_and(|input| {
        input.iter().any(|message| {
            message["role"] == "system" && message.to_string().contains("Delegated subagent task")
        })
    })
}

fn has_result(body: &Value, call_id: &str) -> bool {
    body["input"].as_array().is_some_and(|input| {
        input.iter().any(|item| {
            item["type"] == "function_call_output"
                && !item["output"]
                    .as_str()
                    .is_some_and(|s| s.contains("tool_schema_loaded"))
                && item["call_id"] == call_id
        })
    })
}

async fn fixture() -> (tempfile::TempDir, Arc<Runtime>, MockServer) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("audit-fixture.txt"),
        "A deterministic report fixture",
    )
    .unwrap();
    let runtime = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(answer("OK")))
        .mount(&server)
        .await;
    let mut settings = Settings::initial(dir.path().to_owned());
    settings.base_url = Some(server.uri());
    runtime
        .configure(settings, Some("fake-subagent-test-key".into()))
        .await
        .unwrap();
    server.reset().await;
    let store = runtime.store.clone();
    Mock::given(method("POST"))
        .and(path("/responses"))
        .and(|request: &Request| {
            serde_json::from_slice::<Value>(&request.body)
                .ok()
                .is_some_and(|b| {
                    b["tools"]
                        .as_array()
                        .is_some_and(|ts| ts.iter().any(|t| t["name"] == "completion_audit_read"))
                })
        })
        .respond_with(move |request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            ResponseTemplate::new(200)
                .set_body_json(completion::response(&body, &store).expect("Audit fixture"))
        })
        .with_priority(1)
        .mount(&server)
        .await;
    (dir, runtime, server)
}

async fn wait_task(runtime: &Runtime, id: &str, expected: &str) {
    tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            if runtime.store.task(id).unwrap().status == expected {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap_or_else(|_| {
        panic!(
            "Expected task {id} to be {expected}, got {}",
            runtime.store.task(id).unwrap().status
        )
    });
}

async fn wait_flag(flag: &AtomicBool) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while !flag.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
}

async fn stop(runtime: &Runtime, workers: Vec<tokio::task::JoinHandle<()>>) {
    runtime.shutdown().await.unwrap();
    for worker in workers {
        tokio::time::timeout(Duration::from_secs(5), worker)
            .await
            .unwrap()
            .unwrap();
    }
}

#[tokio::test]
async fn chat_and_replanning_remain_available_during_the_first_model_call() {
    let (dir, runtime, server) = fixture().await;
    let waiting = Arc::new(AtomicBool::new(false));
    let flag = waiting.clone();
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(move |request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            let system = body["input"][0]["content"].as_str().unwrap_or("");
            if system.contains("Owner updates, in arrival order") {
                assert!(system.contains("Use the revised objective"));
                return ResponseTemplate::new(200).set_body_json(answer("Revised draft ready"));
            }
            let last = body["input"]
                .as_array()
                .unwrap()
                .last()
                .unwrap()
                .to_string();
            if last.contains("draft-slow") {
                flag.store(true, Ordering::SeqCst);
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_secs(3))
                    .set_body_json(call(
                        "obsolete-write",
                        "file_write",
                        json!({"path":"obsolete.txt","content":"must not be written"}),
                    ))
            } else {
                ResponseTemplate::new(200).set_body_json(answer("Independent chat ready"))
            }
        })
        .mount(&server)
        .await;
    let workers = runtime.start();
    let first = runtime.submit("draft-slow", "web").await.unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    wait_flag(&waiting).await;
    let second = runtime.submit("A quick question", "web").await.unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    wait_task(&runtime, &second, "completed").await;
    assert_eq!(runtime.store.task(&first).unwrap().status, "running");
    let accepted = runtime
        .submit_referenced(
            "replanificar Use the revised objective",
            "web",
            Some(&first),
        )
        .await
        .unwrap();
    assert_eq!(accepted["task_id"], first);
    assert_eq!(accepted["accepted"], true);
    wait_task(&runtime, &first, "completed").await;
    assert!(!dir.path().join("obsolete.txt").exists());
    assert!(
        runtime
            .store
            .events(&first, 0, 100)
            .unwrap()
            .iter()
            .all(|e| e["data"]["tool"] != "file_write")
    );
    assert!(
        runtime
            .store
            .conversation(None, 100)
            .unwrap()
            .iter()
            .any(|m| m.text == "Revised draft ready")
    );
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn a_minor_question_is_persisted_and_does_not_hold_the_chat() {
    let (_dir, runtime, server) = fixture().await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(|request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            if has_result(&body, "minor-question") {
                return ResponseTemplate::new(200).set_body_json(answer("Format confirmed"));
            }
            let last = body["input"]
                .as_array()
                .unwrap()
                .last()
                .unwrap()
                .to_string();
            let response = if last.contains("format-question") {
                call(
                    "minor-question",
                    "ask_user",
                    json!({"question":"¿Prefieres fechas con guiones o barras?"}),
                )
            } else {
                answer("Chat stays available")
            };
            ResponseTemplate::new(200).set_body_json(response)
        })
        .mount(&server)
        .await;
    let workers = runtime.start();
    let task = runtime.submit("format-question", "web").await.unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    wait_task(&runtime, &task, "waiting").await;
    let saved = runtime.store.saved_questions(Some(&task)).unwrap();
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0]["status"], "pending");
    let next = runtime.submit("Another topic", "web").await.unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    wait_task(&runtime, &next, "completed").await;
    assert_eq!(runtime.store.task(&task).unwrap().status, "waiting");
    runtime
        .submit_referenced("Con guiones", "telegram", Some(&task))
        .await
        .unwrap();
    wait_task(&runtime, &task, "completed").await;
    assert_eq!(
        runtime.store.saved_questions(Some(&task)).unwrap()[0]["answer"],
        "Con guiones"
    );
    let request_count = server.received_requests().await.unwrap().len();
    runtime.deliver_events().unwrap();
    runtime.deliver_events().unwrap();
    assert_eq!(
        server.received_requests().await.unwrap().len(),
        request_count
    );
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn editing_mid_plan_keeps_completed_work_and_blocks_an_obsolete_write() {
    let (dir, runtime, server) = fixture().await;
    let waiting = Arc::new(AtomicBool::new(false));
    let flag = waiting.clone();
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(move |request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            assert!(child_request(&body));
            assert!(
                body["tools"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|t| t["name"] != "plan_manage" && t["name"] != "subagent_spawn")
            );
            let context = body["input"]
                .as_array()
                .unwrap()
                .last()
                .unwrap()
                .to_string();
            if context.contains("old-office-export") {
                flag.store(true, Ordering::SeqCst);
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_secs(3))
                    .set_body_json(call(
                        "old-effect",
                        "file_write",
                        json!({"path":"outdated.csv","content":"old"}),
                    ))
            } else {
                ResponseTemplate::new(200).set_body_json(finish("Current milestone completed"))
            }
        })
        .mount(&server)
        .await;
    let parent = runtime
        .store
        .enqueue_bound("Office report", "web", None, &TaskBinding::default())
        .unwrap();
    runtime.store.claim_next().unwrap();
    runtime
        .store
        .set_task_destination(&parent.id, "telegram", Some("original:7"))
        .unwrap();
    let specs = json!([
        {"id":"sources","objective":"Read office sources","expected_output":"Cited sources"},
        {"id":"export","objective":"old-office-export","expected_output":"A CSV export","depends_on":["sources"]}
    ]);
    let created = runtime
        .plan_manage(
            &parent.id,
            &json!({"action":"create","objective":"Prepare an office report","nodes":specs}),
        )
        .await
        .unwrap();
    let plan_id = created["plan"]["id"].as_str().unwrap().to_owned();
    runtime
        .store
        .set_task_status(&parent.id, "completed", None)
        .unwrap();
    let workers = runtime.start();
    wait_flag(&waiting).await;
    let old = runtime.store.plan(&plan_id).unwrap();
    assert_eq!(old.nodes[0].status, "completed");
    let old_worker = old.nodes[1].worker_id.clone().unwrap();
    let mut revised = specs;
    revised[1]["objective"] = json!("new-office-export");
    let result=runtime.plan_manage(&parent.id,&json!({"action":"revise","plan_id":plan_id,"expected_version":1,"instruction":"Use the new export","nodes":revised})).await.unwrap();
    assert_eq!(result["applied"], true);
    tokio::time::timeout(Duration::from_secs(8), async {
        while runtime.store.plan(&plan_id).unwrap().status != "completed" {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let current = runtime.store.plan(&plan_id).unwrap();
    assert_eq!(current.nodes[0].worker_id, old.nodes[0].worker_id);
    assert_ne!(current.nodes[1].worker_id, old.nodes[1].worker_id);
    assert_eq!(current.version, 2);
    assert!(!dir.path().join("outdated.csv").exists());
    assert!(runtime.store.check_plan_worker(&old_worker).is_err());
    assert!(
        runtime
            .store
            .tasks(100)
            .unwrap()
            .iter()
            .all(|t| t.origin != "subagent_report")
    );
    runtime.deliver_events().unwrap();
    runtime.deliver_events().unwrap();
    let history = runtime.store.conversation(None, 100).unwrap();
    assert_eq!(
        history
            .iter()
            .filter(|m| m.text.contains("Hitos completados: 2/2"))
            .count(),
        1
    );
    assert!(
        !runtime
            .store
            .deliveries_for("original:7")
            .unwrap()
            .is_empty()
    );
    assert!(
        runtime
            .store
            .deliveries_for("another:8")
            .unwrap()
            .is_empty()
    );
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn a_new_plan_edit_supersedes_an_older_question_without_reactivating_it() {
    let (_dir, runtime, server) = fixture().await;
    let parent = runtime
        .store
        .enqueue_bound("Prepare a report", "web", None, &TaskBinding::default())
        .unwrap();
    runtime.store.claim_next().unwrap();
    runtime
        .store
        .set_task_destination(&parent.id, "telegram", Some("pinned-bot:7"))
        .unwrap();
    let created=runtime.plan_manage(&parent.id,&json!({"action":"create","objective":"Report","nodes":[{"id":"report","objective":"Original report","expected_output":"A report"}]})).await.unwrap();
    let plan_id = created["plan"]["id"].as_str().unwrap().to_owned();
    runtime
        .store
        .set_task_status(&parent.id, "completed", None)
        .unwrap();
    let edited = runtime
        .steer_task(&plan_id, "first-change", "web")
        .await
        .unwrap();
    let first = edited["task_id"].as_str().unwrap().to_owned();
    assert_eq!(
        runtime.store.task_destination(&first).unwrap(),
        Some(("telegram".into(), Some("pinned-bot:7".into())))
    );
    let store = runtime.store.clone();
    let plan = plan_id.clone();
    Mock::given(method("POST")).and(path("/responses")).respond_with(move |request:&Request| {
        let body:Value=serde_json::from_slice(&request.body).unwrap();
        if child_request(&body) {return ResponseTemplate::new(200).set_body_json(finish("Updated report complete"));}
        let system=body["input"][0]["content"].as_str().unwrap_or("");
        let response=if system.contains("according to this owner instruction: first-change.") {
            call("old-question","ask_user",json!({"question":"Which heading should the first change use?"}))
        } else if has_result(&body,"revise-plan") {answer("The second change is applied")}
        else if has_result(&body,"read-plan") {
            let current=store.plan(&plan).unwrap();
            call("revise-plan","plan_manage",json!({"action":"revise","plan_id":plan,"expected_version":current.version,"instruction":"second-change","nodes":[{"id":"report","objective":"Second revised report","expected_output":"A report"}]}))
        } else {call("read-plan","plan_manage",json!({"action":"get","plan_id":plan}))};
        ResponseTemplate::new(200).set_body_json(response)
    }).mount(&server).await;
    let workers = runtime.start();
    wait_task(&runtime, &first, "waiting").await;
    let question = runtime.store.saved_questions(Some(&first)).unwrap()[0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let second = runtime
        .steer_task(&plan_id, "second-change", "web")
        .await
        .unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(
        runtime
            .submit(&format!("responder {question} old heading"), "web")
            .await
            .is_err()
    );
    wait_task(&runtime, &first, "cancelled").await;
    wait_task(&runtime, &second, "completed").await;
    tokio::time::timeout(Duration::from_secs(8), async {
        while runtime.store.plan(&plan_id).unwrap().status != "completed" {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let plan = runtime.store.plan(&plan_id).unwrap();
    assert_eq!(plan.version, 4);
    assert_eq!(plan.nodes[0].spec.objective, "Second revised report");
    assert_eq!(
        runtime.store.saved_questions(Some(&first)).unwrap()[0]["status"],
        "cancelled"
    );
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn a_natural_language_edit_can_find_and_steer_a_still_drafting_task() {
    let (dir, runtime, server) = fixture().await;
    let waiting = Arc::new(AtomicBool::new(false));
    let flag = waiting.clone();
    Mock::given(method("POST")).and(path("/responses")).respond_with(move |request:&Request| {
        let body:Value=serde_json::from_slice(&request.body).unwrap();
        let system=body["input"][0]["content"].as_str().unwrap_or("");
        if system.contains("Owner updates, in arrival order") {
            return ResponseTemplate::new(200).set_body_json(answer("Updated drafting objective"));
        }
        if has_result(&body,"steer-draft") {
            return ResponseTemplate::new(200).set_body_json(answer("Change requested"));
        }
        if body["input"].as_array().unwrap().iter().any(|m|m["role"]=="user" && m.to_string().contains("adjust-the-draft")) {
            let references=system.lines().find_map(|line|line.strip_prefix("Active task references in your authorized scope (metadata only, not additional assignments): ")).unwrap();
            let references:Value=serde_json::from_str(references).unwrap();
            let target=references.as_array().unwrap().iter().find(|item|item["objective"]=="draft-slow").unwrap();
            return ResponseTemplate::new(200).set_body_json(call("steer-draft","plan_manage",json!({"action":"steer","task_id":target["task_id"],"instruction":"Use the updated drafting objective"})));
        }
        flag.store(true,Ordering::SeqCst);
        ResponseTemplate::new(200).set_delay(Duration::from_secs(3)).set_body_json(call("obsolete","file_write",json!({"path":"obsolete-natural.txt","content":"outdated"})))
    }).mount(&server).await;
    let workers = runtime.start();
    let first = runtime.submit("draft-slow", "web").await.unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    wait_flag(&waiting).await;
    let edit = runtime.submit("adjust-the-draft", "web").await.unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    wait_task(&runtime, &first, "completed").await;
    wait_task(&runtime, &edit, "completed").await;
    assert!(!dir.path().join("obsolete-natural.txt").exists());
    assert!(
        runtime
            .store
            .conversation(None, 100)
            .unwrap()
            .iter()
            .any(|m| m.text == "Updated drafting objective")
    );
    stop(&runtime, workers).await;
}
