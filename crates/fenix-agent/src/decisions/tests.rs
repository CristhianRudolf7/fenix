use super::*;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

fn questions() -> Value {
    json!({"relation":choice("Compare",json!({"duplicate":"Same","uncertain":"Unknown"}))})
}
fn answer(confidence: f64) -> Value {
    json!({"answers":{"relation":{"type":"choice","choice":"duplicate","confidence":confidence,
        "probabilities":{"duplicate":0.95,"uncertain":0.05}}},"usage":{"input_tokens":100,"output_tokens":0}})
}
async fn fixture(
    server: &MockServer,
    shadow: bool,
) -> (tempfile::TempDir, std::sync::Arc<Runtime>) {
    let dir = tempfile::tempdir().unwrap();
    let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    rt.configure_decisions(
        DecisionSettings {
            enabled: true,
            shadow,
            endpoint: format!("{}/systemone", server.uri()),
            ..Default::default()
        },
        Some("test-decision-key".into()),
    )
    .unwrap();
    (dir, rt)
}
#[tokio::test]
async fn caches_only_identical_state_and_never_crosses_runtime_scope() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/systemone"))
        .respond_with(ResponseTemplate::new(200).set_body_json(answer(0.95)))
        .mount(&server)
        .await;
    let (_dir, rt) = fixture(&server, false).await;
    let first = rt
        .evaluate_decision(None, "memory_pair", json!({"claim":"one"}), questions())
        .await
        .unwrap();
    assert_eq!(rt.accepted_decision(&first, "relation"), Some("duplicate"));
    assert!(first["estimated_cost_usd"].as_f64().unwrap() > 0.0);
    let cached = rt
        .evaluate_decision(None, "memory_pair", json!({"claim":"one"}), questions())
        .await
        .unwrap();
    assert_eq!(cached["cached"], true);
    assert_eq!(cached["estimated_cost_usd"], 0.0);
    rt.evaluate_decision(None, "memory_pair", json!({"claim":"two"}), questions())
        .await
        .unwrap();
    let (_other_dir, other) = fixture(&server, false).await;
    other
        .evaluate_decision(None, "memory_pair", json!({"claim":"one"}), questions())
        .await
        .unwrap();
    assert_eq!(server.received_requests().await.unwrap().len(), 3);
    rt.shutdown().await.unwrap();
    other.shutdown().await.unwrap();
}
#[tokio::test]
async fn shadow_and_low_confidence_never_change_decisions() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(answer(0.7)))
        .mount(&server)
        .await;
    let (_dir, rt) = fixture(&server, false).await;
    let result = rt
        .evaluate_decision(None, "memory", json!({}), questions())
        .await
        .unwrap();
    assert!(rt.accepted_decision(&result, "relation").is_none());
    rt.configure_decisions(
        DecisionSettings {
            enabled: true,
            shadow: true,
            endpoint: format!("{}/systemone", server.uri()),
            ..Default::default()
        },
        None,
    )
    .unwrap();
    assert!(rt.accepted_decision(&answer(0.99), "relation").is_none());
    rt.shutdown().await.unwrap();
}
#[tokio::test]
async fn unavailable_provider_opens_circuit_and_preserves_fallback() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(403).set_body_json(json!({"error":"verification required"})),
        )
        .mount(&server)
        .await;
    let (_dir, rt) = fixture(&server, false).await;
    for _ in 0..2 {
        assert!(
            rt.evaluate_decision(None, "skills", json!({}), questions())
                .await
                .is_none()
        );
    }
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
    rt.shutdown().await.unwrap();
}
#[tokio::test]
async fn invalid_or_slow_provider_cannot_supply_a_decision() {
    let server = MockServer::start().await;
    let mut invalid = answer(0.99);
    invalid["answers"]["relation"]["choice"] = json!("invented");
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(invalid))
        .mount(&server)
        .await;
    let (_dir, rt) = fixture(&server, false).await;
    assert!(
        rt.evaluate_decision(None, "memory", json!({}), questions())
            .await
            .is_none()
    );
    server.reset().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_millis(500))
                .set_body_json(answer(0.99)),
        )
        .mount(&server)
        .await;
    rt.configure_decisions(
        DecisionSettings {
            enabled: true,
            shadow: false,
            timeout_ms: 100,
            endpoint: format!("{}/systemone", server.uri()),
            ..Default::default()
        },
        None,
    )
    .unwrap();
    assert!(
        rt.evaluate_decision(None, "memory", json!({}), questions())
            .await
            .is_none()
    );
    rt.shutdown().await.unwrap();
}
#[test]
fn validates_schema_and_endpoints() {
    assert!(valid_answers(&questions(), &answer(0.99)));
    let mut missing = answer(0.99);
    missing["answers"]["relation"]["probabilities"] = json!({"duplicate":0.95});
    assert!(!valid_answers(&questions(), &missing));
    assert!(
        DecisionSettings {
            endpoint: "https://example.com/steal".into(),
            ..Default::default()
        }
        .validate()
        .is_err()
    );
    assert!(
        DecisionSettings {
            minimum_confidence: f64::NAN,
            ..Default::default()
        }
        .validate()
        .is_err()
    );
}

#[tokio::test]
async fn reranking_keeps_candidates_and_never_invents_a_skill() {
    let server = MockServer::start().await;
    Mock::given(method("POST")).respond_with(ResponseTemplate::new(200).set_body_json(json!({
        "answers":{
            "c0":{"type":"choice","choice":"irrelevant","confidence":0.99,"probabilities":{"relevant":0.0,"irrelevant":1.0,"uncertain":0.0}},
            "c1":{"type":"choice","choice":"relevant","confidence":0.99,"probabilities":{"relevant":1.0,"irrelevant":0.0,"uncertain":0.0}}
        },"usage":{"input_tokens":100,"output_tokens":0}
    }))).mount(&server).await;
    let (_dir, rt) = fixture(&server, false).await;
    let task = rt
        .store
        .enqueue("Repair a Rust build", "web", None)
        .unwrap();
    let mut values = vec![
        json!({"skill":{"name":"spreadsheets"}}),
        json!({"skill":{"name":"rust"}}),
    ];
    rt.rerank_decision(&task.id, "skills", "Rust build", &mut values, "skill")
        .await;
    assert_eq!(values.len(), 2);
    assert_eq!(values[0]["skill"]["name"], "rust");
    assert_eq!(values[1]["skill"]["name"], "spreadsheets");
    rt.shutdown().await.unwrap();
}

/// Explicit opt-in only; uses an isolated runtime and never reads production state.
#[tokio::test]
#[ignore = "requires FENIX_JEV_KEY_FILE and performs billable external requests"]
async fn live_jev_decisions_and_ranking() {
    let key =
        std::fs::read_to_string(std::env::var("FENIX_JEV_KEY_FILE").expect("key file")).unwrap();
    let report_path = std::env::var("FENIX_JEV_REPORT").expect("report path");
    let dir = tempfile::tempdir().unwrap();
    let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let settings = DecisionSettings {
        enabled: true,
        shadow: false,
        ..Default::default()
    };
    rt.configure_decisions(settings.clone(), Some(key.trim().into()))
        .unwrap();
    let relations = json!({"duplicate":"Same fact","complementary":"Compatible additional facts","conflicting":"Incompatible same-property claims for the same time","temporal_change":"Explicit change over time","uncertain":"Cannot determine"});
    let routes = json!({"alpha":"Payroll software project","beta":"University admissions portal project","uncertain":"Multiple or unclear projects"});
    let completion = json!({"review":"Missing or failed independent evidence","no_extra_review":"Successful independent evidence covers requested work","uncertain":"Cannot determine"});
    let relevance = json!({"relevant":"Directly helps the request","irrelevant":"Unrelated","uncertain":"Cannot determine"});
    let cases = vec![
        (
            "duplicate",
            "Memory 1: Ana works at Acme. Memory 2: Ana is an employee of Acme.",
            relations.clone(),
            "duplicate",
        ),
        (
            "complementary",
            "Memory 1: Ana works at Acme. Memory 2: Ana prefers communicating in Spanish.",
            relations.clone(),
            "complementary",
        ),
        (
            "conflicting",
            "Record A: Project Atlas deadline is October 1 2026. Record B: Project Atlas deadline is October 15 2026. Both records describe the same unchanged deadline and validity period.",
            relations.clone(),
            "conflicting",
        ),
        (
            "temporal_change",
            "Ana worked at Acme until June 2026, then left and joined Beta in July 2026.",
            relations,
            "temporal_change",
        ),
        (
            "route_payroll",
            "User: Fix the tax calculation in the payroll software project.",
            routes.clone(),
            "alpha",
        ),
        (
            "route_multiple",
            "User: Review payroll software and then improve the university admissions portal.",
            routes,
            "uncertain",
        ),
        (
            "failed_completion",
            "Task: Fix the build. Worker claims completion. The only build command exited with code 1 and no subsequent successful verification exists.",
            completion.clone(),
            "review",
        ),
        (
            "verified_completion",
            "Task: create a file containing hello. An independent reader read the final file and verified its exact content is hello after the write succeeded.",
            completion,
            "no_extra_review",
        ),
        (
            "relevant_skill",
            "Request: repair Rust compilation errors. Candidate skill: Rust compiler diagnostics and Cargo build troubleshooting.",
            relevance.clone(),
            "relevant",
        ),
        (
            "irrelevant_skill",
            "Request: repair Rust compilation errors. Candidate skill: birthday cake decoration.",
            relevance,
            "irrelevant",
        ),
    ];
    let mut report = vec![];
    for (name, state, criteria, expected) in cases {
        // Reset circuit between independent cases so one timeout does not mask subsequent results.
        rt.configure_decisions(settings.clone(), None).unwrap();
        let start = Instant::now();
        let result = rt.evaluate_decision(None, name, json!(state), json!({"answer":choice("Classify only the provided evidence. Do not invent missing context.", criteria)})).await;
        let actual = result
            .as_ref()
            .and_then(|r| r["answers"]["answer"]["choice"].as_str());
        let accepted = result
            .as_ref()
            .and_then(|r| rt.accepted_decision(r, "answer"));
        report.push(json!({"case":name,"expected":expected,"actual":actual,"correct":actual==Some(expected),"accepted":accepted,"elapsed_ms":start.elapsed().as_millis(),"response":result}));
    }
    rt.configure_decisions(settings, None).unwrap();
    let task = rt
        .store
        .enqueue("Fix Rust compilation", "web", None)
        .unwrap();
    let original = vec![
        json!({"skill":{"name":"cake","description":"Decorate birthday cakes"}}),
        json!({"skill":{"name":"rust","description":"Fix Rust compilation errors using Cargo diagnostics"}}),
    ];
    let mut ranked = original.clone();
    let start = Instant::now();
    rt.rerank_decision(
        &task.id,
        "skills",
        "Fix Rust compilation errors",
        &mut ranked,
        "skill",
    )
    .await;
    report.push(json!({"case":"integrated_skill_ranking","correct":ranked[0]["skill"]["name"]=="rust" && ranked.len()==2,"elapsed_ms":start.elapsed().as_millis(),"events":rt.store.events(&task.id,0,100).unwrap()}));
    let mut repeated = original;
    rt.rerank_decision(
        &task.id,
        "skills",
        "Fix Rust compilation errors",
        &mut repeated,
        "skill",
    )
    .await;
    let events = rt.store.events(&task.id, 0, 100).unwrap();
    report.push(json!({"case":"integrated_cache","correct":events.last().is_some_and(|e|e["data"]["cached"]==true) && repeated==ranked,"events":events}));
    rt.configure_decisions(
        DecisionSettings {
            enabled: true,
            shadow: false,
            ..Default::default()
        },
        None,
    )
    .unwrap();
    let owner = rt
        .store
        .enqueue_bound(
            "Create report",
            "web",
            None,
            &fenix_storage::TaskBinding::default(),
        )
        .unwrap();
    let plan = rt
        .store
        .create_plan(
            &owner.id,
            "Create report",
            "",
            vec![fenix_storage::PlanNodeSpec {
                id: "report".into(),
                profile: "general".into(),
                objective: "Create report".into(),
                expected_output: "report.txt with required contents".into(),
                depends_on: vec![],
                skills: vec![],
            }],
        )
        .unwrap();
    let worker = rt
        .store
        .create_subagent(
            &owner.id,
            &fenix_storage::SubagentSpec {
                profile: "general".into(),
                objective: "Create report".into(),
                expected_output: "report.txt".into(),
                context: String::new(),
                constraints: String::new(),
            },
        )
        .unwrap();
    rt.store
        .attach_plan_worker(&plan.id, plan.version, "report", &worker.id)
        .unwrap();
    rt.flag_completion_review(
        &worker.id,
        &json!({"status":"completed","summary":"I think I wrote the file but did not check it"}),
    )
    .await;
    let flagged = rt
        .store
        .get::<bool>("decision_review_required", &worker.id)
        .unwrap()
        == Some(true);
    report.push(json!({"case":"integrated_completion_review","correct":flagged,"events":rt.store.events(&worker.id,0,100).unwrap()}));
    std::fs::write(report_path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    rt.shutdown().await.unwrap();
    let failures: Vec<_> = report
        .iter()
        .filter(|r| r["correct"] != true)
        .map(|r| r["case"].clone())
        .collect();
    assert!(
        failures.is_empty(),
        "Live failures: {failures:?}; consult report for confidence and transport results"
    );
}

#[tokio::test]
async fn diagnostic_advice_is_optional_and_cannot_authorize_or_finish_work() {
    let server = MockServer::start().await;
    Mock::given(method("POST")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"answers":{"next":{"type":"choice","choice":"repair","confidence":0.99,"probabilities":{"repair":1.0,"probe":0.0,"inspect":0.0,"uncertain":0.0}}},"usage":{"input_tokens":100,"output_tokens":20}}))).mount(&server).await;
    let (_dir, rt) = fixture(&server, false).await;
    let task = rt
        .store
        .enqueue("Repair the failing build", "web", None)
        .unwrap();
    let note = rt.diagnostic_advice(&task).await.unwrap();
    assert!(note.contains("owner's request"));
    assert_eq!(rt.store.task(&task.id).unwrap().status, "queued");
    assert!(rt.store.events(&task.id,0,100).unwrap().iter().any(|e|e["kind"]=="diagnostic_advisory" && e["data"]["authorizes_actions"]==false));
    let mut settings = rt.decision_settings().unwrap();
    settings.shadow = true;
    rt.configure_decisions(settings, None).unwrap();
    assert!(rt.diagnostic_advice(&task).await.is_none());
    rt.shutdown().await.unwrap();
}

#[tokio::test]
async fn evaluation_decision_relay_uses_placeholder_and_preserves_model() {
    let canonical = MockServer::start().await;
    let relay = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/trial/v1/jev"))
        .respond_with(ResponseTemplate::new(200).set_body_json(answer(0.99)))
        .expect(1)
        .mount(&relay)
        .await;
    let (_dir, rt) = fixture(&canonical, false).await;
    *rt.decisions.evaluation_relay.lock().unwrap() = Some(format!("{}/trial/v1/jev", relay.uri()));
    assert!(
        rt.evaluate_decision(None, "relay", json!({}), questions())
            .await
            .is_some()
    );
    assert!(canonical.received_requests().await.unwrap().is_empty());
    let calls = relay.received_requests().await.unwrap();
    let payload: Value = serde_json::from_slice(&calls[0].body).unwrap();
    assert_eq!(payload["model"], "typesafe-ai/jev");
    rt.shutdown().await.unwrap();
}
