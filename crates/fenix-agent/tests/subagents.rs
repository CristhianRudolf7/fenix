use fenix_agent::{
    Runtime,
    config::{ActionPolicy, Settings},
};
use fenix_storage::Subagent;
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
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
        json!({"status":"completed","summary":summary,"artifacts":"","evidence":"The fixture supplied a deterministic result.","blockers":""}),
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
    (dir, runtime, server)
}

async fn delegate(runtime: &Runtime, objective: &str) -> Subagent {
    let parent = runtime
        .store
        .enqueue_bound(
            "Delegate the fixture task",
            "web",
            None,
            &fenix_storage::TaskBinding::default(),
        )
        .unwrap();
    assert_eq!(runtime.store.claim_next().unwrap().unwrap().id, parent.id);
    let result = runtime
        .spawn_subagent(
            &parent.id,
            &json!({"objective":objective,"expected_output":"An exact, structured result"}),
        )
        .await
        .unwrap();
    runtime
        .store
        .set_task_status(&parent.id, "completed", None)
        .unwrap();
    runtime
        .store
        .subagent(result["subagent_id"].as_str().unwrap())
        .unwrap()
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
async fn master_remains_available_and_child_context_loads_skills_on_demand() {
    let (_dir, runtime, server) = fixture().await;
    let skill = runtime.skills_dir.join("demand-only");
    std::fs::create_dir_all(&skill).unwrap();
    std::fs::write(skill.join("SKILL.md"), "---\nname: demand-only\ndescription: Load this skill to resolve the delegated fixture.\nversion: 1.0.0\n---\nONLY_FULL_SKILL_BODY\n").unwrap();
    let child_waiting = Arc::new(AtomicBool::new(false));
    let waiting = child_waiting.clone();
    Mock::given(method("POST")).and(path("/responses"))
        .respond_with(move |request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            if child_request(&body) {
                let encoded = body["input"].to_string();
                assert!(!encoded.contains("PRIVATE_MAIN_CONTEXT"));
                let tools = body["tools"].as_array().unwrap();
                for name in ["subagent_spawn", "subagent_status", "subagent_cancel", "history_search", "history_read"] {
                    assert!(!tools.iter().any(|tool| tool["name"] == name));
                }
                assert!(encoded.contains("demand-only"));
                if has_result(&body, "load-skill") {
                    assert!(encoded.contains("ONLY_FULL_SKILL_BODY"));
                    waiting.store(true, Ordering::SeqCst);
                    return ResponseTemplate::new(200).set_body_json(finish("Delegated work is ready")).set_delay(Duration::from_secs(2));
                }
                assert!(!encoded.contains("ONLY_FULL_SKILL_BODY"));
                return ResponseTemplate::new(200).set_body_json(call("load-skill", "skill_load", json!({"name":"demand-only"})));
            }
            let latest_user = body["input"].as_array().unwrap().iter().rev().find(|item| item["role"] == "user").map(Value::to_string).unwrap_or_default();
            if latest_user.contains("Master availability check") {
                ResponseTemplate::new(200).set_body_json(answer("Master still available"))
            } else if latest_user.contains("Internal subagent report") {
                ResponseTemplate::new(200).set_body_json(answer("Delivered the delegated result"))
            } else if has_result(&body, "spawn") {
                ResponseTemplate::new(200).set_body_json(answer("Delegated; I remain available"))
            } else if !has_result(&body, "load-delegation") {
                ResponseTemplate::new(200).set_body_json(call("load-delegation", "tool_search", json!({"names":["subagent_spawn"]})))
            } else {
                assert!(body["tools"].as_array().unwrap().iter().any(|tool| tool["name"] == "subagent_spawn"));
                ResponseTemplate::new(200).set_body_json(call("spawn", "subagent_spawn", json!({"objective":"Slow independent work","expected_output":"A brief completion report"})))
            }
        }).mount(&server).await;
    let workers = runtime.start();
    let first = runtime
        .submit(
            "Delegate slow work. PRIVATE_MAIN_CONTEXT stays in this conversation.",
            "web",
        )
        .await
        .unwrap();
    wait_task(&runtime, first["task_id"].as_str().unwrap(), "completed").await;
    wait_flag(&child_waiting).await;
    let child = runtime.store.subagents(None, 10).unwrap().remove(0);
    let second = runtime
        .submit("Master availability check", "telegram")
        .await
        .unwrap();
    wait_task(&runtime, second["task_id"].as_str().unwrap(), "completed").await;
    assert_eq!(runtime.store.subagent(&child.id).unwrap().status, "running");
    assert!(
        runtime
            .store
            .conversation(None, 100)
            .unwrap()
            .iter()
            .any(|message| message.text == "Master still available")
    );
    wait_task(&runtime, &child.id, "completed").await;
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if runtime
                .store
                .conversation(None, 100)
                .unwrap()
                .iter()
                .any(|message| message.text == "Delivered the delegated result")
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert!(
        !runtime
            .store
            .context(0)
            .unwrap()
            .iter()
            .any(|message| message.text.contains("ONLY_FULL_SKILL_BODY"))
    );
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn child_cannot_bypass_one_level_delegation_with_an_invented_tool_call() {
    let (_dir, runtime, server) = fixture().await;
    let job = delegate(&runtime, "Try a forbidden nested delegation").await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(move |request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            ResponseTemplate::new(200).set_body_json(if child_request(&body) {
                call(
                    "nested",
                    "subagent_spawn",
                    json!({"objective":"Grandchild","expected_output":"Forbidden"}),
                )
            } else {
                answer("Nested delegation was blocked")
            })
        })
        .mount(&server)
        .await;
    let workers = runtime.start();
    wait_task(&runtime, &job.id, "blocked").await;
    assert_eq!(runtime.store.subagents(None, 20).unwrap().len(), 1);
    assert_eq!(runtime.store.subagent(&job.id).unwrap().attempts, 1);
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn retryable_provider_failures_stop_after_three_total_attempts() {
    let (_dir, runtime, server) = fixture().await;
    let job = delegate(&runtime, "Bounded transient failure").await;
    let requests = Arc::new(AtomicUsize::new(0));
    let count = requests.clone();
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(move |request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            if child_request(&body) {
                count.fetch_add(1, Ordering::SeqCst);
                ResponseTemplate::new(429).insert_header("retry-after", "0")
            } else {
                ResponseTemplate::new(200).set_body_json(answer("The child exhausted its attempts"))
            }
        })
        .mount(&server)
        .await;
    let workers = runtime.start();
    wait_task(&runtime, &job.id, "failed").await;
    assert_eq!(requests.load(Ordering::SeqCst), 3);
    assert_eq!(runtime.store.subagent(&job.id).unwrap().attempts, 3);
    let events = runtime.store.events(&job.id, 0, 100).unwrap();
    assert_eq!(
        events
            .iter()
            .filter(|event| event["kind"] == "subagent_retry")
            .count(),
        2
    );
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn inference_retry_keeps_completed_tool_results_and_does_not_repeat_the_effect() {
    let (dir, runtime, server) = fixture().await;
    // Configure before enqueuing any work; the probe response is separate.
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(answer("OK")))
        .mount(&server)
        .await;
    let mut settings = runtime.settings().unwrap().unwrap();
    settings.file_write_policy = ActionPolicy::Allow;
    runtime.configure(settings, None).await.unwrap();
    server.reset().await;
    let job = delegate(
        &runtime,
        "Create one file despite a later inference failure",
    )
    .await;
    let requests = Arc::new(AtomicUsize::new(0));
    let count = requests.clone();
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(move |request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            if !child_request(&body) {
                return ResponseTemplate::new(200).set_body_json(answer("Created the file"));
            }
            if !body["tools"]
                .as_array()
                .unwrap()
                .iter()
                .any(|t| t["name"] == "file_write")
            {
                return ResponseTemplate::new(200).set_body_json(call(
                    "load-write",
                    "tool_search",
                    json!({"names":["file_write"]}),
                ));
            }
            match count.fetch_add(1, Ordering::SeqCst) {
                0 => ResponseTemplate::new(200).set_body_json(call(
                    "write-once",
                    "file_write",
                    json!({"path":"one-effect.txt","content":"created once"}),
                )),
                1 => {
                    assert!(has_result(&body, "write-once"));
                    ResponseTemplate::new(429).insert_header("retry-after", "0")
                }
                2 => {
                    assert!(has_result(&body, "write-once"));
                    assert!(body["input"].to_string().contains("created"));
                    ResponseTemplate::new(200).set_body_json(finish("The file was created once"))
                }
                _ => panic!("Unexpected extra child inference"),
            }
        })
        .mount(&server)
        .await;
    let workers = runtime.start();
    wait_task(&runtime, &job.id, "completed").await;
    assert_eq!(runtime.store.subagent(&job.id).unwrap().attempts, 2);
    assert_eq!(
        std::fs::read_to_string(dir.path().join("one-effect.txt")).unwrap(),
        "created once"
    );
    let events = runtime.store.events(&job.id, 0, 100).unwrap();
    assert_eq!(events.iter().filter(|event| event["kind"] == "tool_started" && event["data"]["tool"] == "file_write").count(), 1);
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn queued_cancellation_is_terminal_without_starting_an_attempt() {
    let (_dir, runtime, server) = fixture().await;
    let job = delegate(&runtime, "Cancel before the child worker starts").await;
    runtime.cancel_subagent(&job.id).await.unwrap();
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(move |request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            assert!(!child_request(&body));
            ResponseTemplate::new(200).set_body_json(answer("The queued task was cancelled"))
        })
        .mount(&server)
        .await;
    let workers = runtime.start();
    assert_eq!(runtime.store.subagent(&job.id).unwrap().status, "cancelled");
    assert_eq!(runtime.store.subagent(&job.id).unwrap().attempts, 0);
    assert!(runtime.cancel_subagent(&job.id).await.is_err());
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn shutdown_interrupts_child_inference_and_joins_background_workers() {
    let (_dir, runtime, server) = fixture().await;
    let job = delegate(&runtime, "A slow provider call to interrupt").await;
    let requested = Arc::new(AtomicBool::new(false));
    let flag = requested.clone();
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(move |request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            assert!(child_request(&body));
            flag.store(true, Ordering::SeqCst);
            ResponseTemplate::new(200)
                .set_body_json(finish("Too late"))
                .set_delay(Duration::from_secs(30))
        })
        .mount(&server)
        .await;
    let workers = runtime.start();
    wait_flag(&requested).await;
    stop(&runtime, workers).await;
    assert_eq!(
        runtime.store.subagent(&job.id).unwrap().status,
        "interrupted"
    );
    assert!(
        !runtime
            .store
            .tasks(20)
            .unwrap()
            .iter()
            .any(|task| task.origin == "subagent_report")
    );
}

#[tokio::test]
async fn background_concurrency_is_bounded_and_cancelling_a_running_job_releases_a_slot() {
    let (_dir, runtime, server) = fixture().await;
    let mut jobs = Vec::new();
    for index in 0..6 {
        jobs.push(delegate(&runtime, &format!("Concurrent fixture {index}")).await);
    }
    let requested = Arc::new(AtomicUsize::new(0));
    let count = requested.clone();
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(move |request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            if child_request(&body) {
                count.fetch_add(1, Ordering::SeqCst);
                ResponseTemplate::new(200)
                    .set_body_json(finish("Delayed result"))
                    .set_delay(Duration::from_secs(30))
            } else {
                ResponseTemplate::new(200).set_body_json(answer("The cancellation is recorded"))
            }
        })
        .mount(&server)
        .await;
    let workers = runtime.start();
    tokio::time::timeout(Duration::from_secs(5), async {
        while requested.load(Ordering::SeqCst) < 4 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(requested.load(Ordering::SeqCst), 4);
    let states = runtime.store.subagents(None, 20).unwrap();
    assert_eq!(
        states.iter().filter(|job| job.status == "running").count(),
        4
    );
    assert_eq!(
        states.iter().filter(|job| job.status == "queued").count(),
        2
    );
    runtime.cancel_subagent(&jobs[0].id).await.unwrap();
    wait_task(&runtime, &jobs[0].id, "cancelled").await;
    tokio::time::timeout(Duration::from_secs(5), async {
        while requested.load(Ordering::SeqCst) < 5 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(requested.load(Ordering::SeqCst), 5);
    assert_eq!(
        runtime
            .store
            .subagents(None, 20)
            .unwrap()
            .iter()
            .filter(|job| job.status == "running")
            .count(),
        4
    );
    stop(&runtime, workers).await;
    let outcomes = runtime.store.subagents(None, 20).unwrap();
    assert!(
        outcomes
            .iter()
            .all(|job| matches!(job.status.as_str(), "cancelled" | "interrupted"))
    );
    assert_eq!(runtime.store.subagent(&jobs[5].id).unwrap().attempts, 0);
}

#[tokio::test]
async fn simultaneous_child_approvals_are_routed_by_id_while_master_answers() {
    let (dir, runtime, server) = fixture().await;
    let first = delegate(&runtime, "Create approval-a.txt").await;
    let second = delegate(&runtime, "Create approval-b.txt").await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(move |request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            if !child_request(&body) {
                return ResponseTemplate::new(200)
                    .set_body_json(answer("Master is available during approvals"));
            }
            if has_result(&body, "approved-write") {
                return ResponseTemplate::new(200)
                    .set_body_json(finish("The approved file was created"));
            }
            let filename = if body["input"].to_string().contains("approval-a.txt") {
                "approval-a.txt"
            } else {
                "approval-b.txt"
            };
            ResponseTemplate::new(200).set_body_json(call(
                "approved-write",
                "file_write",
                json!({"path":filename,"content":"approved"}),
            ))
        })
        .mount(&server)
        .await;
    let workers = runtime.start();
    let pending = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let state = runtime.status().await.unwrap();
            if state["pending_requests"]
                .as_array()
                .is_some_and(|pending| pending.len() == 2)
            {
                break state["pending_requests"].as_array().unwrap().clone();
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert!(runtime.submit("approve", "web").await.is_err());
    let normal = runtime
        .submit("Please answer while approval is pending", "telegram")
        .await
        .unwrap();
    wait_task(&runtime, normal["task_id"].as_str().unwrap(), "completed").await;
    assert!(!dir.path().join("approval-a.txt").exists());
    assert!(!dir.path().join("approval-b.txt").exists());
    for request in &pending {
        runtime
            .submit(
                &format!("approve {}", request["id"].as_str().unwrap()),
                "web",
            )
            .await
            .unwrap();
    }
    wait_task(&runtime, &first.id, "completed").await;
    wait_task(&runtime, &second.id, "completed").await;
    assert_eq!(
        std::fs::read_to_string(dir.path().join("approval-a.txt")).unwrap(),
        "approved"
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("approval-b.txt")).unwrap(),
        "approved"
    );
    assert!(
        runtime.status().await.unwrap()["pending_requests"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn an_internal_report_cannot_reset_the_attempt_budget_by_spawning_a_replacement() {
    let (_dir, runtime, _server) = fixture().await;
    let child = delegate(&runtime, "Do not restart this failed task automatically").await;
    runtime
        .store
        .finish_subagent(
            &child.id,
            "failed",
            &json!({"status":"failed","summary":"Unable to complete"}),
        )
        .unwrap();
    let report = runtime
        .store
        .enqueue_subagent_report(&child.id)
        .unwrap()
        .unwrap();
    assert_eq!(runtime.store.claim_next().unwrap().unwrap().id, report.id);
    let result = runtime
        .spawn_subagent(
            &report.id,
            &json!({"objective":child.spec.objective,"expected_output":child.spec.expected_output}),
        )
        .await;
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("new user task is required")
    );
    assert_eq!(runtime.store.subagents(None, 20).unwrap().len(), 1);
}

#[tokio::test]
async fn supervisor_messages_interrupt_stale_actions_and_reuse_finished_context() {
    let (dir, runtime, server) = fixture().await;
    let job = delegate(&runtime, "Apply the supervisor's scoped corrections").await;
    let sender = runtime
        .store
        .enqueue_bound(
            "Correct the delegated work",
            "web",
            None,
            &fenix_storage::TaskBinding::default(),
        )
        .unwrap();
    runtime.store.claim_next().unwrap();
    let store = runtime.store.clone();
    let target = job.id.clone();
    let source = sender.id.clone();
    let first = Arc::new(AtomicBool::new(true));
    let initial = first.clone();
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(move |r: &Request| {
            let body: Value = serde_json::from_slice(&r.body).unwrap();
            if !child_request(&body) {
                return ResponseTemplate::new(200).set_body_json(answer("Report received"));
            }
            let text = body.to_string();
            let response = if initial.swap(false, Ordering::SeqCst) {
                store
                    .send_subagent_input(&source, &target, "Use the confirmed CSV format")
                    .unwrap();
                call(
                    "stale-write",
                    "file_write",
                    json!({"path":"must-not-exist.txt","content":"stale instructions"}),
                )
            } else if text.contains("Also report the delimiter") {
                assert!(text.contains("confirmed CSV format"));
                assert!(has_result(&body, "stale-write"));
                finish("CSV uses a comma delimiter")
            } else {
                assert!(text.contains("confirmed CSV format"));
                finish("CSV format preserved")
            };
            ResponseTemplate::new(200).set_body_json(response)
        })
        .mount(&server)
        .await;
    let workers = runtime.start();
    wait_task(&runtime, &job.id, "completed").await;
    assert!(!dir.path().join("must-not-exist.txt").exists());
    assert_eq!(runtime.store.subagent(&job.id).unwrap().attempts, 1);
    runtime
        .store
        .send_subagent_input(&sender.id, &job.id, "Also report the delimiter")
        .unwrap();
    wait_task(&runtime, &job.id, "completed").await;
    assert_eq!(
        runtime.store.subagent(&job.id).unwrap().result.unwrap()["summary"],
        "CSV uses a comma delimiter"
    );
    assert_eq!(runtime.store.subagent(&job.id).unwrap().attempts, 1);
    assert!(
        runtime
            .store
            .pending_subagent_inputs(&job.id)
            .unwrap()
            .is_empty()
    );
    stop(&runtime, workers).await;
}
