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
