use fenix_agent::{Runtime, config::Settings};
use fenix_storage::{Project, ProjectInput};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use wiremock::{
    Mock, MockServer, Request, ResponseTemplate,
    matchers::{method, path},
};

fn answer(text: &str) -> Value {
    json!({"id":"project-answer","status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":text}]}],"usage":{"input_tokens":10,"output_tokens":5,"total_tokens":15}})
}
fn call(id: &str, name: &str, args: Value) -> Value {
    json!({"id":"project-tool","status":"completed","output":[{"type":"function_call","call_id":id,"name":name,"arguments":args.to_string()}],"usage":{"input_tokens":10,"output_tokens":5,"total_tokens":15}})
}
fn has_result(body: &Value, id: &str) -> bool {
    body["input"]
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m["type"] == "function_call_output" && m["call_id"] == id)
}
fn project(runtime: &Runtime, root: &std::path::Path, name: &str, brief: &str) -> Project {
    let workspace = root.join(name);
    std::fs::create_dir(&workspace).unwrap();
    runtime
        .save_project(
            None,
            ProjectInput {
                name: name.into(),
                workspace: Some(workspace.to_string_lossy().into_owned()),
                brief: brief.into(),
                ..Default::default()
            },
        )
        .unwrap()
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
        .configure(settings, Some("project-fixture-key".into()))
        .await
        .unwrap();
    server.reset().await;
    (dir, runtime, server)
}
async fn wait_for(runtime: &Runtime, id: &str, status: &str) {
    tokio::time::timeout(Duration::from_secs(12), async {
        loop {
            let task = runtime.store.task(id).unwrap();
            if task.status == status {
                break;
            }
            assert!(
                !matches!(task.status.as_str(), "failed" | "interrupted"),
                "{}",
                json!(task)
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn compound_request_keeps_general_chat_and_project_work_separate_through_late_approval() {
    let (dir, runtime, server) = fixture().await;
    let a = project(
        &runtime,
        dir.path(),
        "A",
        "A_PRIVATE_REFERENCE: use SQLite for this project only.",
    );
    let b = project(
        &runtime,
        dir.path(),
        "B",
        "B_PRIVATE_REFERENCE: reduce login latency, preserving its behavior.",
    );
    runtime
        .store
        .save_memory(
            None,
            "login preference",
            "GLOBAL_LOGIN_PREFERENCE: explain changes briefly.",
            "owner",
        )
        .unwrap();
    runtime
        .store
        .save_memory_scoped(
            None,
            "login A",
            "A_MEMORY_ONLY: login uses service Alpha.",
            "owner",
            Some(&a.id),
            None,
        )
        .unwrap();
    runtime
        .store
        .save_memory_scoped(
            None,
            "login B",
            "B_MEMORY_ONLY: login uses service Beta.",
            "owner",
            Some(&b.id),
            None,
        )
        .unwrap();
    let a_id = a.id.clone();
    let b_id = b.id.clone();
    Mock::given(method("POST")).and(path("/responses")).respond_with(move |request: &Request| {
        let body:Value = serde_json::from_slice(&request.body).unwrap();
        let input = body["input"].to_string();
        if input.contains("You route an owner's message") {
            assert!(!input.contains("A_PRIVATE_REFERENCE"));
            assert!(!input.contains("B_PRIVATE_REFERENCE"));
            return ResponseTemplate::new(200).set_body_json(answer(&json!({
                "mode":"coordination","project_id":null,"read_project_ids":[a_id,b_id],"focus_project_id":b_id,
                "related_task_ids":[],"intents":[
                    {"project_id":a_id,"request":"Dime cómo va A","action":"read","expected_output":"Recorded status"},
                    {"project_id":b_id,"request":"Mejora el inicio de sesión de B","action":"work","expected_output":"Lower login latency"}
                ],"resolved_request":"Dime cómo va A y después mejora el inicio de sesión de B",
                "explanation":"Read A first and delegate B.","question":null
            }).to_string()));
        }
        if input.contains("Delegated subagent task") {
            assert!(input.contains("B_PRIVATE_REFERENCE"));
            assert!(input.contains("B_MEMORY_ONLY"));
            assert!(input.contains("GLOBAL_LOGIN_PREFERENCE"));
            assert!(!input.contains("A_PRIVATE_REFERENCE"));
            assert!(!input.contains("A_MEMORY_ONLY"));
            assert!(!input.contains("GENERAL_THREAD_ONLY"));
            if has_result(&body,"write-b") {
                return ResponseTemplate::new(200).set_body_json(call("finish-b","subagent_finish",json!({"status":"completed","summary":"B result saved","evidence":"file_write returned the created path","artifacts":"login-note.txt"})));
            }
            return ResponseTemplate::new(200).set_body_json(call("write-b","file_write",json!({"path":"login-note.txt","content":"B login result"})));
        }
        if input.contains("Internal subagent report") {
            return ResponseTemplate::new(200).set_body_json(answer("Proyecto B: resultado listo."));
        }
        let latest = body["input"].as_array().unwrap().iter().rev().find(|m|m["role"]=="user").unwrap().to_string();
        if latest.contains("hola") {
            assert!(!input.contains("A_PRIVATE_REFERENCE"));
            assert!(!input.contains("B_PRIVATE_REFERENCE"));
            return ResponseTemplate::new(200).set_body_json(answer("GENERAL_THREAD_ONLY: aquí seguimos conversando."));
        }
        if latest.contains("Dime cómo va A y después") {
            assert!(input.contains("GENERAL_THREAD_ONLY"));
            if has_result(&body,"spawn-b") {
                return ResponseTemplate::new(200).set_body_json(answer("A: estado consultado. B: trabajo delegado; sigo disponible."));
            }
            if has_result(&body,"status-a") {
                return ResponseTemplate::new(200).set_body_json(call("spawn-b","subagent_spawn",json!({"project_id":b_id,"objective":"Improve login latency in B","expected_output":"Write login-note.txt with the result; preserve login behavior"})));
            }
            return ResponseTemplate::new(200).set_body_json(call("status-a","project_manage",json!({"action":"get","project_id":a_id})));
        }
        assert!(latest.contains("estado del proyecto A"));
        assert!(input.contains("A_PRIVATE_REFERENCE"));
        assert!(!input.contains("B_PRIVATE_REFERENCE"));
        ResponseTemplate::new(200).set_body_json(answer("Proyecto A: seguimos aquí."))
    }).mount(&server).await;
    let workers = runtime.start();
    let hello = runtime.submit("hola", "web").await.unwrap();
    wait_for(&runtime, hello["task_id"].as_str().unwrap(), "completed").await;
    let compound = runtime
        .submit(
            "Dime cómo va A y después mejora el inicio de sesión de B",
            "web",
        )
        .await
        .unwrap();
    let parent = compound["task_id"].as_str().unwrap();
    wait_for(&runtime, parent, "completed").await;
    let child = runtime.store.subagents(Some(parent), 10).unwrap().remove(0);
    wait_for(&runtime, &child.id, "waiting").await;
    assert_eq!(
        runtime
            .store
            .task_binding(&child.id)
            .unwrap()
            .project_id
            .as_deref(),
        Some(b.id.as_str())
    );
    assert_eq!(
        runtime.store.subagents(Some(parent), 10).unwrap().len(),
        1,
        "No subagent for A's simple status query"
    );
    let foreground = runtime
        .submit("Dime el estado del proyecto A", "web")
        .await
        .unwrap();
    wait_for(
        &runtime,
        foreground["task_id"].as_str().unwrap(),
        "completed",
    )
    .await;
    assert_eq!(
        runtime.store.conversation_focus().unwrap(),
        Some(a.id.clone())
    );
    let response = runtime
        .submit_referenced("sí", "web", Some(&child.id))
        .await
        .unwrap();
    assert!(response.get("answered").is_some());
    wait_for(&runtime, &child.id, "completed").await;
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if runtime
                .store
                .tasks(30)
                .unwrap()
                .iter()
                .any(|t| t.origin == "subagent_report" && t.status == "completed")
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        runtime.store.conversation_focus().unwrap(),
        Some(a.id.clone()),
        "Late B report must not steal foreground focus"
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("B/login-note.txt")).unwrap(),
        "B login result"
    );
    assert!(!dir.path().join("A/login-note.txt").exists());
    assert!(!dir.path().join("login-note.txt").exists());
    let events = runtime.store.events(parent, 0, 100).unwrap();
    assert_eq!(
        events
            .iter()
            .filter(|e| e["kind"] == "project_routing_response")
            .count(),
        1
    );
    runtime.shutdown().await.unwrap();
    for worker in workers {
        worker.await.unwrap();
    }
}

#[tokio::test]
async fn scheduled_project_task_keeps_its_scope_when_foreground_changes() {
    let (dir, runtime, _server) = fixture().await;
    let a = project(&runtime, dir.path(), "A", "A context");
    let b = project(&runtime, dir.path(), "B", "B context");
    let schedule = runtime.schedule_manage(json!({"action":"create","name":"A task","content":"Inspect project files","mode":"task","kind":"interval","interval_seconds":60,"project_id":a.id})).await.unwrap();
    let mut saved: fenix_agent::schedules::Schedule = serde_json::from_value(schedule).unwrap();
    saved.next_run = Some(chrono::Utc::now() - chrono::Duration::seconds(1));
    runtime.store.put("schedules", &saved.id, &saved).unwrap();
    runtime.store.set_conversation_focus(Some(&b.id)).unwrap();
    runtime.tick_schedules().await.unwrap();
    let task = runtime.store.claim_next().unwrap().unwrap();
    let binding = runtime.store.task_binding(&task.id).unwrap();
    assert!(binding.bound);
    assert_eq!(binding.project_id, Some(a.id));
    assert_eq!(binding.workspace, a.workspace);
    assert_eq!(runtime.store.conversation_focus().unwrap(), Some(b.id));
}
