use anyhow::Result;
use fenix_storage::{ProjectInput, SpecDeltaInput, Store, SubagentSpec, TaskBinding};

fn setup() -> Result<(tempfile::TempDir, Store, String, String)> {
    let dir = tempfile::tempdir()?;
    let store = Store::open(&dir.path().join("state.db"))?;
    let project = store.save_project(
        None,
        &ProjectInput {
            name: "Example".into(),
            ..Default::default()
        },
    )?;
    let task = store.enqueue_bound(
        "Maintain agreed login behavior",
        "web",
        None,
        &TaskBinding {
            project_id: Some(project.id.clone()),
            mode: "project".into(),
            ..Default::default()
        },
    )?;
    store.set_task_status(&task.id, "running", None)?;
    Ok((dir, store, project.id, task.id))
}
fn delta(id: &str, action: &str, text: &str) -> SpecDeltaInput {
    SpecDeltaInput {
        requirement_id: id.into(),
        action: action.into(),
        statement: text.into(),
        source: "Explicit owner request in the task".into(),
    }
}
fn proposal(
    store: &Store,
    task: &str,
    project: &str,
    version: u64,
    ops: &[SpecDeltaInput],
) -> Result<String> {
    Ok(store.propose_project_spec(task,project,version,"Requested behavior change",ops)?["change"]["id"].as_str().unwrap().into())
}

#[test]
fn durable_contract_keeps_deltas_separate_and_retains_removal_history() -> Result<()> {
    let (dir, store, project, task) = setup()?;
    let added = proposal(
        &store,
        &task,
        &project,
        0,
        &[delta("auth.session", "add", "Keep sessions for seven days")],
    )?;
    assert_eq!(store.project_spec(&project, 0, 20)?["total"], 0);
    assert_eq!(
        proposal(
            &store,
            &task,
            &project,
            0,
            &[delta("auth.session", "add", "Keep sessions for seven days")]
        )?,
        added
    );
    store.resolve_project_spec(
        &task,
        &project,
        &added,
        0,
        true,
        "Owner request, recorded contract only",
    )?;
    let modified = proposal(
        &store,
        &task,
        &project,
        1,
        &[delta(
            "auth.session",
            "modify",
            "Keep sessions for fourteen days",
        )],
    )?;
    store.resolve_project_spec(&task, &project, &modified, 1, true, "Owner amendment")?;
    let removed = proposal(
        &store,
        &task,
        &project,
        2,
        &[delta("auth.session", "remove", "")],
    )?;
    store.resolve_project_spec(
        &task,
        &project,
        &removed,
        2,
        true,
        "Owner retires this requirement",
    )?;
    drop(store);
    let store = Store::open(&dir.path().join("state.db"))?;
    assert_eq!(store.project_spec(&project, 0, 20)?["version"], 3);
    assert_eq!(store.project_spec(&project, 0, 20)?["total"], 0);
    assert_eq!(
        store.project_spec_change(&project, &removed)?["change"]["deltas"][0]["before"]["statement"],
        "Keep sessions for fourteen days"
    );
    let restore = proposal(
        &store,
        &task,
        &project,
        3,
        &[delta("auth.session", "add", "Keep sessions for seven days")],
    )?;
    store.resolve_project_spec(
        &task,
        &project,
        &restore,
        3,
        true,
        "Owner restores behavior",
    )?;
    assert_eq!(
        store.project_spec(&project, 0, 20)?["requirements"][0]["revision"],
        4
    );
    // A transport retry cannot increment the contract version twice.
    store.resolve_project_spec(&task, &project, &restore, 3, true, "Retry")?;
    assert_eq!(store.project_spec(&project, 0, 20)?["version"], 4);
    Ok(())
}

#[test]
fn disjoint_proposals_survive_other_changes_but_overlap_requires_reconciliation() -> Result<()> {
    let (_dir, store, p, t) = setup()?;
    let a = proposal(
        &store,
        &t,
        &p,
        0,
        &[delta("auth.login", "add", "Allow email login")],
    )?;
    let b = proposal(
        &store,
        &t,
        &p,
        0,
        &[delta("exports.pdf", "add", "Export PDF")],
    )?;
    store.resolve_project_spec(&t, &p, &a, 0, true, "Agreed")?;
    assert!(
        store
            .resolve_project_spec(&t, &p, &b, 0, true, "Stale header")
            .is_err()
    );
    store.resolve_project_spec(
        &t,
        &p,
        &b,
        1,
        true,
        "Reread current contract; unrelated amendment",
    )?;
    let c = proposal(
        &store,
        &t,
        &p,
        2,
        &[delta("auth.login", "modify", "Use passwordless login")],
    )?;
    let d = proposal(&store, &t, &p, 2, &[delta("auth.login", "remove", "")])?;
    assert_eq!(
        store.project_spec_change(&p, &c)?["conflicts"][0]["kind"],
        "overlapping_proposal"
    );
    assert!(
        store
            .resolve_project_spec(&t, &p, &c, 2, true, "No silent winner")
            .is_err()
    );
    assert!(store.check_task_spec_changes(&t).is_err());
    store.resolve_project_spec(&t, &p, &d, 2, false, "Owner retains login")?;
    store.resolve_project_spec(&t, &p, &c, 2, true, "Owner chooses passwordless")?;
    assert_eq!(store.project_spec(&p, 0, 20)?["version"], 3);
    assert!(
        store
            .resolve_project_spec(&t, &p, &d, 3, true, "Cannot revive")
            .is_err()
    );
    Ok(())
}

#[test]
fn invalid_batch_is_atomic_and_writes_cannot_cross_project_or_worker_authority() -> Result<()> {
    let (_dir, store, p, t) = setup()?;
    assert!(
        proposal(
            &store,
            &t,
            &p,
            0,
            &[
                delta("valid", "add", "Present"),
                delta("missing", "modify", "Invalid")
            ]
        )
        .is_err()
    );
    assert_eq!(store.project_spec_changes(&p, 0)?["total"], 0);
    let other = store.save_project(
        None,
        &ProjectInput {
            name: "Other".into(),
            ..Default::default()
        },
    )?;
    assert!(
        proposal(
            &store,
            &t,
            &other.id,
            0,
            &[delta("test", "add", "Wrong scope")]
        )
        .is_err()
    );
    let child = store.create_subagent(
        &t,
        &SubagentSpec {
            profile: "general".into(),
            objective: "Change login".into(),
            expected_output: "Result".into(),
            context: String::new(),
            constraints: String::new(),
        },
    )?;
    store.set_task_status(&child.id, "running", None)?;
    let id = proposal(
        &store,
        &child.id,
        &p,
        0,
        &[delta("auth.login", "add", "Email login")],
    )?;
    assert!(
        store
            .resolve_project_spec(&child.id, &p, &id, 0, true, "Self approve")
            .is_err()
    );
    assert!(store.project_spec_change(&other.id, &id).is_err());
    store.resolve_project_spec(
        &t,
        &p,
        &id,
        0,
        true,
        "Master records the explicit owner request",
    )?;
    assert!(store.check_task_spec_changes(&child.id).is_ok());
    let edited = proposal(
        &store,
        &t,
        &p,
        1,
        &[delta("auth.login", "modify", "Use passkeys instead")],
    )?;
    store.resolve_project_spec(&t, &p, &edited, 1, true, "New owner request")?;
    assert!(store.check_task_spec_changes(&child.id).is_err());
    Ok(())
}

#[test]
fn concurrent_consolidation_has_one_winner_and_the_other_can_reread() -> Result<()> {
    let (dir, store, p, t) = setup()?;
    let a = proposal(&store, &t, &p, 0, &[delta("a", "add", "A")])?;
    let b = proposal(&store, &t, &p, 0, &[delta("b", "add", "B")])?;
    let db = dir.path().join("state.db");
    let other = Store::open(&db)?;
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let gate = barrier.clone();
    let pt = p.clone();
    let tt = t.clone();
    let bb = b.clone();
    let worker = std::thread::spawn(move || {
        gate.wait();
        other.resolve_project_spec(&tt, &pt, &bb, 0, true, "Agreed")
    });
    barrier.wait();
    let first = store.resolve_project_spec(&t, &p, &a, 0, true, "Agreed");
    let second = worker.join().unwrap();
    assert_ne!(first.is_ok(), second.is_ok());
    let remaining = if first.is_ok() { &b } else { &a };
    store.resolve_project_spec(&t, &p, remaining, 1, true, "Reread unrelated update")?;
    assert_eq!(store.project_spec(&p, 0, 1)?["next_offset"], 1);
    assert_eq!(store.project_spec(&p, 1, 1)?["requirements"][0]["id"], "b");
    Ok(())
}
