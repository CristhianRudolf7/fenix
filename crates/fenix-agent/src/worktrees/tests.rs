use super::*;
use fenix_storage::{SubagentSpec, TaskBinding};

#[tokio::test]
async fn requirement_conflicts_block_git_integration_even_for_different_files() -> Result<()> {
    let (_dir, runtime, repo, _parent) = fixture().await?;
    let project = runtime.store.save_project(
        None,
        &fenix_storage::ProjectInput {
            name: "Shared login".into(),
            workspace: Some(repo.to_string_lossy().into()),
            ..Default::default()
        },
    )?;
    let parent = runtime.store.enqueue_bound(
        "Implement login",
        "web",
        None,
        &TaskBinding {
            project_id: Some(project.id.clone()),
            mode: "project".into(),
            ..Default::default()
        },
    )?;
    runtime.store.set_task_status(&parent.id, "running", None)?;
    let a = child(&runtime, &parent.id, "Implement password login")?;
    let b = child(&runtime, &parent.id, "Implement passkey login")?;
    runtime.prepare_worktree(&a, &repo).await?;
    runtime.prepare_worktree(&b, &repo).await?;
    std::fs::write(
        runtime.task_workspace(&a, &repo)?.join("password.txt"),
        "password login",
    )?;
    std::fs::write(
        runtime.task_workspace(&b, &repo)?.join("passkey.txt"),
        "passkey login",
    )?;
    let cancel = CancellationToken::new();
    let mut ids = Vec::new();
    for (task, text) in [(&a, "Use passwords"), (&b, "Use passkeys")] {
        let result=runtime.project_spec_tool(task,&json!({"action":"propose","expected_version":0,"reason":"Candidate interpretation; ask owner to reconcile","deltas":[{"requirement_id":"auth.method","action":"add","statement":text,"source":parent.id}]}),&cancel).await?;
        ids.push(result["change"]["id"].as_str().unwrap().to_owned());
    }
    let before = git(&repo, &["rev-parse", "HEAD"]).await?;
    let error = runtime
        .integrate_worktree(&a, &cancel)
        .await
        .unwrap_err()
        .to_string();
    assert!(error.contains("requirement conflict"), "{error}");
    assert_eq!(git(&repo, &["rev-parse", "HEAD"]).await?, before);
    assert!(!repo.join("password.txt").exists());
    runtime.project_spec_tool(&parent.id,&json!({"action":"discard","change_id":ids[1],"expected_version":0,"reason":"Owner clarified: passwords"}),&cancel).await?;
    runtime.integrate_worktree(&a, &cancel).await?;
    runtime.project_spec_tool(&parent.id,&json!({"action":"apply","change_id":ids[0],"expected_version":0,"reason":"Owner clarified: passwords; implementation remains subject to its existing checks"}),&cancel).await?;
    assert!(runtime.integrate_worktree(&b, &cancel).await.is_err());
    assert!(!repo.join("passkey.txt").exists());
    assert!(
        runtime
            .task_workspace(&b, &repo)?
            .join("passkey.txt")
            .exists()
    );
    Ok(())
}

async fn fixture() -> Result<(tempfile::TempDir, Arc<Runtime>, PathBuf, String)> {
    let dir = tempfile::tempdir()?;
    let repo = dir.path().join("repo");
    std::fs::create_dir(&repo)?;
    git(&repo, &["init", "-b", "main"]).await?;
    std::fs::write(repo.join("shared.txt"), "original\n")?;
    git(&repo, &["add", "."]).await?;
    git(&repo, &["commit", "-m", "Start fixture"]).await?;
    let runtime =
        Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    let parent = runtime.store.enqueue_bound(
        "Parallel implementation",
        "web",
        None,
        &TaskBinding {
            workspace: Some(repo.to_string_lossy().into()),
            ..Default::default()
        },
    )?;
    runtime.store.set_task_status(&parent.id, "running", None)?;
    Ok((dir, runtime, repo, parent.id))
}
fn child(runtime: &Runtime, parent: &str, objective: &str) -> Result<String> {
    let job = runtime.store.create_subagent(
        parent,
        &SubagentSpec {
            profile: "general".into(),
            objective: objective.into(),
            expected_output: "Implement the assigned change".into(),
            context: String::new(),
            constraints: String::new(),
        },
    )?;
    runtime.store.set_task_status(&job.id, "running", None)?;
    Ok(job.id)
}

#[tokio::test]
async fn parallel_worktrees_merge_independent_changes_and_retry_idempotently() -> Result<()> {
    let (_dir, runtime, repo, parent) = fixture().await?;
    let a = child(&runtime, &parent, "Implement A")?;
    let b = child(&runtime, &parent, "Implement B")?;
    let (ra, rb) = tokio::join!(
        runtime.prepare_worktree(&a, &repo),
        runtime.prepare_worktree(&b, &repo)
    );
    ra?;
    rb?;
    let wa = runtime.task_workspace(&a, &repo)?;
    let wb = runtime.task_workspace(&b, &repo)?;
    assert_ne!(wa, wb);
    runtime.files_execute(&wa, "file_write", &json!({"path":"a.txt","content":"A"}))?;
    runtime.files_execute(&wb, "file_write", &json!({"path":"b.txt","content":"B"}))?;
    assert!(!repo.join("a.txt").exists());
    assert!(!wa.join("b.txt").exists());
    assert!(
        runtime
            .files_execute(
                &wa,
                "file_write",
                &json!({"path":wb.join("bad.txt"),"content":"bad"})
            )
            .is_err()
    );
    let stop = CancellationToken::new();
    let (ra, rb) = tokio::join!(
        runtime.integrate_worktree(&a, &stop),
        runtime.integrate_worktree(&b, &stop)
    );
    ra?;
    rb?;
    assert_eq!(std::fs::read_to_string(repo.join("a.txt"))?, "A");
    assert_eq!(std::fs::read_to_string(repo.join("b.txt"))?, "B");
    let head = git(&repo, &["rev-parse", "HEAD"]).await?;
    runtime.integrate_worktree(&a, &stop).await?;
    assert_eq!(head, git(&repo, &["rev-parse", "HEAD"]).await?);
    assert!(wa.exists() && wb.exists());
    Ok(())
}

#[tokio::test]
async fn conflicting_changes_preserve_both_branches_and_source_checkout() -> Result<()> {
    let (_dir, runtime, repo, parent) = fixture().await?;
    let a = child(&runtime, &parent, "First implementation")?;
    let b = child(&runtime, &parent, "Second implementation")?;
    runtime.prepare_worktree(&a, &repo).await?;
    runtime.prepare_worktree(&b, &repo).await?;
    std::fs::write(
        runtime.task_workspace(&a, &repo)?.join("shared.txt"),
        "first\n",
    )?;
    std::fs::write(
        runtime.task_workspace(&b, &repo)?.join("shared.txt"),
        "second\n",
    )?;
    let stop = CancellationToken::new();
    runtime.integrate_worktree(&a, &stop).await?;
    let head = git(&repo, &["rev-parse", "HEAD"]).await?;
    let report = runtime
        .finish_worktree(&b, json!({"status":"completed","summary":"Done"}), &stop)
        .await;
    assert_eq!(report["status"], "blocked");
    assert_eq!(runtime.worktree(&b)?.unwrap().status, "conflict");
    assert_eq!(git(&repo, &["rev-parse", "HEAD"]).await?, head);
    assert_eq!(std::fs::read_to_string(repo.join("shared.txt"))?, "first\n");
    assert_eq!(
        std::fs::read_to_string(runtime.task_workspace(&b, &repo)?.join("shared.txt"))?,
        "second\n"
    );
    clean(&repo).await?;
    Ok(())
}

#[tokio::test]
async fn dirty_source_and_changed_target_are_never_overwritten() -> Result<()> {
    let (_dir, runtime, repo, parent) = fixture().await?;
    let a = child(&runtime, &parent, "Implement safely")?;
    std::fs::write(repo.join("user.txt"), "user draft")?;
    assert!(runtime.prepare_worktree(&a, &repo).await.is_err());
    assert!(runtime.worktree(&a)?.is_none());
    git(&repo, &["add", "."]).await?;
    git(&repo, &["commit", "-m", "Save user draft"]).await?;
    runtime.prepare_worktree(&a, &repo).await?;
    std::fs::write(
        runtime.task_workspace(&a, &repo)?.join("result.txt"),
        "result",
    )?;
    std::fs::write(repo.join("user.txt"), "new user draft")?;
    let stop = CancellationToken::new();
    assert!(runtime.integrate_worktree(&a, &stop).await.is_err());
    assert_eq!(
        std::fs::read_to_string(repo.join("user.txt"))?,
        "new user draft"
    );
    assert!(!repo.join("result.txt").exists());
    git(&repo, &["add", "."]).await?;
    git(&repo, &["commit", "-m", "Save next draft"]).await?;
    git(&repo, &["switch", "-c", "other"]).await?;
    assert!(runtime.integrate_worktree(&a, &stop).await.is_err());
    assert!(!repo.join("result.txt").exists());
    git(&repo, &["switch", "main"]).await?;
    git(&repo, &["reset", "--hard", "HEAD~2"]).await?;
    assert!(runtime.integrate_worktree(&a, &stop).await.is_err());
    assert!(!repo.join("result.txt").exists());
    Ok(())
}

#[tokio::test]
async fn isolated_commands_and_file_writes_do_not_share_effect_lock() -> Result<()> {
    let (_dir, runtime, repo, parent) = fixture().await?;
    let a = child(&runtime, &parent, "Parallel A")?;
    let b = child(&runtime, &parent, "Parallel B")?;
    runtime.prepare_worktree(&a, &repo).await?;
    runtime.prepare_worktree(&b, &repo).await?;
    let wa = runtime.task_workspace(&a, &repo)?;
    let wb = runtime.task_workspace(&b, &repo)?;
    let stop = CancellationToken::new();
    let held = runtime
        .effect_lease_for(&a, Some(&[wa.clone()]), &stop)
        .await?;
    let second = tokio::time::timeout(
        std::time::Duration::from_secs(1),
        runtime.effect_lease_for(&b, Some(&[wb]), &stop),
    )
    .await??;
    assert!(
        tokio::time::timeout(
            std::time::Duration::from_millis(30),
            runtime.effect_lease_for(&a, Some(&[wa]), &stop)
        )
        .await
        .is_err()
    );
    drop(held);
    drop(second);
    Ok(())
}

#[tokio::test]
async fn assignment_survives_reopening_runtime_and_cancellation_preserves_edits() -> Result<()> {
    let (dir, runtime, repo, parent) = fixture().await?;
    let a = child(&runtime, &parent, "Persist implementation")?;
    runtime.prepare_worktree(&a, &repo).await?;
    let path = runtime.task_workspace(&a, &repo)?;
    std::fs::write(path.join("draft.txt"), "keep me")?;
    drop(runtime);
    let runtime =
        Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    runtime.prepare_worktree(&a, &repo).await?;
    assert_eq!(runtime.task_workspace(&a, &repo)?, path);
    let cancelled = CancellationToken::new();
    cancelled.cancel();
    assert!(runtime.integrate_worktree(&a, &cancelled).await.is_err());
    assert_eq!(std::fs::read_to_string(path.join("draft.txt"))?, "keep me");
    assert!(!repo.join("draft.txt").exists());
    Ok(())
}

#[tokio::test]
async fn non_git_directories_keep_existing_behavior_and_nested_scopes_do_not_expand() -> Result<()>
{
    let (dir, runtime, repo, _parent) = fixture().await?;
    let plain = dir.path().join("plain");
    let nested = repo.join("nested");
    std::fs::create_dir(&plain)?;
    std::fs::create_dir(&nested)?;
    for (path, expected) in [(&plain, true), (&nested, false)] {
        let parent = runtime.store.enqueue_bound(
            "Scoped task",
            "web",
            None,
            &TaskBinding {
                workspace: Some(path.to_string_lossy().into()),
                ..Default::default()
            },
        )?;
        let id = child(&runtime, &parent.id, "Inspect this directory")?;
        assert_eq!(runtime.prepare_worktree(&id, &repo).await.is_ok(), expected);
        assert!(runtime.worktree(&id)?.is_none());
    }
    Ok(())
}
#[tokio::test]
async fn managed_worker_loop_writes_and_integrates_before_publishing_completion() -> Result<()> {
    use wiremock::{
        Mock, MockServer, Request, ResponseTemplate,
        matchers::{method, path},
    };
    let (_dir, runtime, repo, parent) = fixture().await?;
    let server = MockServer::start().await;
    let answer = json!({"id":"setup","status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":"OK"}]}],"usage":{"input_tokens":1,"output_tokens":1,"total_tokens":2}});
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(answer))
        .mount(&server)
        .await;
    let mut settings = crate::config::Settings::initial(repo.clone());
    settings.base_url = Some(server.uri());
    settings.file_write_policy = ActionPolicy::Allow;
    runtime.store.set_task_status(&parent, "completed", None)?;
    runtime
        .configure(settings, Some("worktree-fixture-key".into()))
        .await?;
    runtime.store.set_task_status(&parent, "running", None)?;
    server.reset().await;
    Mock::given(method("POST")).and(path("/responses")).respond_with(|request:&Request| {
            let body:Value=serde_json::from_slice(&request.body).unwrap();
            let text=body["input"].to_string();
            assert!(text.contains("Isolated Git workspace"));
            let written=body["input"].as_array().unwrap().iter().any(|v| v["type"]=="function_call_output" && v["call_id"]=="write" && v["output"].as_str().is_some_and(|o|o.contains("verified")));
            let name=if text.contains("Implement A") {"a.txt"} else {"b.txt"};
            let (tool,args)=if written {("subagent_finish",json!({"status":"completed","summary":"Implemented the file","artifacts":name,"evidence":"File tool verified write","blockers":""}))} else {("file_write",json!({"path":name,"content":"implemented"}))};
            ResponseTemplate::new(200).set_body_json(json!({"id":"step","status":"completed","output":[{"type":"function_call","call_id":if written {"finish"} else {"write"},"name":tool,"arguments":args.to_string()}],"usage":{"input_tokens":2,"output_tokens":2,"total_tokens":4}}))
        }).mount(&server).await;
    let a = child(&runtime, &parent, "Implement A")?;
    let b = child(&runtime, &parent, "Implement B")?;
    for id in [&a, &b] {
        runtime.store.set_task_status(id, "queued", None)?;
    }
    let worker = tokio::spawn(runtime.clone().subagent_worker());
    let finished = tokio::time::timeout(std::time::Duration::from_secs(15), async {
        loop {
            let jobs = [
                runtime.store.subagent(&a).unwrap(),
                runtime.store.subagent(&b).unwrap(),
            ];
            if jobs
                .iter()
                .all(|j| matches!(j.status.as_str(), "completed" | "failed" | "blocked"))
            {
                break jobs;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await;
    runtime.stop.cancel();
    worker.await?;
    let jobs = finished?;
    for job in jobs {
        assert_eq!(job.status, "completed", "{:?}", job.result);
        assert_eq!(job.result.unwrap()["worktree"]["status"], "integrated");
    }
    assert_eq!(std::fs::read_to_string(repo.join("a.txt"))?, "implemented");
    assert_eq!(std::fs::read_to_string(repo.join("b.txt"))?, "implemented");
    Ok(())
}

#[tokio::test]
async fn refuses_repository_defined_programs_without_executing_them() -> Result<()> {
    let (_dir, runtime, repo, parent) = fixture().await?;
    let a = child(&runtime, &parent, "Do not run repository programs")?;
    git(&repo, &["config", "filter.bad.clean", "touch executed"]).await?;
    assert!(runtime.prepare_worktree(&a, &repo).await.is_err());
    assert!(!repo.join("executed").exists());
    assert!(runtime.worktree(&a)?.is_none());
    Ok(())
}
#[tokio::test]
async fn commands_really_run_in_parallel_and_cannot_select_another_checkout() -> Result<()> {
    let (_dir, runtime, repo, parent) = fixture().await?;
    let a = child(&runtime, &parent, "Run parallel command A")?;
    let b = child(&runtime, &parent, "Run parallel command B")?;
    runtime.prepare_worktree(&a, &repo).await?;
    runtime.prepare_worktree(&b, &repo).await?;
    let settings = crate::config::Settings::initial(repo.clone());
    let stop = CancellationToken::new();
    let command = json!({"command":"printf ready; sleep 3; printf done", "cwd":".","timeout_seconds":10,"yield_ms":0});
    let first = runtime
        .execute_tool(&a, "test", &settings, "command_run", &command, &stop)
        .await?;
    let second = runtime
        .execute_tool(&b, "test", &settings, "command_run", &command, &stop)
        .await?;
    assert_eq!(first["status"], "running");
    assert_eq!(second["status"], "running");
    assert_eq!(
        runtime
            .processes
            .list()
            .await
            .iter()
            .filter(|p| p.status == "running")
            .count(),
        2
    );
    runtime.settle_task(&a).await;
    runtime.settle_task(&b).await;
    let escape = json!({"command":"touch escaped", "cwd":repo,"timeout_seconds":10});
    assert!(
        runtime
            .execute_tool(&a, "test", &settings, "command_run", &escape, &stop)
            .await
            .is_err()
    );
    assert!(!repo.join("escaped").exists());
    Ok(())
}

#[tokio::test]
async fn superseded_plan_worker_cannot_integrate_changes() -> Result<()> {
    let (_dir, runtime, repo, parent) = fixture().await?;
    let node = fenix_storage::PlanNodeSpec {
        id: "code".into(),
        profile: "general".into(),
        objective: "Implement feature".into(),
        expected_output: "Working code".into(),
        depends_on: vec![],
        skills: vec![],
    };
    let plan = runtime
        .store
        .create_plan(&parent, "Implement feature", "", vec![node])?;
    let a = child(&runtime, &parent, "Implement planned feature")?;
    runtime.store.set_task_status(&a, "queued", None)?;
    runtime
        .store
        .attach_plan_worker(&plan.id, plan.version, "code", &a)?;
    runtime.prepare_worktree(&a, &repo).await?;
    let wa = runtime.task_workspace(&a, &repo)?;
    std::fs::write(wa.join("stale.txt"), "old work")?;
    runtime.store.cancel_plan(&plan.id, plan.version, &parent)?;
    assert!(
        runtime
            .integrate_worktree(&a, &CancellationToken::new())
            .await
            .is_err()
    );
    assert!(!repo.join("stale.txt").exists());
    assert!(wa.join("stale.txt").exists());
    Ok(())
}

#[tokio::test]
async fn integration_approval_denial_preserves_source_and_worker_files() -> Result<()> {
    let (_dir, runtime, repo, parent) = fixture().await?;
    let id = child(&runtime, &parent, "Prepare approved change")?;
    runtime.prepare_worktree(&id, &repo).await?;
    let work = runtime.task_workspace(&id, &repo)?;
    std::fs::write(work.join("approval.txt"), "pending")?;
    runtime.store.put(
        "settings",
        "active",
        &crate::config::Settings::initial(repo.clone()),
    )?;
    let background = runtime.clone();
    let task = id.clone();
    let finishing = tokio::spawn(async move {
        background
            .finish_worktree(
                &task,
                json!({"status":"completed","summary":"File prepared"}),
                &CancellationToken::new(),
            )
            .await
    });
    let pending = tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop {
            let mut requests = runtime.pending.lock().await;
            if let Some(key) = requests.keys().next().cloned() {
                break requests.remove(&key).unwrap();
            }
            drop(requests);
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await?;
    assert_eq!(pending.view.kind, "approval");
    assert!(!repo.join("approval.txt").exists());
    pending.reply.send("reject".into()).unwrap();
    let report = finishing.await?;
    assert_eq!(report["status"], "blocked");
    assert!(!repo.join("approval.txt").exists());
    assert_eq!(
        std::fs::read_to_string(work.join("approval.txt"))?,
        "pending"
    );
    Ok(())
}

#[tokio::test]
async fn tenant_scope_and_symlinked_storage_cannot_escape_workspace_roots() -> Result<()> {
    let (dir, mut runtime, repo, parent) = fixture().await?;
    let other = tempfile::tempdir()?;
    std::os::unix::fs::symlink(other.path(), dir.path().join(".fenix-worktrees"))?;
    let id = child(&runtime, &parent, "Isolate inside the authorized owner")?;
    assert!(runtime.prepare_worktree(&id, &repo).await.is_err());
    assert_eq!(std::fs::read_dir(other.path())?.count(), 0);
    std::fs::remove_file(dir.path().join(".fenix-worktrees"))?;
    let tenant = dir.path().join("tenant");
    std::fs::create_dir_all(tenant.join("projects"))?;
    Arc::get_mut(&mut runtime).unwrap().tenant_root = Some(tenant);
    assert!(runtime.prepare_worktree(&id, &repo).await.is_err());
    assert!(runtime.worktree(&id)?.is_none());
    Ok(())
}

#[tokio::test]
async fn master_can_retry_a_blocked_standalone_integration_without_reexecuting_worker() -> Result<()>
{
    let (_dir, runtime, repo, parent) = fixture().await?;
    let id = child(&runtime, &parent, "Finish a blocked integration")?;
    runtime.prepare_worktree(&id, &repo).await?;
    std::fs::write(
        runtime.task_workspace(&id, &repo)?.join("result.txt"),
        "ready",
    )?;
    std::fs::write(repo.join("user.txt"), "draft")?;
    let stop = CancellationToken::new();
    let report = runtime
        .finish_worktree(
            &id,
            json!({"status":"completed","summary":"Implementation done"}),
            &stop,
        )
        .await;
    assert_eq!(report["status"], "blocked");
    runtime.store.finish_subagent(&id, "blocked", &report)?;
    git(&repo, &["add", "."]).await?;
    git(&repo, &["commit", "-m", "Preserve user changes"]).await?;
    let settings = crate::config::Settings::initial(repo.clone());
    let result = runtime
        .execute_tool(
            &parent,
            "test",
            &settings,
            "worktree_manage",
            &json!({"action":"integrate","subagent_id":id}),
            &stop,
        )
        .await?;
    assert_eq!(result["status"], "integrated");
    assert_eq!(runtime.store.subagent(&id)?.status, "completed");
    assert_eq!(std::fs::read_to_string(repo.join("result.txt"))?, "ready");
    assert_eq!(std::fs::read_to_string(repo.join("user.txt"))?, "draft");
    // A follow-up/reopened job cannot be overwritten by a late integration receipt.
    runtime.store.save_subagent_context(
        &id,
        &[fenix_core::Message::user(
            "Recorded completed worker context",
        )],
        0,
    )?;
    runtime
        .store
        .send_subagent_input(&parent, &id, "Continue with a new change")?;
    assert!(
        runtime
            .store
            .complete_subagent_integration(&id, &report, &result)
            .is_err()
    );
    assert_eq!(runtime.store.task(&id)?.status, "queued");
    Ok(())
}

#[tokio::test]
async fn waiting_for_source_effects_does_not_block_replanning_or_cancellation() -> Result<()> {
    let (_dir, runtime, repo, parent) = fixture().await?;
    let id = child(&runtime, &parent, "Wait for integration resources")?;
    runtime.prepare_worktree(&id, &repo).await?;
    let source_lock = runtime.effects.clone().lock_owned().await;
    let cancel = CancellationToken::new();
    let child_cancel = cancel.clone();
    let worker = runtime.clone();
    let target = id.clone();
    let pending =
        tokio::spawn(async move { worker.integrate_worktree(&target, &child_cancel).await });
    tokio::time::sleep(std::time::Duration::from_millis(30)).await;
    let planner =
        tokio::time::timeout(std::time::Duration::from_secs(1), runtime.plan_lock.lock()).await?;
    cancel.cancel();
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(1), pending)
            .await??
            .is_err()
    );
    drop(planner);
    drop(source_lock);
    assert_eq!(runtime.worktree(&id)?.unwrap().status, "active");
    Ok(())
}
