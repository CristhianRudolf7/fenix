use super::*;
use fenix_storage::{ProjectInput, SubagentSpec, TaskBinding};

#[tokio::test]
async fn tools_preserve_scopes_and_workers_cannot_promote_their_proposals() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let runtime =
        Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    let project = runtime.store.save_project(
        None,
        &ProjectInput {
            name: "App".into(),
            ..Default::default()
        },
    )?;
    let other = runtime.store.save_project(
        None,
        &ProjectInput {
            name: "Private other project".into(),
            ..Default::default()
        },
    )?;
    let parent = runtime.store.enqueue_bound(
        "Keep sessions for seven days",
        "web",
        None,
        &TaskBinding {
            project_id: Some(project.id.clone()),
            mode: "project".into(),
            ..Default::default()
        },
    )?;
    runtime.store.set_task_status(&parent.id, "running", None)?;
    let child = runtime.store.create_subagent(
        &parent.id,
        &SubagentSpec {
            profile: "general".into(),
            objective: "Implement session persistence".into(),
            expected_output: "Updated login".into(),
            context: String::new(),
            constraints: String::new(),
        },
    )?;
    runtime.store.set_task_status(&child.id, "running", None)?;
    let cancel = CancellationToken::new();
    assert!(
        runtime
            .project_spec_tool(
                &child.id,
                &json!({"action":"get","project_id":other.id}),
                &cancel
            )
            .await
            .is_err()
    );
    let result=runtime.project_spec_tool(&child.id,&json!({"action":"propose","expected_version":0,"reason":"Explicit owner request","deltas":[{"requirement_id":"auth.session","action":"add","statement":"Sessions last seven days","source":parent.id}]}),&cancel).await?;
    let id = &result["change"]["id"];
    let apply = json!({"action":"apply","expected_version":0,"change_id":id,"reason":"Record the owner's requested behavior"});
    assert!(
        runtime
            .project_spec_tool(&child.id, &apply, &cancel)
            .await
            .is_err()
    );
    assert_eq!(
        runtime.store.project_spec(&project.id, 0, 20)?["version"],
        0
    );
    runtime
        .project_spec_tool(&parent.id, &apply, &cancel)
        .await?;
    let context = runtime.project_context_instructions(&child.id)?;
    assert!(context.contains("Project behavior contract: version 1"));
    assert!(!context.contains("Sessions last seven days"));
    let catalog = runtime.task_tool_catalog(&child.id)?;
    assert_eq!(
        catalog
            .iter()
            .find(|t| t.name == "project_spec")
            .unwrap()
            .parameters["properties"]["action"]["enum"],
        json!(["get", "changes", "propose"])
    );
    assert_eq!(runtime.store.task(&child.id)?.status, "running");
    let changes = runtime
        .project_spec_tool(&child.id, &json!({"action":"changes"}), &cancel)
        .await?;
    assert_eq!(changes["changes"][0]["status"], "applied");
    Ok(())
}

#[tokio::test]
async fn read_only_cross_project_scope_and_cancellation_cannot_change_contracts() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let runtime =
        Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    let project = runtime.store.save_project(
        None,
        &ProjectInput {
            name: "Read only".into(),
            ..Default::default()
        },
    )?;
    let task = runtime.store.enqueue_bound(
        "Summarize status",
        "web",
        None,
        &TaskBinding {
            mode: "coordination".into(),
            read_project_ids: vec![project.id.clone()],
            ..Default::default()
        },
    )?;
    runtime.store.set_task_status(&task.id, "running", None)?;
    let cancel = CancellationToken::new();
    runtime
        .project_spec_tool(
            &task.id,
            &json!({"action":"get","project_id":project.id}),
            &cancel,
        )
        .await?;
    let args = json!({"action":"propose","project_id":project.id,"expected_version":0,"reason":"Wrong authority","deltas":[{"requirement_id":"new","action":"add","statement":"New behavior","source":"Owner"}]});
    assert!(
        runtime
            .project_spec_tool(&task.id, &args, &cancel)
            .await
            .is_err()
    );
    let writer = runtime.store.enqueue_bound(
        "Set agreed behavior",
        "web",
        None,
        &TaskBinding {
            project_id: Some(project.id.clone()),
            mode: "project".into(),
            ..Default::default()
        },
    )?;
    runtime.store.set_task_status(&writer.id, "running", None)?;
    cancel.cancel();
    assert!(
        runtime
            .project_spec_tool(&writer.id, &args, &cancel)
            .await
            .is_err()
    );
    assert_eq!(
        runtime.store.project_spec_changes(&project.id, 0)?["total"],
        0
    );
    Ok(())
}
