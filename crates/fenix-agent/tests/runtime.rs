use fenix_agent::{Runtime, config::Settings, schedules::Schedule, telegram::TelegramSettings};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
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
async fn fixture(script: Vec<Value>) -> (tempfile::TempDir, Arc<Runtime>, MockServer) {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let server = MockServer::start().await;
    let mut steps = VecDeque::from([answer("OK")]);
    steps.extend(script);
    let steps = Arc::new(Mutex::new(steps));
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(move |request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            assert_eq!(body["model"], "gpt-5.6-luna");
            assert_eq!(body["reasoning"]["effort"], "low");
            ResponseTemplate::new(200).set_body_json(
                steps
                    .lock()
                    .unwrap()
                    .pop_front()
                    .unwrap_or_else(|| answer("Handled")),
            )
        })
        .mount(&server)
        .await;
    let mut settings = Settings::initial(dir.path().to_owned());
    settings.base_url = Some(server.uri());
    runtime
        .configure(settings, Some("fake-private-test-key".into()))
        .await
        .unwrap();
    (dir, runtime, server)
}
async fn wait_for(runtime: &Runtime, id: &str, status: &str) {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if runtime.store.task(id).unwrap().status == status {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap_or_else(|_| {
        panic!(
            "Expected {status}, got {}",
            runtime.store.task(id).unwrap().status
        )
    });
}
async fn pending(runtime: &Runtime) -> Value {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let status = runtime.status().await.unwrap();
            if status["pending"].is_object() {
                break status["pending"].clone();
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap()
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
async fn incomplete_stream_discards_preview_without_retries_or_tool_execution() {
    let (_dir, runtime, server) = fixture(vec![]).await;
    server.reset().await;
    let body = format!(
        "data: {}\n\ndata: {}\n\n",
        json!({"type":"response.output_text.delta","delta":"Unfinished preview "}),
        json!({"type":"response.function_call_arguments.delta","delta":"{}"})
    );
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "text/event-stream"))
        .expect(1)
        .mount(&server)
        .await;
    let workers = runtime.start();
    let submitted = runtime
        .submit("Handle an interrupted stream", "web")
        .await
        .unwrap();
    let id = submitted["task_id"].as_str().unwrap();
    wait_for(&runtime, id, "failed").await;
    stop(&runtime, workers).await;
    let preview = runtime.subscribe_progress().borrow().clone();
    assert!(preview.preview.is_empty());
    assert!(preview.task_id.is_none());
    let history = runtime.store.context(0).unwrap();
    assert!(
        !history
            .iter()
            .any(|m| m.text.contains("Unfinished preview"))
    );
    server.verify().await;
}

#[tokio::test]
async fn mandatory_approval_unblocks_active_task_while_new_messages_stay_fifo() {
    let (dir, runtime, server) = fixture(vec![
        call(
            "write-1",
            "file_write",
            json!({"path":"approved.txt","content":"done"}),
        ),
        answer("File verified"),
        answer("Second request handled"),
    ])
    .await;
    let workers = runtime.start();
    let first = runtime.submit("Create approved.txt", "web").await.unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let approval = pending(&runtime).await;
    let second = runtime
        .submit("A new independent request", "telegram")
        .await
        .unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(runtime.store.task(&second).unwrap().status, "queued");
    assert!(!dir.path().join("approved.txt").exists());
    runtime.submit("quizá", "web").await.unwrap();
    assert!(!dir.path().join("approved.txt").exists());
    assert_eq!(
        runtime.status().await.unwrap()["pending"]["id"],
        approval["id"]
    );
    runtime
        .submit(
            &format!("aprobar {}", approval["id"].as_str().unwrap()),
            "web",
        )
        .await
        .unwrap();
    wait_for(&runtime, &first, "completed").await;
    wait_for(&runtime, &second, "completed").await;
    assert_eq!(
        std::fs::read_to_string(dir.path().join("approved.txt")).unwrap(),
        "done"
    );
    let requests = server.received_requests().await.unwrap();
    assert!(
        requests
            .iter()
            .any(|r| String::from_utf8_lossy(&r.body).contains("function_call_output"))
    );
    let context = runtime.store.context(0).unwrap();
    let first_end = context
        .iter()
        .position(|m| m.text == "File verified")
        .unwrap();
    let second_start = context
        .iter()
        .position(|m| m.text == "A new independent request")
        .unwrap();
    assert!(first_end < second_start);
    assert!(
        !runtime
            .status()
            .await
            .unwrap()
            .to_string()
            .contains("fake-private-test-key")
    );
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn explicit_grant_cannot_authorize_different_arguments() {
    let (dir, runtime, _server) = fixture(vec![
        call(
            "approval",
            "request_approval",
            json!({"tool":"file_write","arguments":{"path":"one.txt","content":"one"}}),
        ),
        call(
            "changed",
            "file_write",
            json!({"path":"two.txt","content":"two"}),
        ),
        answer("The action was denied"),
    ])
    .await;
    let workers = runtime.start();
    let task = runtime.submit("Create one file", "web").await.unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let first = pending(&runtime).await;
    runtime
        .submit(&format!("aprobar {}", first["id"].as_str().unwrap()), "web")
        .await
        .unwrap();
    let second = pending(&runtime).await;
    assert_ne!(first["id"], second["id"]);
    runtime
        .submit(
            &format!("rechazar {}", second["id"].as_str().unwrap()),
            "web",
        )
        .await
        .unwrap();
    wait_for(&runtime, &task, "completed").await;
    assert!(!dir.path().join("one.txt").exists());
    assert!(!dir.path().join("two.txt").exists());
    let events = runtime.store.events(&task, 0, 100).unwrap();
    assert!(events.iter().any(|e| e["data"]["result"]["denied"] == true));
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn cancellation_while_waiting_never_executes_the_action() {
    let (dir, runtime, _server) = fixture(vec![call(
        "write",
        "file_write",
        json!({"path":"cancelled.txt","content":"no"}),
    )])
    .await;
    let workers = runtime.start();
    let task = runtime.submit("Create a file", "web").await.unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    pending(&runtime).await;
    runtime.submit("cancelar", "web").await.unwrap();
    wait_for(&runtime, &task, "cancelled").await;
    assert!(!dir.path().join("cancelled.txt").exists());
    assert!(runtime.status().await.unwrap()["pending"].is_null());
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn clarification_answer_bypasses_queue_without_consuming_other_requests() {
    let (_dir, runtime, _server) = fixture(vec![
        call("ask", "ask_user", json!({"question":"Which file?"})),
        answer("I received the answer"),
    ])
    .await;
    let workers = runtime.start();
    let task = runtime.submit("Read the file", "web").await.unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let question = pending(&runtime).await;
    let later = runtime.submit("Another request", "web").await.unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    runtime
        .submit(
            &format!("responder {} notes.md", question["id"].as_str().unwrap()),
            "web",
        )
        .await
        .unwrap();
    wait_for(&runtime, &task, "completed").await;
    wait_for(&runtime, &later, "completed").await;
    assert!(
        runtime
            .store
            .events(&task, 0, 100)
            .unwrap()
            .iter()
            .any(|e| e["data"]["result"]["answer"] == "notes.md")
    );
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn repeated_identical_tools_stop_without_erasing_the_audit_trail() {
    let (_dir, runtime, _server) = fixture(
        (1..=3)
            .map(|i| {
                call(
                    &format!("repeat-{i}"),
                    "memory_search",
                    json!({"query":"absent"}),
                )
            })
            .collect(),
    )
    .await;
    let workers = runtime.start();
    let task = runtime.submit("Find absent memory", "web").await.unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    wait_for(&runtime, &task, "failed").await;
    assert!(
        runtime
            .store
            .task(&task)
            .unwrap()
            .error
            .unwrap()
            .contains("avoid a loop")
    );
    assert_eq!(
        runtime
            .store
            .events(&task, 0, 100)
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "tool_result")
            .count(),
        3
    );
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn invalid_credentials_never_complete_setup() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(401).set_body_string("sensitive remote error body"))
        .mount(&server)
        .await;
    let mut settings = Settings::initial(dir.path().to_owned());
    settings.base_url = Some(server.uri());
    let error = runtime
        .configure(settings, Some("fake-key".into()))
        .await
        .err()
        .unwrap();
    assert!(!error.to_string().contains("sensitive"));
    assert_eq!(runtime.status().await.unwrap()["ready"], false);
    assert!(runtime.settings().unwrap().is_none());
}

#[tokio::test]
async fn reminders_fire_once_and_persist_without_catch_up() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data");
    let runtime = Runtime::open(&path, Some(&path.join("skills")))
        .await
        .unwrap();
    let future = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
    let created=runtime.schedule_manage(json!({"action":"create","name":"Break","content":"Take a break","mode":"reminder","kind":"once","at":future})).await.unwrap();
    let id = created["id"].as_str().unwrap();
    let mut schedule = runtime
        .store
        .get::<Schedule>("schedules", id)
        .unwrap()
        .unwrap();
    schedule.next_run = Some(chrono::Utc::now() - chrono::Duration::seconds(1));
    runtime.store.put("schedules", id, &schedule).unwrap();
    runtime.tick_schedules().await.unwrap();
    runtime.tick_schedules().await.unwrap();
    assert_eq!(runtime.store.conversation(None, 100).unwrap().len(), 1);
    assert_eq!(
        runtime
            .store
            .get::<Schedule>("schedules", id)
            .unwrap()
            .unwrap()
            .status,
        "completed"
    );
    schedule.status = "active".into();
    runtime.store.put("schedules", id, &schedule).unwrap();
    drop(runtime);
    let runtime = Runtime::open(&path, Some(&path.join("skills")))
        .await
        .unwrap();
    assert_eq!(
        runtime
            .store
            .get::<Schedule>("schedules", id)
            .unwrap()
            .unwrap()
            .status,
        "active"
    );
    assert_eq!(runtime.store.conversation(None, 100).unwrap().len(), 1);
    // An overdue one-time spec becomes missed instead of firing on restart.
    let mut schedule = runtime
        .store
        .get::<Schedule>("schedules", id)
        .unwrap()
        .unwrap();
    schedule.spec.at = Some((chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339());
    schedule.next_run = Some(chrono::Utc::now() - chrono::Duration::hours(1));
    runtime.store.put("schedules", id, &schedule).unwrap();
    drop(runtime);
    let runtime = Runtime::open(&path, Some(&path.join("skills")))
        .await
        .unwrap();
    assert_eq!(
        runtime
            .store
            .get::<Schedule>("schedules", id)
            .unwrap()
            .unwrap()
            .status,
        "missed"
    );
    assert_eq!(runtime.store.conversation(None, 100).unwrap().len(), 1);
}

#[tokio::test]
async fn cron_uses_peru_and_duplicate_schedules_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let args = json!({"action":"create","name":"Morning","content":"Review","mode":"reminder","kind":"cron","cron":"0 9 * * *"});
    let value = runtime.schedule_manage(args.clone()).await.unwrap();
    let schedule: Schedule = serde_json::from_value(value).unwrap();
    assert_eq!(
        schedule
            .next_run
            .unwrap()
            .with_timezone(&chrono_tz::America::Lima)
            .format("%H:%M")
            .to_string(),
        "09:00"
    );
    assert!(runtime.schedule_manage(args).await.is_err());
    runtime
        .schedule_manage(json!({"action":"pause","id":schedule.id}))
        .await
        .unwrap();
    let resumed = runtime
        .schedule_manage(json!({"action":"resume","id":schedule.id}))
        .await
        .unwrap();
    assert_eq!(resumed["status"], "active");
}

#[tokio::test]
async fn telegram_pairs_only_with_local_code_and_accepts_only_owner_text() {
    let (_dir, runtime, _server) = fixture(vec![]).await;
    let mut settings = TelegramSettings {
        enabled: true,
        username: "test_bot".into(),
        owner_id: None,
        pairing_code: Some("local-random-code".into()),
        offset: 0,
        last_error: None,
    };
    runtime
        .store
        .put("telegram", "settings", &settings)
        .unwrap();
    let update = |sender, text: &str| json!({"message":{"from":{"id":sender},"chat":{"id":sender,"type":"private"},"text":text}});
    runtime
        .receive_telegram(&update(41, "/start wrong"), &mut settings)
        .await
        .unwrap();
    assert!(settings.owner_id.is_none());
    runtime
        .receive_telegram(&update(42, "/start local-random-code"), &mut settings)
        .await
        .unwrap();
    assert_eq!(settings.owner_id, Some(42));
    assert!(settings.pairing_code.is_none());
    runtime
        .receive_telegram(&update(43, "Do something"), &mut settings)
        .await
        .unwrap();
    assert!(runtime.store.tasks(100).unwrap().is_empty());
    runtime.receive_telegram(&json!({"message":{"from":{"id":42},"chat":{"id":42,"type":"private"},"document":{"file_id":"ignored"}}}),&mut settings).await.unwrap();
    assert!(runtime.store.tasks(100).unwrap().is_empty());
    runtime
        .receive_telegram(&update(42, "Hello from Telegram"), &mut settings)
        .await
        .unwrap();
    let task = runtime.store.tasks(100).unwrap().pop().unwrap();
    assert_eq!(task.origin, "telegram");
    assert_eq!(task.session_id, runtime.store.session_id());
}

#[tokio::test]
async fn skill_installation_preserves_personal_edits_and_enabled_state() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data");
    let runtime = Runtime::open(&path, Some(&path.join("skills")))
        .await
        .unwrap();
    assert_eq!(runtime.skills().unwrap().len(), 4);
    let skill = runtime.skills_dir.join("conversation-recap/SKILL.md");
    std::fs::write(
        &skill,
        "---\ndescription: Custom recap\nversion: 2.0.0\n---\nPersonal instructions",
    )
    .unwrap();
    runtime
        .set_skill_enabled("conversation-recap", false)
        .unwrap();
    assert!(runtime.skill_load("conversation-recap").is_err());
    drop(runtime);
    let runtime = Runtime::open(&path, Some(&path.join("skills")))
        .await
        .unwrap();
    assert!(
        std::fs::read_to_string(skill)
            .unwrap()
            .contains("Personal instructions")
    );
    let skill = runtime
        .skills()
        .unwrap()
        .into_iter()
        .find(|s| s.name == "conversation-recap")
        .unwrap();
    assert_eq!(skill.version, "2.0.0");
    assert!(!skill.enabled);
}

#[tokio::test]
async fn compaction_keeps_original_history_and_complete_tool_pairs() {
    use fenix_core::{ContentBlock, Message, Role};
    let (_dir,runtime,_server)=fixture(vec![answer("Objective\nKeep the original decision.\nConstraints and preferences\nNone.\nProgress\nCompleted: previous work.\nKey decisions\nPreserve source history.\nRelevant files\nnotes.txt\nNext steps\nAnswer the owner.\nCritical context\nAssumptions are labeled."),answer("Continued with the summary")]).await;
    for i in 0..7 {
        let task = runtime
            .store
            .enqueue_bound(
                &format!("Previous task {i}"),
                "web",
                None,
                &fenix_storage::TaskBinding::default(),
            )
            .unwrap();
        runtime.store.claim_next().unwrap();
        let call_id = format!("older-call-{i}");
        runtime
            .store
            .append(
                Some(&task.id),
                &Message {
                    role: Role::Assistant,
                    content: vec![ContentBlock::ToolCall {
                        id: call_id.clone(),
                        name: "file_read".into(),
                        arguments: json!({"path":"notes.txt"}),
                    }],
                    provider_data: None,
                },
                false,
                true,
                "agent",
            )
            .unwrap();
        runtime
            .store
            .append(
                Some(&task.id),
                &Message {
                    role: Role::Tool,
                    content: vec![ContentBlock::ToolResult {
                        tool_call_id: call_id,
                        name: Some("file_read".into()),
                        content: json!({"text":"reference ".repeat(1200)}),
                        is_error: false,
                    }],
                    provider_data: None,
                },
                false,
                true,
                "tool",
            )
            .unwrap();
        runtime
            .store
            .append(
                Some(&task.id),
                &Message::text(Role::Assistant, format!("Completed earlier task {i}")),
                true,
                true,
                "agent",
            )
            .unwrap();
        runtime
            .store
            .set_task_status(&task.id, "completed", None)
            .unwrap();
    }
    let original = runtime.store.context(0).unwrap();
    let workers = runtime.start();
    let id = runtime
        .submit("Continue from the previous decisions", "web")
        .await
        .unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    wait_for(&runtime, &id, "completed").await;
    let summary = runtime
        .store
        .get::<Value>("scoped_context", "general")
        .unwrap()
        .unwrap();
    assert!(
        summary["text"]
            .as_str()
            .unwrap()
            .contains("Critical context")
    );
    let through = summary["through"].as_i64().unwrap();
    for task in original.iter().filter_map(|m| m.task_id.as_ref()) {
        let rows = original
            .iter()
            .filter(|m| m.task_id.as_ref() == Some(task))
            .collect::<Vec<_>>();
        assert!(rows.iter().all(|m| m.seq <= through) || rows.iter().all(|m| m.seq > through));
    }
    assert!(runtime.store.context(0).unwrap().len() > original.len());
    assert_eq!(
        runtime
            .store
            .history_search("Previous", 30)
            .unwrap()
            .iter()
            .filter(|m| m["text"].as_str().unwrap().starts_with("Previous task"))
            .count(),
        7
    );
    assert!(
        runtime
            .store
            .events(&id, 0, 100)
            .unwrap()
            .iter()
            .any(|e| e["kind"] == "context_compacted")
    );
    stop(&runtime, workers).await;
}

#[test]
fn a_project_named_default_cannot_collide_with_unscoped_credentials() {
    let mut settings = Settings::initial(std::path::PathBuf::from("/tmp"));
    let unscoped = settings.credential_id();
    settings.project = Some("default".into());
    assert_ne!(unscoped, settings.credential_id());
}

#[tokio::test]
async fn explicit_denial_overrides_a_previously_allowing_policy() {
    let (dir, runtime, _server) = fixture(vec![
        answer("OK"),
        call(
            "permission",
            "request_approval",
            json!({"tool":"file_write","arguments":{"path":"denied.txt","content":"blocked"}}),
        ),
        call(
            "effect",
            "file_write",
            json!({"path":"denied.txt","content":"blocked"}),
        ),
        answer("The denied action was not performed"),
    ])
    .await;
    let mut settings = runtime.settings().unwrap().unwrap();
    settings.file_write_policy = fenix_agent::config::ActionPolicy::Allow;
    runtime.configure(settings, None).await.unwrap();
    let workers = runtime.start();
    let task = runtime.submit("Check permissions", "web").await.unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let request = pending(&runtime).await;
    runtime
        .submit(
            &format!("rechazar {}", request["id"].as_str().unwrap()),
            "web",
        )
        .await
        .unwrap();
    wait_for(&runtime, &task, "completed").await;
    assert!(!dir.path().join("denied.txt").exists());
    stop(&runtime, workers).await;
}
