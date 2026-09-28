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
            let mut queue = steps.lock().unwrap();
            if let Some(next) = queue.front()
                && next["status"] == "completed"
                && let Some(name) = next["output"][0]["name"].as_str()
            {
                let tools = body["tools"].as_array();
                let exposed = tools.is_some_and(|ts| ts.iter().any(|t| t["name"] == name));
                let available = tools.is_some_and(|ts| {
                    ts.iter().any(|t| {
                        t["name"] == "tool_search"
                            && t["description"].as_str().is_some_and(|s| s.contains(name))
                    })
                });
                if !exposed && available {
                    return ResponseTemplate::new(200).set_body_json(call(
                        &format!("load-{name}"),
                        "tool_search",
                        json!({"names":[name]}),
                    ));
                }
            }
            ResponseTemplate::new(200)
                .set_body_json(queue.pop_front().unwrap_or_else(|| answer("Handled")))
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
    wait_for_with_timeout(runtime, id, status, Duration::from_secs(5)).await;
}
async fn wait_for_with_timeout(runtime: &Runtime, id: &str, status: &str, timeout: Duration) {
    tokio::time::timeout(timeout, async {
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
async fn mandatory_approval_waits_without_blocking_independent_conversation() {
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
    wait_for(&runtime, &second, "completed").await;
    assert_eq!(runtime.store.task(&first).unwrap().status, "waiting");
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
    let first_start = context
        .iter()
        .position(|m| m.task_id.as_deref() == Some(&first))
        .unwrap();
    let second_end = context
        .iter()
        .rposition(|m| m.task_id.as_deref() == Some(&second))
        .unwrap();
    assert!(
        second_end < first_start,
        "Completed turns are sealed independently; a waiting tool sequence never enters another turn"
    );
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
        (1..=6)
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
            .contains("Recovery budget exhausted for no_progress")
    );
    assert_eq!(
        runtime
            .store
            .events(&task, 0, 100)
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "tool_result" && e["data"]["tool"] == "memory_search")
            .count(),
        6
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
    // Telegram now enters through the independently configured conversational model.
    let conversation = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(answer(r#"{"reply":"Forwarded","action":"delegate"}"#)),
        )
        .mount(&conversation)
        .await;
    runtime
        .configure_live(
            fenix_agent::live::LiveSettings {
                reasoning_effort: None,
                provider: fenix_core::ProviderId::OpenAi,
                model: "conversation-test".into(),
                base_url: Some(conversation.uri()),
            },
            Some("conversation-test-key".into()),
        )
        .await
        .unwrap();
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
    let bundled = std::fs::read_dir(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../resources/skills"),
    )
    .unwrap()
    .filter(|entry| entry.as_ref().unwrap().path().join("SKILL.md").is_file())
    .count();
    assert_eq!(runtime.skills().unwrap().len(), bundled);
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

#[tokio::test]
async fn file_patch_uses_exact_textual_approval_and_preserves_existing_files() {
    let (dir,runtime,server)=fixture(vec![call("patch-1","file_patch",json!({"patch":"*** Begin Patch\n*** Add File: created.rs\n+fn main() {}\n*** End Patch"})),answer("Patch complete")]).await;
    let workers = runtime.start();
    let task = runtime
        .submit("Create created.rs using a patch", "web")
        .await
        .unwrap();
    let id = task["task_id"].as_str().unwrap();
    let question = pending(&runtime).await;
    assert_eq!(question["action"]["tool"], "file_patch");
    assert!(!dir.path().join("created.rs").exists());
    runtime
        .submit(
            &format!("aprobar {}", question["id"].as_str().unwrap()),
            "web",
        )
        .await
        .unwrap();
    wait_for(&runtime, id, "completed").await;
    assert_eq!(
        std::fs::read_to_string(dir.path().join("created.rs")).unwrap(),
        "fn main() {}\n"
    );
    stop(&runtime, workers).await;
    server.verify().await;
}

#[tokio::test]
async fn nested_guidance_is_consumed_before_a_same_batch_edit() {
    let mut batch = call("read-rules", "file_read", json!({"path":"src/a.rs"}));
    batch["output"].as_array_mut().unwrap().push(json!({"type":"function_call","call_id":"early-edit","name":"file_edit","arguments":json!({"path":"src/a.rs","expected":"old","replacement":"new"}).to_string()}));
    let (dir, runtime, server) = fixture(vec![
        batch,
        call(
            "retry-edit",
            "file_edit",
            json!({"path":"src/a.rs","expected":"old","replacement":"new"}),
        ),
        answer("Done"),
    ])
    .await;
    std::fs::create_dir(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/AGENTS.md"), "Preserve the public API.").unwrap();
    std::fs::write(dir.path().join("src/a.rs"), "old").unwrap();
    let workers = runtime.start();
    let task = runtime.submit("Update src/a.rs", "web").await.unwrap();
    let id = task["task_id"].as_str().unwrap();
    let question = pending(&runtime).await;
    assert_eq!(
        std::fs::read_to_string(dir.path().join("src/a.rs")).unwrap(),
        "old"
    );
    let events = runtime.store.events(id, 0, 100).unwrap();
    let blocked = events
        .iter()
        .find(|e| e["kind"] == "tool_result" && e["data"]["call_id"] == "early-edit")
        .unwrap();
    assert_eq!(blocked["data"]["result"]["executed"], false);
    runtime
        .submit(
            &format!("aprobar {}", question["id"].as_str().unwrap()),
            "web",
        )
        .await
        .unwrap();
    wait_for(&runtime, id, "completed").await;
    assert_eq!(
        std::fs::read_to_string(dir.path().join("src/a.rs")).unwrap(),
        "new"
    );
    stop(&runtime, workers).await;
    server.verify().await;
}

#[tokio::test]
async fn long_active_task_compacts_and_continues_without_replaying_tools() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let (dir, runtime, server) = fixture(vec![]).await;
    let text = (0..5000)
        .map(|i| format!("line {i}: {}\n", "reference ".repeat(15)))
        .collect::<String>();
    std::fs::write(dir.path().join("large.txt"), text).unwrap();
    let mut settings = runtime.settings().unwrap().unwrap();
    settings.coding.context_window_tokens = 65536;
    settings.coding.compact_at_tokens = 16000;
    settings.coding.keep_recent_tokens = 5000;
    runtime.configure(settings, None).await.unwrap();
    server.reset().await;
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = calls.clone();
    Mock::given(method("POST")).and(path("/responses")).respond_with(move |request:&Request| {
        let body:Value=serde_json::from_slice(&request.body).unwrap();
        let reply=if body.get("tools").is_none() {answer("Objective: inspect large.txt. Progress: prior reads completed; do not repeat them. Next steps: continue remaining reads. Critical context: retain the same task.")} else {
            let inputs=body["input"].as_array().unwrap();
            for item in inputs.iter().filter(|v|v["type"]=="function_call_output") {assert!(inputs.iter().any(|v|v["type"]=="function_call" && v["call_id"]==item["call_id"]));}
            let i=counter.fetch_add(1,Ordering::SeqCst);
            if i<9 {call(&format!("read-{i}"),"file_read",json!({"path":"large.txt","start_line":1+i*500,"line_count":500}))} else {answer("All nine distinct ranges inspected")}
        };ResponseTemplate::new(200).set_body_json(reply)
    }).mount(&server).await;
    let workers = runtime.start();
    let task = runtime
        .submit("Inspect large.txt in nine ranges", "web")
        .await
        .unwrap();
    let id = task["task_id"].as_str().unwrap();
    // Several large reads and summary calls need headroom on shared CI hosts.
    wait_for_with_timeout(&runtime, id, "completed", Duration::from_secs(30)).await;
    let events = runtime.store.events(id, 0, 1000).unwrap();
    assert_eq!(
        events
            .iter()
            .filter(|e| e["kind"] == "tool_started")
            .count(),
        9
    );
    assert!(
        events
            .iter()
            .any(|e| e["kind"] == "active_context_compacted")
    );
    assert_eq!(calls.load(Ordering::SeqCst), 10);
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn persistent_processes_reuse_stdin_across_turns_and_load_tools_on_demand() {
    use fenix_agent::config::ActionPolicy;
    use std::sync::atomic::{AtomicUsize, Ordering};
    let (dir, runtime, server) = fixture(vec![]).await;
    let mut settings = runtime.settings().unwrap().unwrap();
    settings.command_policy = ActionPolicy::Allow;
    runtime.configure(settings, None).await.unwrap();
    server.reset().await;
    let root = dir.path().to_string_lossy().into_owned();
    let process_id = Arc::new(Mutex::new(String::new()));
    let saved = process_id.clone();
    let counter = Arc::new(AtomicUsize::new(0));
    let steps = counter.clone();
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(move |request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            let step = steps.fetch_add(1, Ordering::SeqCst);
            let response = match step {
                0 => call(
                    "start-service",
                    "command_run",
                    json!({"command":"cat","cwd":root,"persistent":true}),
                ),
                1 => {
                    let result = body["input"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|m| {
                            m["call_id"] == "start-service" && m["type"] == "function_call_output"
                        })
                        .unwrap();
                    let output: Value =
                        serde_json::from_str(result["output"].as_str().unwrap()).unwrap();
                    *saved.lock().unwrap() = output["process_id"].as_str().unwrap().into();
                    answer("The persistent session is ready")
                }
                2 => {
                    assert!(
                        !body["tools"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|t| t["name"] == "command_input")
                    );
                    call(
                        "load-input",
                        "tool_search",
                        json!({"names":["command_input","command_cancel"]}),
                    )
                }
                3 => {
                    assert!(
                        body["tools"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|t| t["name"] == "command_input")
                    );
                    call(
                        "stdin",
                        "command_input",
                        json!({"process_id":*saved.lock().unwrap(),"text":"hello fixture\n"}),
                    )
                }
                4 => call(
                    "poll",
                    "command_status",
                    json!({"process_id":*saved.lock().unwrap(),"wait_ms":100}),
                ),
                5 => {
                    let result = body["input"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|m| m["call_id"] == "poll" && m["type"] == "function_call_output")
                        .unwrap();
                    assert!(result["output"].as_str().unwrap().contains("hello fixture"));
                    call(
                        "stop-service",
                        "command_cancel",
                        json!({"process_id":*saved.lock().unwrap()}),
                    )
                }
                _ => answer("The session echoed the input and is stopped"),
            };
            ResponseTemplate::new(200).set_body_json(response)
        })
        .mount(&server)
        .await;
    let workers = runtime.start();
    let first = runtime
        .submit("Start a persistent echo session", "web")
        .await
        .unwrap();
    wait_for(&runtime, first["task_id"].as_str().unwrap(), "completed").await;
    assert!(!process_id.lock().unwrap().is_empty());
    let next = runtime
        .submit(
            "Send hello fixture to the existing session and stop it",
            "web",
        )
        .await
        .unwrap();
    wait_for(&runtime, next["task_id"].as_str().unwrap(), "completed").await;
    assert_eq!(counter.load(Ordering::SeqCst), 7);
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn tasks_can_progress_beyond_the_previous_32_step_limit() {
    let mut script = (0..40)
        .map(|i| {
            call(
                &format!("read-{i}"),
                "file_read",
                json!({"path":format!("source-{i}.txt")}),
            )
        })
        .collect::<Vec<_>>();
    script.push(answer("All forty distinct sources read"));
    let (dir, runtime, _server) = fixture(script).await;
    for i in 0..40 {
        std::fs::write(
            dir.path().join(format!("source-{i}.txt")),
            format!("Evidence {i}"),
        )
        .unwrap();
    }
    let mut settings = runtime.settings().unwrap().unwrap();
    settings.coding.context_window_tokens = 262144;
    settings.coding.compact_at_tokens = 200000;
    runtime.store.put("settings", "active", &settings).unwrap();
    // Re-open to exercise persisted settings without consuming a scripted model response.
    let data = runtime.data_dir.clone();
    let skills = runtime.skills_dir.clone();
    drop(runtime);
    let runtime = Runtime::open(&data, Some(&skills)).await.unwrap();
    let workers = runtime.start();
    let task = runtime.submit("Read forty sources", "web").await.unwrap();
    wait_for_with_timeout(
        &runtime,
        task["task_id"].as_str().unwrap(),
        "completed",
        Duration::from_secs(30),
    )
    .await;
    assert_eq!(
        runtime
            .store
            .get::<usize>("execution_steps", task["task_id"].as_str().unwrap())
            .unwrap(),
        Some(41)
    );
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn agent_discovers_browser_tools_and_enforces_personal_profile_handoff() {
    let (_dir, runtime, server) = fixture(vec![
        call(
            "discover-browser",
            "tool_search",
            json!({"names":["browser_open","browser_resize"]}),
        ),
        call(
            "open-browser",
            "browser_open",
            json!({"profile":"personal","url":"https://example.com"}),
        ),
        answer("Enable agent access in the Browser view first."),
    ])
    .await;
    runtime.browser.profile("personal", false).await.unwrap();
    let workers = runtime.start();
    let submitted = runtime
        .submit("Use my personal browser profile", "web")
        .await
        .unwrap();
    wait_for(
        &runtime,
        submitted["task_id"].as_str().unwrap(),
        "completed",
    )
    .await;
    let requests = server.received_requests().await.unwrap();
    let last: Value = serde_json::from_slice(&requests.last().unwrap().body).unwrap();
    assert!(
        last["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["name"] == "browser_open")
    );
    assert!(last["input"].as_array().unwrap().iter().any(|v| {
        v["type"] == "function_call_output"
            && v["output"]
                .as_str()
                .unwrap_or("")
                .contains("authorize agent access")
    }));
    assert!(
        runtime.browser.status().await.unwrap()["sessions"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        runtime
            .skill_load("browser-navigation")
            .unwrap()
            .contains("Owner handoff")
    );
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn truncated_action_is_not_executed_and_recovery_keeps_task_identity() {
    let mut truncated = call(
        "partial",
        "file_write",
        json!({"path":"must-not-exist.txt","content":"partial"}),
    );
    truncated["status"] = json!("incomplete");
    truncated["incomplete_details"] = json!({"reason":"max_output_tokens"});
    let (dir, runtime, server) = fixture(vec![
        truncated,
        answer("Recovered without executing the partial action."),
    ])
    .await;
    let workers = runtime.start();
    let task = runtime
        .submit("Respond after a truncated generation", "web")
        .await
        .unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    wait_for(&runtime, &task, "completed").await;
    assert!(!dir.path().join("must-not-exist.txt").exists());
    let events = runtime.store.events(&task, 0, 100).unwrap();
    assert!(
        events
            .iter()
            .any(|e| e["kind"] == "task_recovery" && e["data"]["reason"] == "output_length")
    );
    assert!(!events.iter().any(|e| e["kind"] == "tool_started"));
    let requests = server.received_requests().await.unwrap();
    let last: Value = serde_json::from_slice(&requests.last().unwrap().body).unwrap();
    assert!(!last.to_string().contains("must-not-exist.txt"));
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn length_recovery_has_a_durable_finite_budget() {
    let mut response = answer("unfinished");
    response["status"] = json!("incomplete");
    response["incomplete_details"] = json!({"reason":"max_output_tokens"});
    let (_dir, runtime, _server) = fixture(vec![response; 5]).await;
    let workers = runtime.start();
    let task = runtime.submit("Bound recovery", "web").await.unwrap()["task_id"]
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
            .contains("Recovery budget exhausted")
    );
    assert_eq!(
        runtime
            .store
            .events(&task, 0, 100)
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "task_recovery")
            .count(),
        3
    );
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn large_receipts_keep_tail_evidence_and_full_redacted_journal() {
    let (dir, runtime, server) = fixture(vec![
        call("read-large", "file_read", json!({"path":"receipt.txt"})),
        answer("Read the final diagnostic."),
    ])
    .await;
    let body = format!(
        "START {} MIDDLE_WITNESS {} fake-private-test-key FINAL_DIAGNOSTIC",
        "x".repeat(10_000),
        "y".repeat(10_000)
    );
    std::fs::write(dir.path().join("receipt.txt"), body).unwrap();
    let workers = runtime.start();
    let submitted = runtime.submit("Read receipt.txt", "web").await.unwrap();
    let id = submitted["task_id"].as_str().unwrap();
    wait_for(&runtime, id, "completed").await;
    let requests = server.received_requests().await.unwrap();
    let last: Value = serde_json::from_slice(&requests.last().unwrap().body).unwrap();
    let receipt: Value = serde_json::from_str(
        last["input"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["type"] == "function_call_output" && item["call_id"] == "read-large")
            .unwrap()["output"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(receipt["truncated"], true);
    let preview = receipt["preview"].as_str().unwrap();
    assert!(preview.contains("FINAL_DIAGNOSTIC"));
    assert!(!preview.contains("MIDDLE_WITNESS"));
    assert!(!preview.contains("fake-private-test-key"));
    let events = runtime.store.events(id, 0, 1000).unwrap();
    let journal = events
        .iter()
        .find(|event| event["kind"] == "tool_result" && event["data"]["call_id"] == "read-large")
        .unwrap()
        .to_string();
    assert!(journal.contains("MIDDLE_WITNESS"));
    assert!(journal.contains("FINAL_DIAGNOSTIC"));
    assert!(!journal.contains("fake-private-test-key"));
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn native_document_creation_waits_for_approval_and_records_the_effect() {
    let (dir,runtime,_server)=fixture(vec![
        call("load-documents","skill_load",json!({"name":"documents-word"})),
        call("create-document","document_create",json!({"path":"approved.docx","document":{"format":"docx","paragraphs":[{"text":"Approved report"}]}})),
        answer("Document created"),
    ]).await;
    let workers = runtime.start();
    let task = runtime
        .submit("Create approved.docx with the text Approved report", "web")
        .await
        .unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let approval = pending(&runtime).await;
    assert!(!dir.path().join("approved.docx").exists());
    runtime
        .submit(
            &format!("aprobar {}", approval["id"].as_str().unwrap()),
            "web",
        )
        .await
        .unwrap();
    wait_for(&runtime, &task, "completed").await;
    let bytes = std::fs::read(dir.path().join("approved.docx")).unwrap();
    assert!(
        fenix_documents::inspect(&bytes, 0, 10)
            .unwrap()
            .to_string()
            .contains("Approved report")
    );
    let events = runtime.store.events(&task, 0, 100).unwrap();
    assert!(
        events
            .iter()
            .any(|e| e["kind"] == "file_effect_started" && e["data"]["tool"] == "document_create")
    );
    assert!(events.iter().any(|e| e["kind"] == "file_effect_finished" && e["data"]["result"]["verified"] == true));
    stop(&runtime, workers).await;
}

#[tokio::test]
async fn agent_discovers_design_tools_presents_choices_and_publishes_a_visual() {
    let (_dir, runtime, _server) = fixture(vec![
        call("design-options", "design_present", json!({"template_ids":["decision-flow","login-sequence"]})),
        call("design-output", "design_publish", json!({"design":{"title":"Login diagram","kind":"diagram","content":{"source":"flowchart LR\n User --> Login\n Login --> Session"}}})),
        answer("The login diagram is available in Design."),
    ]).await;
    let workers = runtime.start();
    let task = runtime
        .submit("Show two diagram options and create a login diagram", "web")
        .await
        .unwrap();
    let task_id = task["task_id"].as_str().unwrap();
    wait_for_with_timeout(&runtime, task_id, "completed", Duration::from_secs(15)).await;
    let results = runtime.store.designs(None, true, 0, 20).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["title"], "Login diagram");
    assert_eq!(results[0]["task_id"], task_id);
    assert_eq!(
        runtime.store.records::<Value>("design_offer").unwrap()[0]["templates"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    stop(&runtime, workers).await;
}
