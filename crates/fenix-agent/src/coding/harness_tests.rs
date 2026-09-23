use crate::{Runtime, config::Settings};
use anyhow::Result;
use serde_json::json;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn short_commands_modes_process_references_and_environment() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let workspace = dir.path().join("workspace");
    std::fs::create_dir(&workspace)?;
    let runtime =
        Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    let settings = Settings::initial(workspace.clone());
    let task = runtime.store.enqueue_bound(
        "Repair software",
        "web",
        None,
        &fenix_storage::TaskBinding {
            bound: true,
            ..Default::default()
        },
    )?;
    let cancel = CancellationToken::new();
    std::fs::write(workspace.join("input.txt"), "input")?;
    let result = runtime.execute_tool(&task.id, "test", &settings, "command_run", &json!({"command":"sleep 0.15; printf done; printf changed > output.txt","cwd":workspace,"execution_mode":"workspace","input_paths":["input.txt"],"yield_ms":3000}), &cancel).await?;
    assert_eq!(result["exit_code"], 0);
    assert_eq!(result["output"], "done");
    assert_eq!(result["execution_mode"], "workspace");
    assert_eq!(
        std::fs::read_to_string(workspace.join("output.txt"))?,
        "changed"
    );
    let id = result["process_id"].as_str().unwrap();
    assert!(id.starts_with("p_") && id.len() == 18);
    assert!(
        runtime
            .processes
            .initial_wait(&task.id, &workspace, 1, false)
            .await
            >= 200
    );
    assert_eq!(
        runtime
            .processes
            .initial_wait(&task.id, &workspace, 1000, true)
            .await,
        100
    );
    assert_eq!(
        runtime
            .processes
            .initial_wait(&task.id, &workspace, 0, false)
            .await,
        0
    );
    let bad = runtime
        .execute_tool(
            &task.id,
            "test",
            &settings,
            "command_status",
            &json!({"process_id":"p_ffffffffffffffff"}),
            &cancel,
        )
        .await;
    let (receipt, failed) =
        runtime.record_tool_outcome(&task.id, "bad-handle", "command_status", &json!({}), bad)?;
    assert!(failed);
    assert_eq!(receipt["code"], "process_reference_unavailable");
    assert_eq!(receipt["executed"], false);
    assert_eq!(receipt["available_processes"][0]["process_id"], id);
    let other = runtime.store.enqueue_bound(
        "Other task",
        "web",
        None,
        &fenix_storage::TaskBinding {
            bound: true,
            ..Default::default()
        },
    )?;
    let bad = runtime
        .execute_tool(
            &other.id,
            "test",
            &settings,
            "command_status",
            &json!({"process_id":"p_ffffffffffffffff"}),
            &cancel,
        )
        .await;
    let (receipt, _) =
        runtime.record_tool_outcome(&other.id, "bad-handle", "command_status", &json!({}), bad)?;
    assert_eq!(receipt["available_processes"], json!([]));
    assert!(
        runtime
            .execute_tool(
                &other.id,
                "test",
                &settings,
                "command_cancel",
                &json!({"process_id":id}),
                &cancel
            )
            .await
            .is_err()
    );
    let env = runtime
        .execute_tool(
            &task.id,
            "test",
            &settings,
            "environment_inspect",
            &json!({}),
            &cancel,
        )
        .await?;
    assert_eq!(env["runtime_probe"]["exit_code"], 0);
    let cached = runtime
        .execute_tool(
            &task.id,
            "test",
            &settings,
            "environment_inspect",
            &json!({}),
            &cancel,
        )
        .await?;
    assert_eq!(cached["cached"], true);
    assert_eq!(
        cached["runtime_probe"]["process_id"],
        env["runtime_probe"]["process_id"]
    );
    std::fs::write(
        workspace.join("package.json"),
        r#"{"scripts":{"test":"node --test"}}"#,
    )?;
    let refreshed = runtime
        .execute_tool(
            &task.id,
            "test",
            &settings,
            "environment_inspect",
            &json!({}),
            &cancel,
        )
        .await?;
    assert_eq!(refreshed["cached"], false);
    assert_ne!(refreshed["fingerprint"], env["fingerprint"]);
    assert!(refreshed["manifests"].to_string().contains("node --test"));
    runtime.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn invalid_program_dependencies_do_not_execute_a_prefix() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let workspace = dir.path().join("workspace");
    std::fs::create_dir(&workspace)?;
    let runtime =
        Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    let settings = Settings::initial(workspace.clone());
    let task = runtime.store.enqueue_bound(
        "Edit",
        "web",
        None,
        &fenix_storage::TaskBinding {
            bound: true,
            ..Default::default()
        },
    )?;
    let program = json!({"steps":[{"id":"write","tool":"file_write","arguments":{"path":"should-not-exist","content":"x"}},{"id":"read","tool":"file_read","arguments":{"path":{"$ref":"future/path"}}}],"output":{"$ref":"read"}});
    assert!(
        runtime
            .tool_program(&task.id, &settings, &program, &CancellationToken::new())
            .await
            .is_err()
    );
    assert!(!workspace.join("should-not-exist").exists());
    let names = runtime
        .exposed_tools(&task.id)?
        .into_iter()
        .map(|t| t.name)
        .collect::<Vec<_>>();
    for name in [
        "files_list",
        "files_glob",
        "file_patch",
        "tool_program",
        "environment_inspect",
    ] {
        assert!(names.iter().any(|n| n == name));
    }
    runtime.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn model_file_reads_are_batched_and_receipts_keep_call_order() -> Result<()> {
    use serde_json::Value;
    use std::time::Duration;
    use wiremock::{
        Mock, MockServer, Request, ResponseTemplate,
        matchers::{method, path},
    };
    let dir = tempfile::tempdir()?;
    let workspace = dir.path().join("workspace");
    std::fs::create_dir(&workspace)?;
    std::fs::write(workspace.join("a.txt"), "alpha evidence")?;
    std::fs::write(workspace.join("b.txt"), "beta evidence")?;
    let runtime =
        Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    let server = MockServer::start().await;
    let answer = json!({"id":"done","status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":"Done"}]}]});
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(answer.clone()))
        .mount(&server)
        .await;
    let mut settings = Settings::initial(workspace);
    settings.base_url = Some(server.uri());
    runtime
        .configure(settings, Some("fake-harness-test-key".into()))
        .await?;
    server.reset().await;
    Mock::given(method("POST")).and(path("/responses")).respond_with(move |request:&Request| {
        let body:Value=serde_json::from_slice(&request.body).unwrap();
        let outputs=body["input"].as_array().unwrap().iter().filter(|v|v["type"]=="function_call_output").collect::<Vec<_>>();
        if !outputs.is_empty() {
            assert_eq!(outputs.len(),2);
            for (i,word) in ["alpha","beta"].into_iter().enumerate() {
                assert_eq!(outputs[i]["call_id"],format!("read-{i}"));
                assert!(outputs[i]["output"].as_str().unwrap().contains(word));
            }
            return ResponseTemplate::new(200).set_body_json(answer.clone());
        }
        let calls=["a.txt","b.txt"].into_iter().enumerate().map(|(i,file)|json!({"type":"function_call","call_id":format!("read-{i}"),"name":"file_read","arguments":json!({"path":file}).to_string()})).collect::<Vec<_>>();
        ResponseTemplate::new(200).set_body_json(json!({"id":"reads","status":"completed","output":calls}))
    }).mount(&server).await;
    let submitted = runtime.submit("Read both files", "web").await?;
    let id = submitted["task_id"].as_str().unwrap();
    let workers = runtime.start();
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let state = runtime.store.task(id).unwrap().status;
            if state == "completed" {
                break;
            }
            assert_ne!(state, "failed", "{:?}", runtime.store.events(id, 0, 100));
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await?;
    let events = runtime.store.events(id, 0, 100)?;
    let first_result = events
        .iter()
        .position(|e| e["kind"] == "tool_result")
        .unwrap();
    assert_eq!(
        events[..first_result]
            .iter()
            .filter(|e| e["kind"] == "tool_started")
            .count(),
        2
    );
    runtime.shutdown().await?;
    for worker in workers {
        worker.abort();
    }
    Ok(())
}

#[tokio::test]
async fn composed_patch_command_and_exact_reference_preserve_dependencies() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let workspace = dir.path().join("workspace");
    std::fs::create_dir(&workspace)?;
    let runtime =
        Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    let mut settings = Settings::initial(workspace.clone());
    settings.file_write_policy = crate::config::ActionPolicy::Allow;
    settings.command_policy = crate::config::ActionPolicy::Allow;
    let task = runtime.store.enqueue_bound(
        "Patch and verify",
        "web",
        None,
        &fenix_storage::TaskBinding {
            bound: true,
            ..Default::default()
        },
    )?;
    let program = json!({"steps":[
        {"id":"patch","tool":"file_patch","arguments":{"patch":"*** Begin Patch\n*** Add File: a.txt\n+alpha\n*** Add File: b.txt\n+beta\n*** End Patch"}},
        {"id":"verify","tool":"command_run","arguments":{"command":"test -f a.txt && test -f b.txt && printf verified","cwd":workspace,"yield_ms":0}},
        {"id":"status","tool":"command_status","arguments":{"process_id":{"$ref":"verify/process_id"}}}
    ],"output":{"$ref":"status/exit_code"}});
    let result = runtime
        .tool_program(&task.id, &settings, &program, &CancellationToken::new())
        .await?;
    assert_eq!(result["status"], "completed", "{result}");
    assert_eq!(result["output"], 0, "{result}");
    assert!(workspace.join("a.txt").exists() && workspace.join("b.txt").exists());
    runtime.shutdown().await?;
    Ok(())
}
