#[path = "support/completion.rs"]
mod completion;
use fenix_agent::{Runtime, config::Settings};
use fenix_storage::TaskBinding;
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
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

fn finish(summary: &str) -> Value {
    call(
        "finish",
        "subagent_finish",
        json!({"status":"completed","summary":summary,"artifacts":"","evidence":"The fixture supplied a deterministic result.","blockers":"","review":{"verdict":"pass","findings":[],"specialists":[]}}),
    )
}

fn child_request(body: &Value) -> bool {
    body["input"].as_array().is_some_and(|input| {
        input.iter().any(|message| {
            message["role"] == "system" && message.to_string().contains("Delegated subagent task")
        })
    })
}

fn has_result(body: &Value, call_id: &str) -> bool {
    body["input"].as_array().is_some_and(|input| {
        input.iter().any(|item| {
            item["type"] == "function_call_output"
                && !item["output"]
                    .as_str()
                    .is_some_and(|s| s.contains("tool_schema_loaded"))
                && item["call_id"] == call_id
        })
    })
}

async fn fixture() -> (tempfile::TempDir, Arc<Runtime>, MockServer) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("audit-fixture.txt"),
        "A deterministic report fixture",
    )
    .unwrap();
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
        .configure(settings, Some("fake-subagent-test-key".into()))
        .await
        .unwrap();
    server.reset().await;
    let store = runtime.store.clone();
    Mock::given(method("POST"))
        .and(path("/responses"))
        .and(|request: &Request| {
            serde_json::from_slice::<Value>(&request.body)
                .ok()
                .is_some_and(|b| {
                    b["tools"]
                        .as_array()
                        .is_some_and(|ts| ts.iter().any(|t| t["name"] == "completion_audit_read"))
                })
        })
        .respond_with(move |request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            ResponseTemplate::new(200)
                .set_body_json(completion::response(&body, &store).expect("Audit fixture"))
        })
        .with_priority(1)
        .mount(&server)
        .await;
    (dir, runtime, server)
}

async fn wait_task(runtime: &Runtime, id: &str, expected: &str) {
    tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            if runtime.store.task(id).unwrap().status == expected {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap_or_else(|_| {
        panic!(
            "Expected task {id} to be {expected}, got {}",
            runtime.store.task(id).unwrap().status
        )
    });
}

async fn wait_flag(flag: &AtomicBool) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while !flag.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
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

async fn create(runtime: &Runtime, nodes: Value, extra: Value) -> (String, String) {
    let task = runtime
        .store
        .enqueue_bound(
            "Improve the monthly report without inventing requirements",
            "web",
            None,
            &TaskBinding::default(),
        )
        .unwrap();
    runtime.store.claim_next().unwrap();
    runtime
        .store
        .set_task_destination(&task.id, "telegram", Some("review-bot:42"))
        .unwrap();
    let mut args = json!({"action":"create","objective":"Prepare the report","nodes":nodes});
    args.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    let result = runtime.plan_manage(&task.id, &args).await.unwrap();
    runtime
        .store
        .set_task_status(&task.id, "completed", None)
        .unwrap();
    (task.id, result["plan"]["id"].as_str().unwrap().to_owned())
}

fn is_reviewer(body: &Value) -> bool {
    child_request(body)
        && body["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["name"] == "plan_review_read")
}

fn finding() -> Value {
    json!({"verdict":"changes_requested","findings":[{"id":"scope","category":"scope","severity":"blocking","description":"The export format has not been established","evidence":"The owner request says report but does not specify CSV","node_ids":["report"]}],"specialists":[]})
}

#[tokio::test]
async fn specialists_are_siblings_and_a_minority_objection_requires_explicit_resolution() {
    let (_dir, runtime, server) = fixture().await;
    let (parent, plan) = create(
        &runtime,
        json!([{"id":"report","objective":"Prepare the report","expected_output":"A report"}]),
        json!({"review_reason":"Separate format and compatibility questions"}),
    )
    .await;
    let store = runtime.store.clone();
    let id = plan.clone();
    Mock::given(method("POST")).and(path("/responses")).respond_with(move |r:&Request| {
        let body:Value=serde_json::from_slice(&r.body).unwrap();
        if is_reviewer(&body) {
            let input=body["input"].to_string();
            assert!(!input.contains("SENSITIVE_PEER_OPINION"));
            let report=if input.contains("specialist_1") {
                let mut report=finding();
                report["findings"][0]["description"]=json!("SENSITIVE_PEER_OPINION: the format is not confirmed");
                report
            } else if input.contains("specialist_2") { json!({"verdict":"pass","findings":[]}) }
            else { json!({"verdict":"pass","findings":[],"specialists":[{"specialty":"Data formats","question":"Check the format requirement","node_ids":["report"]},{"specialty":"Compatibility","question":"Check compatibility with the current report","node_ids":["report"]}]}) };
            return ResponseTemplate::new(200).set_body_json(reviewed(report));
        }
        if child_request(&body) { return ResponseTemplate::new(200).set_body_json(finish("Report completed")); }
        if has_result(&body,"resolve") { return ResponseTemplate::new(200).set_body_json(answer("Resolved using the owner's answer")); }
        let review=store.plan_reviews(&id).unwrap().last().unwrap().clone();
        if !has_result(&body,"format-answer") { return ResponseTemplate::new(200).set_body_json(call("format-answer","ask_user",json!({"question":"¿Confirmas que quieres el informe en CSV?"}))); }
        ResponseTemplate::new(200).set_body_json(call("resolve","plan_manage",json!({"action":"resolve_review","plan_id":id,"review_id":review.id,"expected_version":review.version,"resolutions":[{"finding_id":"specialist_1/scope","disposition":"addressed","rationale":"The owner has now chosen CSV","evidence":"Owner answer to format-answer: Sí, CSV"}]})))
    }).mount(&server).await;
    let handles = runtime.start();
    let editor = tokio::time::timeout(Duration::from_secs(12), async {
        loop {
            if let Some(id) = runtime.store.plan_reviews(&plan).unwrap()[0]
                .coordinator_id
                .clone()
                && runtime.store.task(&id).unwrap().status == "waiting"
            {
                break id;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    assert!(
        runtime.store.plan(&plan).unwrap().nodes[0]
            .worker_id
            .is_none(),
        "Two passes cannot outvote one blocking finding"
    );
    runtime
        .submit_referenced("Sí, CSV", "web", Some(&editor))
        .await
        .unwrap();
    wait_plan(&runtime, &plan, "completed").await;
    let review = runtime.store.plan_reviews(&plan).unwrap()[0].clone();
    assert_eq!(review.assignments.len(), 3);
    assert_eq!(review.resolutions.len(), 1);
    for a in review.assignments {
        assert_eq!(
            runtime
                .store
                .subagent(a.worker_id.as_deref().unwrap())
                .unwrap()
                .parent_task_id,
            parent
        );
    }
    assert!(
        runtime.store.plan_review_metrics(&plan).unwrap()[0]["reported_total_tokens"]
            .as_u64()
            .unwrap()
            > 0
    );
    stop(&runtime, handles).await;
}

fn reviewed(report: Value) -> Value {
    call(
        "review-finish",
        "subagent_finish",
        json!({"status":"completed","summary":"Independent review complete","review":report}),
    )
}

async fn wait_plan(runtime: &Runtime, id: &str, status: &str) {
    tokio::time::timeout(Duration::from_secs(12), async {
        while runtime.store.plan(id).unwrap().status != status {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap_or_else(|_| {
        panic!(
            "Plan: {:?}; reviews: {:?}",
            runtime.store.plan(id),
            runtime.store.plan_reviews(id)
        )
    });
}

#[tokio::test]
async fn complex_review_waits_for_clarification_without_blocking_chat_or_permitting_writes() {
    let (dir, runtime, server) = fixture().await;
    let checked = Arc::new(AtomicBool::new(false));
    let flag = checked.clone();
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(move |request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            let output = if is_reviewer(&body) {
                assert!(body["tools"].as_array().unwrap().iter().all(|t| !matches!(
                    t["name"].as_str(),
                    Some(
                        "command_run"
                            | "file_write"
                            | "file_edit"
                            | "subagent_spawn"
                            | "plan_manage"
                    )
                )));
                if has_result(&body, "clarify") {
                    reviewed(json!({"verdict":"pass","findings":[]}))
                } else if has_result(&body, "forbidden") {
                    let result = body["input"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|r| {
                            r["call_id"] == "forbidden" && r["type"] == "function_call_output"
                        })
                        .unwrap();
                    assert!(
                        result["output"]
                            .as_str()
                            .unwrap()
                            .contains("Plan reviewers")
                    );
                    flag.store(true, Ordering::SeqCst);
                    call(
                        "clarify",
                        "ask_user",
                        json!({"question":"¿El informe debe exportarse como CSV?"}),
                    )
                } else {
                    call(
                        "forbidden",
                        "file_write",
                        json!({"path":"reviewer-must-not-write.txt","content":"bad"}),
                    )
                }
            } else if child_request(&body) {
                finish("Milestone complete")
            } else {
                answer("The conversation remains available")
            };
            ResponseTemplate::new(200).set_body_json(output)
        })
        .mount(&server)
        .await;
    let (_parent,plan)=create(&runtime,json!([
        {"id":"sources","objective":"Read sources","expected_output":"Sources"},
        {"id":"report","objective":"Produce report","expected_output":"Report","depends_on":["sources"]}
    ]),json!({"brief":{"requirements":["Owner requested a monthly report"],"assumptions":["CSV may be appropriate; unconfirmed"]}})).await;
    assert_eq!(
        runtime.store.plan_reviews(&plan).unwrap()[0].status,
        "queued"
    );
    let handles = runtime.start();
    wait_flag(&checked).await;
    let reviewer = runtime.store.plan_reviews(&plan).unwrap()[0].assignments[0]
        .worker_id
        .clone()
        .unwrap();
    wait_task(&runtime, &reviewer, "waiting").await;
    assert!(
        runtime
            .store
            .plan(&plan)
            .unwrap()
            .nodes
            .iter()
            .all(|n| n.worker_id.is_none())
    );
    let quick = runtime.submit("Hello", "web").await.unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned();
    wait_task(&runtime, &quick, "completed").await;
    runtime
        .submit_referenced("Sí, CSV", "web", Some(&reviewer))
        .await
        .unwrap();
    wait_plan(&runtime, &plan, "completed").await;
    assert!(!dir.path().join("reviewer-must-not-write.txt").exists());
    assert_eq!(
        runtime.store.plan_reviews(&plan).unwrap()[0].status,
        "passed"
    );
    runtime.deliver_events().unwrap();
    assert!(
        !runtime
            .store
            .deliveries_for("review-bot:42")
            .unwrap()
            .is_empty()
    );
    stop(&runtime, handles).await;
}

#[tokio::test]
async fn a_simple_clear_plan_does_not_add_review_inference() {
    let (_dir, runtime, server) = fixture().await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(|r: &Request| {
            let body: Value = serde_json::from_slice(&r.body).unwrap();
            assert!(!is_reviewer(&body));
            ResponseTemplate::new(200).set_body_json(finish("Done"))
        })
        .mount(&server)
        .await;
    let (_,plan)=create(&runtime,json!([{"id":"report","objective":"Read the report title","expected_output":"The existing title"}]),json!({})).await;
    let handles = runtime.start();
    wait_plan(&runtime, &plan, "completed").await;
    assert!(runtime.store.plan_reviews(&plan).unwrap().is_empty());
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
    stop(&runtime, handles).await;
}

#[tokio::test]
async fn disagreement_requires_evidence_and_one_revision_gets_a_final_review() {
    let (_dir, runtime, server) = fixture().await;
    let (parent, plan) = create(
        &runtime,
        json!([{"id":"report","objective":"Make a report","expected_output":"CSV"}]),
        json!({"review_reason":"The output requirement may be invented"}),
    )
    .await;
    let store = runtime.store.clone();
    let id = plan.clone();
    Mock::given(method("POST")).and(path("/responses")).respond_with(move |r:&Request| {
        let body:Value=serde_json::from_slice(&r.body).unwrap();
        if is_reviewer(&body) { return ResponseTemplate::new(200).set_body_json(reviewed(finding())); }
        assert!(!child_request(&body),"Milestones must remain gated when the final review still objects");
        if has_result(&body,"fix-plan") { return ResponseTemplate::new(200).set_body_json(answer("The revised plan is awaiting its final review")); }
        let current=store.plan(&id).unwrap();
        ResponseTemplate::new(200).set_body_json(call("fix-plan","plan_manage",json!({"action":"revise","plan_id":id,"expected_version":current.version,"instruction":"Address the review finding using the original report request","resolutions":[{"finding_id":"principal/scope","disposition":"addressed","rationale":"Remove the unjustified format","evidence":"The original request only asks for a report"}],"nodes":[{"id":"report","objective":"Make a report without assuming CSV","expected_output":"A report"}]})))
    }).mount(&server).await;
    let handles = runtime.start();
    tokio::time::timeout(Duration::from_secs(12), async {
        loop {
            let reviews = runtime.store.plan_reviews(&plan).unwrap();
            if reviews.len() == 2 && reviews[1].status == "blocked" {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("{:?}", runtime.store.plan_reviews(&plan)));
    let reviews = runtime.store.plan_reviews(&plan).unwrap();
    assert_eq!(reviews[0].round, 0);
    assert_eq!(reviews[1].round, 1);
    assert!(
        runtime.store.plan(&plan).unwrap().nodes[0]
            .worker_id
            .is_none()
    );
    assert_eq!(
        runtime.store.subagents(Some(&parent), 100).unwrap().len(),
        2
    );
    assert_eq!(
        runtime
            .store
            .tasks(100)
            .unwrap()
            .iter()
            .filter(|t| t.origin == "plan_review")
            .count(),
        1
    );
    let count = server.received_requests().await.unwrap().len();
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(server.received_requests().await.unwrap().len(), count);
    stop(&runtime, handles).await;
}

#[tokio::test]
async fn a_late_review_cannot_release_a_new_plan_version() {
    let (_dir, runtime, server) = fixture().await;
    let flag = Arc::new(AtomicBool::new(false));
    let seen = flag.clone();
    let (parent, plan) = create(
        &runtime,
        json!([{"id":"report","objective":"Original draft","expected_output":"A report"}]),
        json!({"review_reason":"Check the initial scope"}),
    )
    .await;
    let store = runtime.store.clone();
    let id = plan.clone();
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(move |r: &Request| {
            let body: Value = serde_json::from_slice(&r.body).unwrap();
            if is_reviewer(&body) && store.plan(&id).unwrap().version == 1 {
                seen.store(true, Ordering::SeqCst);
                return ResponseTemplate::new(200)
                    .set_delay(Duration::from_secs(2))
                    .set_body_json(reviewed(json!({"verdict":"pass","findings":[]})));
            }
            ResponseTemplate::new(200).set_body_json(if is_reviewer(&body) {
                reviewed(json!({"verdict":"pass","findings":[]}))
            } else {
                finish("Revised work complete")
            })
        })
        .mount(&server)
        .await;
    let handles = runtime.start();
    wait_flag(&flag).await;
    let old = runtime.store.plan_reviews(&plan).unwrap()[0].clone();
    runtime.plan_manage(&parent,&json!({"action":"revise","plan_id":plan,"expected_version":1,"instruction":"Use updated scope","review_reason":"Check the changed requirement","nodes":[{"id":"report","objective":"Revised draft","expected_output":"Updated report"}]})).await.unwrap();
    wait_plan(&runtime, &plan, "completed").await;
    assert_eq!(
        runtime.store.plan_review(&old.id).unwrap().status,
        "superseded"
    );
    assert!(
        runtime
            .store
            .check_review_worker(old.assignments[0].worker_id.as_deref().unwrap())
            .is_err()
    );
    assert_eq!(
        runtime.store.plan_reviews(&plan).unwrap()[1].status,
        "passed"
    );
    stop(&runtime, handles).await;
}

#[tokio::test]
async fn an_execution_failure_after_correction_does_not_restart_the_review_loop() {
    let (_dir, runtime, server) = fixture().await;
    let (_, plan) = create(
        &runtime,
        json!([{"id":"report","objective":"Make a report","expected_output":"CSV"}]),
        json!({"review_reason":"Check the output requirement"}),
    )
    .await;
    let store = runtime.store.clone();
    let id = plan.clone();
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(move |r: &Request| {
            let body: Value = serde_json::from_slice(&r.body).unwrap();
            let current = store.plan(&id).unwrap();
            let output = if is_reviewer(&body) {
                if current.version == 1 || current.nodes[0].status == "blocked" {
                    reviewed(finding())
                } else {
                    reviewed(json!({"verdict":"pass","findings":[]}))
                }
            } else if child_request(&body) {
                call("blocked", "subagent_finish", json!({
                    "status":"blocked", "summary":"The source document is missing",
                    "blockers":"An owner-supplied document is required", "artifacts":"",
                    "evidence":"The fixture deliberately supplies no document"
                }))
            } else if has_result(&body, "fix-plan") {
                answer("The updated plan is awaiting its final check")
            } else {
                assert_eq!(current.version, 1, "No new correction turn is allowed after a final check");
                call("fix-plan", "plan_manage", json!({
                    "action":"revise", "plan_id":id, "expected_version":1,
                    "instruction":"Remove the unsupported CSV requirement",
                    "resolutions":[{"finding_id":"principal/scope", "disposition":"addressed",
                        "rationale":"Use the requested report format", "evidence":"The owner did not request CSV"}],
                    "nodes":[{"id":"report", "objective":"Make a report", "expected_output":"A report"}]
                }))
            };
            ResponseTemplate::new(200).set_body_json(output)
        })
        .mount(&server)
        .await;
    let handles = runtime.start();
    wait_plan(&runtime, &plan, "blocked").await;
    let reviews = runtime.store.plan_reviews(&plan).unwrap();
    assert_eq!(reviews.len(), 3);
    assert_eq!(reviews[1].status, "passed");
    assert_eq!(reviews[2].status, "blocked");
    assert_eq!(reviews[2].round, 1);
    assert!(reviews[2].coordinator_id.is_none());
    assert_eq!(
        runtime
            .store
            .tasks(100)
            .unwrap()
            .iter()
            .filter(|t| t.origin == "plan_review")
            .count(),
        1
    );
    stop(&runtime, handles).await;
}

#[tokio::test]
async fn security_plan_reviewer_keeps_read_only_tools_and_the_plan_report_contract() {
    let (_dir, runtime, server) = fixture().await;
    let (_parent, plan) = create(
        &runtime,
        json!([{"id":"report","objective":"Prepare a report","expected_output":"A report"}]),
        json!({"review_reason":"Check the access-control requirement"}),
    )
    .await;
    Mock::given(method("POST")).and(path("/responses"))
        .respond_with(move |request:&Request| {
            let body:Value=serde_json::from_slice(&request.body).unwrap();
            if is_reviewer(&body) {
                let input=body["input"].to_string();
                if input.contains("specialist_1") {
                    let tools=body["tools"].as_array().unwrap();
                    assert!(!tools.iter().any(|t|t["name"]=="command_run" || t["name"]=="file_write"));
                    let finish=tools.iter().find(|t|t["name"]=="subagent_finish").unwrap();
                    assert!(finish["parameters"]["properties"]["security_report"].is_null());
                    if !has_result(&body,"review-security-skill") {
                        return ResponseTemplate::new(200).set_body_json(call("review-security-skill","skill_load",json!({"name":"security-code-review"})));
                    }
                    assert!(input.contains("Security Code Review"));
                    return ResponseTemplate::new(200).set_body_json(reviewed(json!({"verdict":"pass","findings":[]})));
                }
                return ResponseTemplate::new(200).set_body_json(reviewed(json!({"verdict":"pass","findings":[],"specialists":[{"specialty":"Security","question":"Check the access-control requirement","node_ids":["report"],"skills":["security-code-review"]}]})));
            }
            if child_request(&body) { return ResponseTemplate::new(200).set_body_json(finish("Report ready")); }
            ResponseTemplate::new(200).set_body_json(answer("Plan completed"))
        }).mount(&server).await;
    let handles = runtime.start();
    wait_plan(&runtime, &plan, "completed").await;
    let review = runtime.store.plan_reviews(&plan).unwrap().remove(0);
    let specialist = review
        .assignments
        .iter()
        .find(|a| a.role == "specialist_1")
        .unwrap();
    assert_eq!(
        runtime
            .store
            .subagent(specialist.worker_id.as_deref().unwrap())
            .unwrap()
            .spec
            .profile,
        "general"
    );
    stop(&runtime, handles).await;
}
