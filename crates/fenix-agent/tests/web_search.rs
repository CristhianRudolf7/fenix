use fenix_agent::{Runtime, config::Settings, tools};
use serde_json::{Value, json};
use std::time::Duration;
use wiremock::{
    Mock, MockServer, Request, ResponseTemplate,
    matchers::{method, path},
};

fn answer(text: &str) -> Value {
    json!({"id":"response","status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":text}]}]})
}

#[test]
fn both_agent_roles_receive_validated_public_web_tools() {
    for delegated in [false, true] {
        let catalog = tools::catalog_for(delegated);
        for name in ["web_search", "web_read"] {
            assert!(catalog.iter().any(|t| t.name == name));
        }
        assert!(tools::validate_for(delegated, "web_search", &json!({"query":"science"})).is_ok());
        assert!(
            tools::validate_for(
                delegated,
                "web_search",
                &json!({"query":"science","limit":-1})
            )
            .is_err()
        );
        assert!(
            tools::validate_for(
                delegated,
                "web_search",
                &json!({"query":"science","provider":"brave"})
            )
            .is_err()
        );
        assert!(
            tools::validate_for(
                delegated,
                "web_read",
                &json!({"url":"https://example.com/","cookies":"secret"})
            )
            .is_err()
        );
    }
}

#[tokio::test]
async fn agent_dispatches_query_only_search_and_receives_honest_empty_coverage() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("data")).unwrap();
    std::fs::write(dir.path().join("data/web-search.json"), r#"{"seeds":[]}"#).unwrap();
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
        .configure(settings, Some("fake-web-test-key".into()))
        .await
        .unwrap();
    server.reset().await;
    Mock::given(method("POST")).and(path("/responses"))
        .respond_with(|request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            let input = body["input"].as_array().unwrap();
            if let Some(result) = input.iter().find(|v| v["type"] == "function_call_output" && v["call_id"] == "search-call") {
                let output = result["output"].as_str().unwrap();
                assert!(output.contains("local_fts5"));
                assert!(output.contains("No matching indexed pages"));
                assert!(output.contains("indexed_pages"));
                let value: Value = serde_json::from_str(output).unwrap();
                assert_eq!(value["discovery"]["enabled"], true);
                return ResponseTemplate::new(200).set_body_json(answer("The local index has no matching evidence yet."));
            }
            if !input.iter().any(|v| v["type"] == "function_call_output" && v["call_id"] == "discover-search") {
                assert!(body["tools"].as_array().unwrap().iter().any(|v| v["name"] == "tool_search"));
                assert!(!body["tools"].as_array().unwrap().iter().any(|v| v["name"] == "web_search"));
                return ResponseTemplate::new(200).set_body_json(json!({"id":"discovery","status":"completed","output":[{
                    "type":"function_call","call_id":"discover-search","name":"tool_search","arguments":"{\"query\":\"web_search\"}"}]}));
            }
            assert!(body["tools"].as_array().unwrap().iter().any(|v| v["name"] == "web_search"));
            ResponseTemplate::new(200).set_body_json(json!({"id":"tool-response","status":"completed","output":[{
                "type":"function_call","call_id":"search-call","name":"web_search","arguments":"{\"query\":\"science\"}"}]}))
        }).mount(&server).await;
    let workers = runtime.start();
    let submitted = runtime
        .submit("Search for public science information", "web")
        .await
        .unwrap();
    let task_id = submitted["task_id"].as_str().unwrap();
    tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            let task = runtime.store.task(task_id).unwrap();
            if task.status == "completed" {
                break;
            }
            assert!(
                !matches!(task.status.as_str(), "failed" | "waiting"),
                "unexpected task status: {}",
                task.status
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        runtime.web.status().await.unwrap().coverage.requests_today,
        0
    );
    assert!(
        runtime
            .store
            .context(0)
            .unwrap()
            .iter()
            .any(|m| m.text.contains("no matching evidence"))
    );
    runtime.shutdown().await.unwrap();
    for worker in workers {
        tokio::time::timeout(Duration::from_secs(5), worker)
            .await
            .unwrap()
            .unwrap();
    }
}

#[tokio::test]
async fn one_model_response_dispatches_ten_web_calls_as_a_batch_and_records_each_once() {
    verify_web_batch(false).await;
}

#[tokio::test]
async fn web_batch_preserves_validation_and_prior_denials_without_aborting_siblings() {
    verify_web_batch(true).await;
}

async fn verify_web_batch(with_rejections: bool) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("data")).unwrap();
    std::fs::write(dir.path().join("data/web-search.json"), r#"{"seeds":[]}"#).unwrap();
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
        .configure(settings, Some("fake-batch-test-key".into()))
        .await
        .unwrap();
    server.reset().await;
    Mock::given(method("POST")).and(path("/responses"))
        .respond_with(move |request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            let outputs: Vec<_> = body["input"].as_array().unwrap().iter().filter(|v|v["type"]=="function_call_output").collect();
            if !outputs.is_empty() {
                assert_eq!(outputs.len(),10);
                for (i,output) in outputs.iter().enumerate() {
                    assert_eq!(output["call_id"],format!("batch-{i}"));
                    let result: Value = serde_json::from_str(output["output"].as_str().unwrap()).unwrap();
                    if with_rejections && i < 2 {
                        assert!(result["error"]["error"].is_string());
                    } else if with_rejections && i == 2 {
                        assert_eq!(result["denied"], true);
                        assert_eq!(result["executed"], false);
                    } else {
                        assert_eq!(result["backend"], "local_fts5", "batch {i} output: {result}");
                        assert_eq!(result["discovery"]["enabled"], i != 9);
                    }
                }
                return ResponseTemplate::new(200).set_body_json(answer("Ten searches completed with explicit empty coverage."));
            }
            let calls:Vec<_>=(0..10).map(|i| {
                let (name, args) = if with_rejections && i == 0 {
                    ("web_read", json!({"url":"http://127.0.0.1/"}))
                } else if with_rejections && i == 1 {
                    ("web_search", json!({"query":""}))
                } else {
                    ("web_search", if i == 9 { json!({"query":format!("topic{i}"), "discover":false}) }
                        else { json!({"query":format!("topic{i}")}) })
                };
                json!({"type":"function_call","call_id":format!("batch-{i}"),"name":name,"arguments":args.to_string()})
            }).collect();
            ResponseTemplate::new(200).set_body_json(json!({"id":"web-batch","status":"completed","output":calls}))
        }).mount(&server).await;
    let submitted = runtime
        .submit("Search ten independent topics", "web")
        .await
        .unwrap();
    let task = submitted["task_id"].as_str().unwrap();
    if with_rejections {
        runtime
            .store
            .put(
                "task_denials",
                task,
                &json!([{"tool":"web_search","arguments":{"query":"topic2"}}]),
            )
            .unwrap();
    }
    let workers = runtime.start();
    tokio::time::timeout(Duration::from_secs(8), async {
        while runtime.store.task(task).unwrap().status != "completed" {
            let status = runtime.store.task(task).unwrap().status;
            assert!(
                !matches!(status.as_str(), "failed" | "waiting"),
                "Unexpected batch task status {status}: {:?}",
                runtime.store.events(task, 0, 100).unwrap()
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let events = runtime.store.events(task, 0, 100).unwrap();
    let starts: Vec<_> = events
        .iter()
        .filter(|e| e["kind"] == "tool_started")
        .collect();
    let results: Vec<_> = events
        .iter()
        .filter(|e| e["kind"] == "tool_result")
        .collect();
    assert_eq!(starts.len(), 10);
    assert_eq!(results.len(), 10);
    assert!(starts.last().unwrap()["seq"].as_i64().unwrap() < results[0]["seq"].as_i64().unwrap());
    let ids: std::collections::HashSet<_> = results
        .iter()
        .map(|e| e["data"]["call_id"].as_str().unwrap())
        .collect();
    assert_eq!(ids.len(), 10);
    assert_eq!(
        results
            .iter()
            .filter(|e| e["data"]["is_error"] == true)
            .count(),
        if with_rejections { 2 } else { 0 }
    );
    assert_eq!(
        runtime.web.status().await.unwrap().coverage.requests_today,
        0
    );
    runtime.shutdown().await.unwrap();
    for worker in workers {
        tokio::time::timeout(Duration::from_secs(5), worker)
            .await
            .unwrap()
            .unwrap();
    }
}
