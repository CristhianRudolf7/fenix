use super::*;
use fenix_storage::{ProjectInput, TaskBinding};

#[tokio::test]
async fn noninteractive_clarification_requires_an_indispensable_dependency() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    let task = rt
        .store
        .enqueue("Create a data export", "evaluation", None)?;
    let error = rt
        .check_clarification_policy(&task.id, &json!({"question":"Which optional sort order?"}))
        .unwrap_err();
    assert!(
        !error
            .to_string()
            .contains("noninteractive_input_unavailable:")
    );
    rt.record_tool_outcome(&task.id, "optional", "ask_user", &json!({}), Err(error))?;
    assert!(rt.validate_task_stop(&task.id,&json!({"status":"blocked","stop_reason":"owner_input_unavailable","blocker_call_id":"optional"})).is_err());
    let error=rt.check_clarification_policy(&task.id,&json!({"question":"Which source account?","blocking_input":"owner_decision","source_quote":"Create a data export","blocking_reason":"The request identifies no source account and two authorized accounts contain different owner data; choosing one changes the requested export."})).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("noninteractive_input_unavailable:")
    );
    rt.record_tool_outcome(&task.id, "essential", "ask_user", &json!({}), Err(error))?;
    assert!(rt.validate_task_stop(&task.id,&json!({"status":"blocked","stop_reason":"owner_input_unavailable","blocker_call_id":"essential"}))?.is_none());
    let interactive = rt.store.enqueue("Improve my page", "web", None)?;
    assert!(
        rt.check_clarification_policy(
            &interactive.id,
            &json!({"question":"Which outcome matters?"})
        )
        .is_ok()
    );
    assert!(rt.pending.lock().await.is_empty());
    assert!(
        rt.store
            .get::<Vec<Value>>("task_grants", &task.id)?
            .is_none()
    );
    rt.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn owner_scopes_inherit_without_weakening_requirements_or_permissions() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    let project = rt.store.save_project(
        None,
        &ProjectInput {
            name: "Research".into(),
            ..Default::default()
        },
    )?;
    let global = rt
        .store
        .enqueue("De ahora en adelante hazlo todo tú solo", "web", None)?;
    let args = |mode: &str, scope: &str, quote: &str| json!({"action":"set","mode":mode,"scope":scope,"source_quote":quote});
    rt.manage_autonomy(&global.id, &args("autonomous", "global", &global.request))?;
    let project_task = rt.store.enqueue_bound(
        "Para este proyecto consúltame antes de decidir",
        "telegram",
        None,
        &TaskBinding {
            bound: true,
            project_id: Some(project.id.clone()),
            mode: "project".into(),
            ..Default::default()
        },
    )?;
    rt.manage_autonomy(
        &project_task.id,
        &args("assisted", "project", &project_task.request),
    )?;
    let task = rt.store.enqueue_bound(
        "Hazlo todo tú solo. Run the tests",
        "web",
        None,
        &rt.store.task_binding(&project_task.id)?,
    )?;
    assert_eq!(rt.autonomy_context(&task.id)?["mode"], "assisted");
    assert!(
        rt.manage_autonomy(
            &task.id,
            &args("autonomous", "global", "Hazlo todo tú solo")
        )
        .is_err()
    );
    rt.manage_autonomy(&task.id, &args("autonomous", "task", "Hazlo todo tú solo"))?;
    assert_eq!(rt.autonomy_context(&task.id)?["mode"], "autonomous");
    assert!(rt.check_question_policy(&task.id, "requirement").is_ok());
    assert!(
        rt.check_question_policy(&task.id, "implementation")
            .is_err()
    );
    assert!(rt.validate_task_completion(&task.id).is_err());
    assert!(
        rt.store
            .get::<Vec<Value>>("task_grants", &task.id)?
            .is_none()
    );
    assert_eq!(rt.autonomy_context(&project_task.id)?["mode"], "assisted");
    let unrelated = rt.store.enqueue("Write a note", "web", None)?;
    assert_eq!(rt.autonomy_context(&unrelated.id)?["mode"], "autonomous");
    let worker = rt.store.create_subagent(
        &task.id,
        &fenix_storage::SubagentSpec {
            profile: "general".into(),
            objective: "Inspect code".into(),
            expected_output: "Findings".into(),
            context: String::new(),
            constraints: String::new(),
        },
    )?;
    assert_eq!(rt.autonomy_context(&worker.id)?["mode"], "autonomous");
    assert!(
        rt.manage_autonomy(&worker.id, &args("assisted", "global", "Always ask me"))
            .is_err()
    );
    rt.store.put(
        "task_updates",
        &task.id,
        &vec!["Ahora consúltame antes de decidir".to_owned()],
    )?;
    rt.manage_autonomy(
        &task.id,
        &args("assisted", "task", "Ahora consúltame antes de decidir"),
    )?;
    assert_eq!(rt.autonomy_context(&worker.id)?["mode"], "assisted");
    assert!(rt.check_question_policy(&task.id, "implementation").is_ok());
    // Preferences are durable, not process-local state.
    assert_eq!(
        rt.store
            .get::<Preference>("autonomy", &format!("task:{}", task.id))?
            .unwrap()
            .mode,
        "assisted"
    );
    rt.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn benchmark_questions_do_not_wait_or_grant_permission() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    let task = rt.store.enqueue("Build the package", "evaluation", None)?;
    let worker = rt.store.create_subagent(
        &task.id,
        &fenix_storage::SubagentSpec {
            profile: "general".into(),
            objective: "Build".into(),
            expected_output: "Binary".into(),
            context: String::new(),
            constraints: String::new(),
        },
    )?;
    assert_eq!(rt.autonomy_context(&worker.id)?["interactive"], false);
    assert!(
        rt.check_question_policy(&task.id, "implementation")
            .unwrap_err()
            .to_string()
            .contains("continue authorized work")
    );
    let error = rt
        .check_question_policy(&task.id, "requirement")
        .unwrap_err();
    rt.record_tool_outcome(
        &task.id,
        "missing-owner",
        "ask_user",
        &json!({"question":"Which target?"}),
        Err(error),
    )?;
    assert!(rt.pending.lock().await.is_empty());
    assert!(rt.validate_task_stop(&task.id,&json!({"status":"blocked","stop_reason":"owner_input_unavailable","blocker_call_id":"missing-owner"})).is_err());
    let error = rt
        .ask(
            &task.id,
            "approval",
            "May I write?",
            None,
            &tokio_util::sync::CancellationToken::new(),
        )
        .await
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("noninteractive_input_unavailable")
    );
    assert!(rt.pending.lock().await.is_empty());
    assert!(
        rt.store
            .get::<Vec<Value>>("task_grants", &task.id)?
            .is_none()
    );
    rt.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn receipt_reuse_never_satisfies_missing_or_stale_checks() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    let task = rt.store.enqueue("Run the tests", "web", None)?;
    assert!(rt.completion_receipt_evidence(&task.id).is_err());
    rt.record_tool_outcome(
        &task.id,
        "write",
        "file_write",
        &json!({"path":"module.py"}),
        Ok(json!({"path":"module.py","verified":true})),
    )?;
    assert!(rt.completion_receipt_evidence(&task.id)?.contains("write"));
    assert!(rt.validate_task_completion(&task.id).is_err());
    rt.record_tool_outcome(
        &task.id,
        "test",
        "command_run",
        &json!({"command":"python -m pytest tests","cwd":"/app"}),
        Ok(json!({"exit_code":0,"output":"3 passed in 0.01s"})),
    )?;
    assert!(rt.validate_task_completion(&task.id).is_ok());
    rt.record_tool_outcome(
        &task.id,
        "edit",
        "file_edit",
        &json!({"path":"module.py"}),
        Ok(json!({"path":"module.py","verified":true})),
    )?;
    assert!(rt.validate_task_completion(&task.id).is_err());
    assert!(rt.completion_receipt_evidence(&task.id)?.contains("edit"));
    rt.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn autonomous_intake_still_asks_and_records_the_owner_requirement() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    let task = rt
        .store
        .enqueue("Hazlo todo tú solo. Mejora mi página", "web", None)?;
    rt.manage_autonomy(&task.id,&json!({"action":"set","mode":"autonomous","scope":"task","source_quote":"Hazlo todo tú solo"}))?;
    let pending_rt = rt.clone();
    let pending_task = task.id.clone();
    let handle = tokio::spawn(async move {
        pending_rt.check_question_policy(&pending_task, "requirement")?;
        pending_rt
            .ask(
                &pending_task,
                "clarification",
                "¿Quieres mejorar el diseño o la velocidad?",
                None,
                &tokio_util::sync::CancellationToken::new(),
            )
            .await
    });
    let id = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            if let Some(id) = rt.pending.lock().await.keys().next().cloned() {
                break id;
            }
            tokio::task::yield_now().await;
        }
    })
    .await?;
    rt.answer_pending(&id, "La velocidad, conserva el diseño", None, "web", None)
        .await?;
    assert_eq!(handle.await??, "La velocidad, conserva el diseño");
    let contract = rt.task_contract(&task.id)?;
    assert_eq!(
        contract["clarifications"][0]["answer"],
        "La velocidad, conserva el diseño"
    );
    assert!(rt.pending.lock().await.is_empty());
    rt.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn file_merge_routes_to_data_workflow_without_model_output_skill() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    let result = rt.search_skills("Merge JSON CSV Parquet users, normalize schema, deduplicate by source priority and report conflicts",3)?;
    let names = result["skills"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["skill"]["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(names.contains(&"data-pipelines"), "{names:?}");
    assert_ne!(names.first(), Some(&"structured-llm-output"));
    rt.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn technical_and_self_contradictory_questions_do_not_create_blockers() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    let task = rt
        .store
        .enqueue("Implement the supplied interface", "evaluation", None)?;
    for (index, args) in [
        json!({"question":"Which API?", "blocking_reason":"No blocking question is needed; inspect the available interface."}),
        json!({"question":"What is this format?", "blocking_input":"technical_information", "source_quote":"Implement the supplied interface", "blocking_reason":"The index is not in the workspace; I have not inspected public format metadata."}),
        json!({"question":"Which account?", "blocking_input":"owner_decision", "source_quote":"Invented owner request", "blocking_reason":"Two different accounts would give different requested results."}),
    ].into_iter().enumerate() {
        let error = rt.check_clarification_policy(&task.id, &args).unwrap_err();
        assert!(!error.to_string().contains("noninteractive_input_unavailable:"));
        let call = format!("question-{index}");
        rt.record_tool_outcome(&task.id, &call, "ask_user", &args, Err(error))?;
        let outcome = rt.validate_task_stop(&task.id, &json!({"status":"blocked","stop_reason":"owner_input_unavailable","blocker_call_id":call}));
        // A stagnation pause is permitted; accepting the claimed owner dependency is not.
        assert!(!matches!(outcome, Ok(None)));
    }
    let interactive = rt.store.enqueue("Improve my page", "web", None)?;
    assert!(
        rt.check_clarification_policy(
            &interactive.id,
            &json!({"question":"Which intended outcome?"})
        )
        .is_ok()
    );
    rt.shutdown().await?;
    Ok(())
}
