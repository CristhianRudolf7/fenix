use super::*;
use fenix_storage::{PlanNodeSpec, PlanRevision, SubagentSpec, TaskBinding};
use tokio_util::sync::CancellationToken;

fn task(rt: &Runtime, request: &str) -> Task {
    let task = rt
        .store
        .enqueue_bound(
            request,
            "web",
            None,
            &TaskBinding {
                bound: true,
                ..Default::default()
            },
        )
        .unwrap();
    rt.store.set_task_status(&task.id, "running", None).unwrap();
    rt.store.task(&task.id).unwrap()
}
fn node(id: &str) -> PlanNodeSpec {
    PlanNodeSpec {
        profile: "general".into(),
        id: id.into(),
        objective: format!("Implement {id}"),
        expected_output: format!("Verified {id}"),
        depends_on: vec![],
        skills: vec![],
    }
}
fn child(rt: &Runtime, parent: &Task, objective: &str) -> fenix_storage::Subagent {
    rt.store
        .create_subagent(
            &parent.id,
            &SubagentSpec {
                profile: "general".into(),
                objective: objective.into(),
                expected_output: "Working artifact".into(),
                context: String::new(),
                constraints: String::new(),
            },
        )
        .unwrap()
}
fn failure(rt: &Runtime, task: &str, call: &str, args: Value) {
    let mut contract = rt.task_contract(task).unwrap();
    rt.remember_failed_approach(
        task,
        call,
        "command_run",
        &args,
        &json!({"exit_code":1,"output":"missing dependency"}),
        false,
        &mut contract,
    )
    .unwrap();
    rt.store.put("task_contracts", task, &contract).unwrap();
}
#[test]
fn distinguishes_execution_failures_from_pending_and_control_receipts() {
    assert!(failed(&json!({"exit_code":2}), false));
    assert!(failed(&json!({"status":"timed_out"}), false));
    assert!(failed(
        &json!({"executed":false,"error":"bad arguments"}),
        true
    ));
    assert!(!failed(&json!({"denied":true,"executed":false}), false));
    assert!(!failed(
        &json!({"status":"running","exit_code":null}),
        false
    ));
    assert!(!failed(
        &json!({"executed":false,"status":"tool_schema_loaded"}),
        false
    ));
    assert!(!failed(&json!({"exit_code":0}), false));
    assert!(failed(
        &json!({"status":"completed","process_id":"p","exit_code":null}),
        false
    ));
}
#[tokio::test]
async fn failures_are_bounded_redacted_deduplicated_and_survive_reopen() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    let task = task(&rt, "Repair software");
    assert!(rt.operational_context(&task)?.is_none());
    rt.redactor.add("secret-for-test");
    let args = json!({"command":"build secret-for-test"});
    failure(&rt, &task.id, "first", args.clone());
    failure(&rt, &task.id, "second", args.clone());
    let mut contract = rt.task_contract(&task.id)?;
    assert_eq!(contract["failed_approaches"].as_array().unwrap().len(), 1);
    assert_eq!(contract["failed_approaches"][0]["observations"], 2);
    assert_eq!(contract["failed_approaches"][0]["cause"], "not_established");
    assert!(!contract.to_string().contains("secret-for-test"));
    rt.remember_failed_approach(
        &task.id,
        "pending",
        "command_run",
        &args,
        &json!({"status":"running"}),
        false,
        &mut contract,
    )?;
    assert!(
        contract["failed_approaches"][0]
            .get("later_success_call_id")
            .is_none()
    );
    rt.remember_failed_approach(
        &task.id,
        "success",
        "command_run",
        &args,
        &json!({"exit_code":0}),
        false,
        &mut contract,
    )?;
    assert_eq!(
        contract["failed_approaches"][0]["later_success_call_id"],
        "success"
    );
    rt.store.put("task_contracts", &task.id, &contract)?;
    let first = rt.operational_context(&task)?.unwrap().text_content();
    assert_eq!(
        first,
        rt.operational_context(&task)?.unwrap().text_content()
    );
    assert!(first.len() < 12_500);
    for i in 0..12 {
        failure(
            &rt,
            &task.id,
            &format!("failure-{i}"),
            json!({"command":format!("build {i}")}),
        );
    }
    assert_eq!(
        rt.task_contract(&task.id)?["failed_approaches"]
            .as_array()
            .unwrap()
            .len(),
        8
    );
    rt.shutdown().await?;
    drop(rt);
    let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    assert!(
        rt.operational_context(&task)?
            .unwrap()
            .text_content()
            .contains("failure-11")
    );
    rt.shutdown().await?;
    Ok(())
}
#[tokio::test]
async fn plan_revision_and_owner_answers_invalidate_old_next_steps() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    let task = task(&rt, "Build project");
    let plan = rt.store.create_plan(
        &task.id,
        "Current goal",
        "Preserve API",
        vec![node("build")],
    )?;
    let settings = crate::config::Settings::initial(dir.path().into());
    let annotate = json!({"action":"annotate","kind":"next_step","text":"UNIQUE_NEXT_ACTION","source":"owner request"});
    rt.execute_tool(
        &task.id,
        &task.id,
        &settings,
        "task_contract",
        &annotate,
        &CancellationToken::new(),
    )
    .await?;
    let old = rt.operational_context(&task)?.unwrap().text_content();
    assert!(old.contains("UNIQUE_NEXT_ACTION"));
    rt.contract_answer(&task.id, "Which API?", "Keep the existing API")?;
    assert!(
        !rt.operational_context(&task)?
            .unwrap()
            .text_content()
            .contains("UNIQUE_NEXT_ACTION")
    );
    rt.execute_tool(
        &task.id,
        &task.id,
        &settings,
        "task_contract",
        &annotate,
        &CancellationToken::new(),
    )
    .await?;
    assert!(
        rt.operational_context(&task)?
            .unwrap()
            .text_content()
            .contains("UNIQUE_NEXT_ACTION")
    );
    let (revised, _) = rt.store.revise_plan(
        &plan.id,
        plan.version,
        &task.id,
        PlanRevision {
            objective: "Changed goal",
            constraints: "New constraint",
            nodes: vec![node("test")],
            reason: "Owner change",
        },
    )?;
    rt.store.activate_plan_revision(&plan.id, revised.version)?;
    let latest = rt.operational_context(&task)?.unwrap().text_content();
    assert!(latest.contains("Changed goal"));
    assert!(latest.contains("New constraint"));
    assert!(!latest.contains("UNIQUE_NEXT_ACTION"));
    assert_ne!(old, latest);
    rt.shutdown().await?;
    Ok(())
}
#[tokio::test]
async fn new_generation_inherits_prior_failures_but_not_independent_siblings() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
    let parent = task(&rt, "Build project");
    let plan = rt.store.create_plan(
        &parent.id,
        "Goal",
        "",
        vec![node("main"), node("independent")],
    )?;
    let old = child(&rt, &parent, "Implement main milestone");
    let sibling = child(&rt, &parent, "Implement independent milestone");
    rt.store
        .attach_plan_worker(&plan.id, plan.version, "main", &old.id)?;
    rt.store
        .attach_plan_worker(&plan.id, plan.version, "independent", &sibling.id)?;
    failure(
        &rt,
        &old.id,
        "old-failure",
        json!({"command":"RELEVANT_PRIOR_ATTEMPT"}),
    );
    failure(
        &rt,
        &sibling.id,
        "sibling-failure",
        json!({"command":"UNRELATED_PRIVATE_ATTEMPT"}),
    );
    let mut changed = node("main");
    changed.objective = "Correct previous attempt".into();
    let (revised, _) = rt.store.revise_plan(
        &plan.id,
        plan.version,
        &parent.id,
        PlanRevision {
            objective: "Goal",
            constraints: "",
            nodes: vec![changed, node("independent")],
            reason: "Repair main milestone",
        },
    )?;
    rt.store.activate_plan_revision(&plan.id, revised.version)?;
    let next = child(&rt, &parent, "Repair main milestone with new evidence");
    rt.store
        .attach_plan_worker(&plan.id, revised.version, "main", &next.id)?;
    let context = rt
        .operational_context(&rt.store.task(&next.id)?)?
        .unwrap()
        .text_content();
    assert!(context.contains("RELEVANT_PRIOR_ATTEMPT"));
    assert!(!context.contains("UNRELATED_PRIVATE_ATTEMPT"));
    assert!(!context.contains("Implement independent"));
    let separate = task(&rt, "Unrelated task");
    assert!(rt.operational_context(&separate)?.is_none());
    rt.shutdown().await?;
    Ok(())
}
