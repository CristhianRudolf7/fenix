use crate::{
    Runtime,
    config::{ActionPolicy, Settings},
};
use anyhow::Result;
use fenix_core::{Message, ToolDefinition};
use fenix_credentials::ApiKey;
use fenix_storage::{Task, TaskBinding};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

async fn fixture() -> Result<(tempfile::TempDir, Arc<Runtime>, Settings, Task)> {
    let dir = tempfile::tempdir()?;
    let workspace = dir.path().join("workspace");
    std::fs::create_dir(&workspace)?;
    let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    let mut settings = Settings::initial(workspace.clone());
    settings.file_write_policy = ActionPolicy::Allow;
    settings.command_policy = ActionPolicy::Allow;
    let task = rt.store.enqueue_bound(
        "Implement this task",
        "web",
        None,
        &TaskBinding {
            bound: true,
            workspace: Some(workspace.to_string_lossy().into_owned()),
            ..Default::default()
        },
    )?;
    rt.store.set_task_status(&task.id, "running", None)?;
    Ok((dir, rt, settings, task))
}
#[tokio::test]
async fn usage_anchor_invalidates_on_early_edit_tool_change_and_missing_usage() -> Result<()> {
    let (_dir, rt, settings, task) = fixture().await?;
    let model = rt.model_connection(settings, ApiKey::new("fake-test-key")?)?;
    let mut messages = vec![Message::user("old"), Message::user("same last")];
    rt.anchor_usage(&task.id, &model, &messages, &[], Some(100))?;
    assert_eq!(rt.context_tokens(&task.id, &model, &messages, &[])?, 101); // empty delta envelope estimate
    messages.push(Message::user("new data"));
    assert!(rt.context_tokens(&task.id, &model, &messages, &[])? > 100);
    messages[0] = Message::user("edited");
    assert_eq!(
        rt.context_tokens(&task.id, &model, &messages, &[])?,
        crate::active_context::estimated_tokens(&messages, &[])
    );
    rt.anchor_usage(&task.id, &model, &messages, &[], Some(1000))?;
    let tools = vec![ToolDefinition {
        name: "changed".into(),
        description: "new schema".into(),
        parameters: json!({}),
    }];
    assert_eq!(
        rt.context_tokens(&task.id, &model, &messages, &tools)?,
        crate::active_context::estimated_tokens(&messages, &tools)
    );
    rt.anchor_usage(&task.id, &model, &messages, &[], None)?;
    assert_eq!(
        rt.context_tokens(&task.id, &model, &messages, &[])?,
        crate::active_context::estimated_tokens(&messages, &[])
    );
    rt.shutdown().await?;
    Ok(())
}
#[tokio::test]
async fn tool_program_foreach_projection_and_original_receipts() -> Result<()> {
    let (_dir, rt, settings, task) = fixture().await?;
    for (name, text) in [("a.txt", "alpha"), ("b.txt", "beta")] {
        std::fs::write(settings.workspace.join(name), text)?;
    }
    let program = json!({"steps":[{"id":"read","tool":"file_read","arguments":{"path":{"$ref":"item"}},"foreach":["a.txt","b.txt"],"select":{"$ref":"read/content"}}],"output":{"$ref":"read"}});
    let result = rt
        .execute_tool(
            &task.id,
            "test",
            &settings,
            "tool_program",
            &program,
            &CancellationToken::new(),
        )
        .await?;
    assert_eq!(result["status"], "completed");
    assert_eq!(result["calls"], 2);
    assert!(result["output"].to_string().contains("alpha"));
    let receipts = rt
        .store
        .events(&task.id, 0, 100)?
        .into_iter()
        .filter(|e| e["kind"] == "tool_result")
        .collect::<Vec<_>>();
    assert_eq!(receipts.len(), 2);
    assert_eq!(receipts[0]["data"]["tool"], "file_read");
    rt.shutdown().await?;
    Ok(())
}
#[tokio::test]
async fn program_stops_after_error_and_honors_persisted_denials() -> Result<()> {
    let (_dir, rt, settings, task) = fixture().await?;
    let action = json!({"tool":"file_write","arguments":{"path":"denied.txt","content":"no"}});
    rt.store
        .put("task_denials", &task.id, &vec![action.clone()])?;
    let result=rt.tool_program(&task.id,&settings,&json!({"steps":[{"id":"first","tool":"file_write","arguments":action["arguments"]},{"id":"second","tool":"file_write","arguments":{"path":"later.txt","content":"no"}}],"output":null}),&CancellationToken::new()).await?;
    assert_eq!(result["status"], "failed");
    assert_eq!(result["calls"], 1);
    assert!(!settings.workspace.join("denied.txt").exists());
    assert!(!settings.workspace.join("later.txt").exists());
    rt.shutdown().await?;
    Ok(())
}
#[tokio::test]
async fn history_refuses_newer_user_edits_and_restores_exact_binary_bytes() -> Result<()> {
    let (_dir, rt, settings, task) = fixture().await?;
    let path = settings.workspace.join("data.bin");
    let original = vec![0, 1, 255, 2];
    std::fs::write(&path, &original)?;
    let snapshot = rt.history_begin(&task.id, &settings.workspace, &["data.bin".into()], "test")?;
    std::fs::write(&path, [9, 8, 7])?;
    rt.history_finish(snapshot)?;
    std::fs::write(&path, b"user edit")?;
    assert!(
        rt.file_history(
            &task.id,
            &settings,
            &json!({"action":"restore"}),
            &CancellationToken::new()
        )
        .await
        .is_err()
    );
    assert_eq!(std::fs::read(&path)?, b"user edit");
    std::fs::write(&path, [9, 8, 7])?;
    let result = rt
        .file_history(
            &task.id,
            &settings,
            &json!({"action":"restore"}),
            &CancellationToken::new(),
        )
        .await?;
    assert_eq!(result["restored"].as_array().unwrap().len(), 1);
    assert_eq!(std::fs::read(path)?, original);
    rt.shutdown().await?;
    Ok(())
}
#[tokio::test]
async fn history_is_automatic_for_native_writes_and_can_undo_creation() -> Result<()> {
    let (_dir, rt, settings, task) = fixture().await?;
    let cancel = CancellationToken::new();
    let first = rt
        .execute_tool(
            &task.id,
            "test",
            &settings,
            "file_write",
            &json!({"path":"new.txt","content":"hello"}),
            &cancel,
        )
        .await?;
    assert_eq!(first["verified"], true);
    let versions = rt
        .file_history(&task.id, &settings, &json!({"action":"list"}), &cancel)
        .await?;
    assert_eq!(versions["total"], 1);
    rt.file_history(&task.id, &settings, &json!({"action":"restore"}), &cancel)
        .await?;
    assert!(!settings.workspace.join("new.txt").exists());
    rt.shutdown().await?;
    Ok(())
}
#[tokio::test]
async fn history_preflights_all_files_and_rejects_incomplete_versions() -> Result<()> {
    let (_dir, rt, settings, task) = fixture().await?;
    for n in ["a", "b"] {
        std::fs::write(settings.workspace.join(n), "old")?;
    }
    let snapshots = rt.history_begin(
        &task.id,
        &settings.workspace,
        &["a".into(), "b".into()],
        "test",
    )?;
    for n in ["a", "b"] {
        std::fs::write(settings.workspace.join(n), "agent")?;
    }
    rt.history_finish(snapshots)?;
    std::fs::write(settings.workspace.join("b"), "owner")?;
    assert!(
        rt.file_history(
            &task.id,
            &settings,
            &json!({"action":"restore"}),
            &CancellationToken::new()
        )
        .await
        .is_err()
    );
    assert_eq!(
        std::fs::read_to_string(settings.workspace.join("a"))?,
        "agent"
    );
    let pending = rt.history_begin(&task.id, &settings.workspace, &["c".into()], "pending")?;
    assert!(
        rt.file_history(
            &task.id,
            &settings,
            &json!({"action":"restore","id":pending[0].id}),
            &CancellationToken::new()
        )
        .await
        .is_err()
    );
    rt.shutdown().await?;
    Ok(())
}
#[tokio::test]
async fn restart_admits_only_complete_boundaries_and_preserves_counters() -> Result<()> {
    let (_dir, rt, _settings, task) = fixture().await?;
    rt.store.put("execution_steps", &task.id, &7)?;
    rt.recovery_boundary(&task.id, &[Message::user("original context")], true)?;
    let unsafe_task = rt.store.enqueue_bound(
        "uncertain command",
        "web",
        None,
        &TaskBinding {
            bound: true,
            ..Default::default()
        },
    )?;
    rt.store.set_task_status(&unsafe_task.id, "running", None)?;
    rt.recovery_boundary(&unsafe_task.id, &[], false)?;
    let (_, recovered) = rt.recover_restart()?;
    assert_eq!(recovered, 1);
    assert_eq!(rt.store.task(&task.id)?.status, "queued");
    assert_eq!(rt.store.task(&unsafe_task.id)?.status, "interrupted");
    assert_eq!(rt.store.get::<usize>("execution_steps", &task.id)?, Some(7));
    let messages = rt.recovered_messages(&task.id)?.unwrap();
    assert_eq!(messages[0].text_content(), "original context");
    assert!(
        messages
            .last()
            .unwrap()
            .text_content()
            .contains("do not replay")
    );
    rt.shutdown().await?;
    Ok(())
}
#[tokio::test]
async fn restart_caps_repeated_crashes_and_does_not_resume_waiting_approval() -> Result<()> {
    let (_dir, rt, _settings, task) = fixture().await?;
    rt.recovery_boundary(&task.id, &[Message::user("safe")], true)?;
    rt.store.put("restart_attempts", &task.id, &3)?;
    let other = rt.store.enqueue("waiting", "web", None)?;
    rt.recovery_boundary(&other.id, &[Message::user("safe")], true)?;
    rt.store.set_task_status(&other.id, "waiting", None)?;
    let (_, recovered) = rt.recover_restart()?;
    assert_eq!(recovered, 0);
    assert_eq!(rt.store.task(&other.id)?.status, "interrupted");
    rt.shutdown().await?;
    Ok(())
}
#[tokio::test]
async fn simple_tasks_do_not_queue_learning_and_complex_tasks_deduplicate() -> Result<()> {
    let (_dir, rt, _settings, task) = fixture().await?;
    rt.store.set_task_status(&task.id, "completed", None)?;
    rt.queue_skill_learning(&task.id)?;
    assert!(rt.store.get::<Value>("skill_learning", &task.id)?.is_none());
    for index in 0..8 {
        rt.store.event(&task.id,"tool_result",&json!({"call_id":format!("read-{index}"),"tool":"file_read","result":{"content":"useful"},"is_error":false}))?;
    }
    rt.queue_skill_learning(&task.id)?;
    rt.queue_skill_learning(&task.id)?;
    assert_eq!(rt.store.records::<Value>("skill_learning")?.len(), 1);
    assert_eq!(
        rt.store.get::<Value>("skill_learning", &task.id)?.unwrap()["status"],
        "queued"
    );
    rt.shutdown().await?;
    Ok(())
}
#[tokio::test]
async fn history_never_attributes_an_external_write_to_the_agent() -> Result<()> {
    let (_dir, rt, settings, task) = fixture().await?;
    let path = settings.workspace.join("race.txt");
    std::fs::write(&path, "old")?;
    let snapshots = rt.history_begin(
        &task.id,
        &settings.workspace,
        &["race.txt".into()],
        "file_edit",
    )?;
    let receipt = Ok(
        json!({"path":path,"before_sha256":fenix_documents::sha256(b"old"),"sha256":fenix_documents::sha256(b"agent")}),
    );
    std::fs::write(&path, "later owner edit")?;
    rt.history_finish_receipt(snapshots, &receipt)?;
    assert!(
        rt.file_history(
            &task.id,
            &settings,
            &json!({"action":"restore"}),
            &CancellationToken::new()
        )
        .await
        .is_err()
    );
    assert_eq!(std::fs::read_to_string(path)?, "later owner edit");
    rt.shutdown().await?;
    Ok(())
}
#[tokio::test]
async fn programs_surface_unseen_repository_guidance_before_writing() -> Result<()> {
    let (_dir, rt, settings, task) = fixture().await?;
    std::fs::write(settings.workspace.join("AGENTS.md"), "Use clear names.")?;
    let result=rt.tool_program(&task.id,&settings,&json!({"steps":[{"id":"write","tool":"file_write","arguments":{"path":"code.txt","content":"data"}}],"output":null}),&CancellationToken::new()).await?;
    assert_eq!(result["status"], "paused");
    assert!(result["repository_instructions"].is_array());
    assert!(!settings.workspace.join("code.txt").exists());
    rt.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn reopening_runtime_preserves_a_safe_task_identity_and_journal() -> Result<()> {
    let (dir, rt, _settings, task) = fixture().await?;
    let session = rt.store.session_id().to_owned();
    rt.store.event(
        &task.id,
        "completed_effect",
        &json!({"id":"original-effect"}),
    )?;
    rt.recovery_boundary(
        &task.id,
        &[Message::user("Continue after recorded effect")],
        true,
    )?;
    rt.shutdown().await?;
    drop(rt);
    let reopened =
        Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    assert_eq!(reopened.store.session_id(), session);
    assert_eq!(reopened.store.task(&task.id)?.status, "queued");
    assert!(reopened.recovered_messages(&task.id)?.is_some());
    assert_eq!(
        reopened
            .store
            .events(&task.id, 0, 100)?
            .iter()
            .filter(|e| e["kind"] == "completed_effect")
            .count(),
        1
    );
    reopened.shutdown().await?;
    Ok(())
}
#[tokio::test]
async fn unresolved_write_errors_prevent_automatic_restart() -> Result<()> {
    let (_dir, rt, _settings, task) = fixture().await?;
    rt.record_tool_outcome(
        &task.id,
        "external",
        "connector_call",
        &json!({}),
        Err(anyhow::anyhow!("Response lost after request")),
    )?;
    rt.recovery_boundary(&task.id, &[Message::user("inspect outcome")], true)?;
    let (_, recovered) = rt.recover_restart()?;
    assert_eq!(recovered, 0);
    assert_eq!(rt.store.task(&task.id)?.status, "interrupted");
    rt.shutdown().await?;
    Ok(())
}
