use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use fenix_agent::Runtime;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

#[tokio::test]
async fn public_origin_is_explicit_and_preserves_origin_checks() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(dir.path(), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let app = fenix_server::router_with_public_origin(
        runtime.clone(),
        4173,
        Some("https://fenix.example.com"),
    )
    .unwrap();
    for (host, origin, expected) in [
        (
            "fenix.example.com",
            "https://fenix.example.com",
            StatusCode::OK,
        ),
        (
            "fenix.example.com",
            "https://evil.example",
            StatusCode::FORBIDDEN,
        ),
        (
            "fenix.example.com",
            "http://fenix.example.com",
            StatusCode::FORBIDDEN,
        ),
        (
            "evil.example",
            "https://fenix.example.com",
            StatusCode::FORBIDDEN,
        ),
        ("127.0.0.1:4173", "http://127.0.0.1:4173", StatusCode::OK),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/status")
                    .header("host", host)
                    .header("origin", origin)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
    }
    for invalid in [
        "http://fenix.example.com",
        "https://fenix.example.com/path",
        "https://user:pass@fenix.example.com",
        "https://fenix.example.com?query=yes",
    ] {
        assert!(
            fenix_server::router_with_public_origin(runtime.clone(), 4173, Some(invalid)).is_err()
        );
    }
}

#[tokio::test]
async fn progress_sends_a_fresh_snapshot_and_keeps_the_local_origin_guard() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(dir.path(), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    runtime
        .store
        .put("credentials", "openai:default", &"private-not-for-events")
        .unwrap();
    let app = fenix_server::router(runtime.clone(), 4173);
    for origin in ["https://foreign.example", "http://localhost:4173"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/progress")
                    .header("host", "localhost:4173")
                    .header("origin", origin)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        if origin.starts_with("https") {
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
            continue;
        }
        assert_eq!(response.headers()["content-type"], "text/event-stream");
        let mut body = response.into_body();
        let frame = tokio::time::timeout(std::time::Duration::from_secs(1), body.frame())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let text = std::str::from_utf8(frame.data_ref().unwrap()).unwrap();
        assert!(text.contains("event: progress"));
        assert!(text.contains("\"phase\":\"idle\""));
        assert!(!text.contains("private-not-for-events"));
        runtime.shutdown().await.unwrap();
        assert!(
            tokio::time::timeout(std::time::Duration::from_secs(1), body.frame())
                .await
                .unwrap()
                .is_none()
        );
    }
}

#[tokio::test]
async fn loopback_guard_blocks_foreign_hosts_origins_and_form_posts() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(dir.path(), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let app = fenix_server::router(runtime, 4173);
    for (host, origin, method, content_type, expected) in [
        (
            "evil.example:4173",
            None,
            "GET",
            None,
            StatusCode::FORBIDDEN,
        ),
        (
            "127.0.0.1:4173",
            Some("https://evil.example"),
            "GET",
            None,
            StatusCode::FORBIDDEN,
        ),
        (
            "127.0.0.1:4173",
            None,
            "POST",
            Some("text/plain"),
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (
            "127.0.0.1:4173",
            Some("http://127.0.0.1:4173"),
            "GET",
            None,
            StatusCode::OK,
        ),
    ] {
        let mut request = Request::builder()
            .method(method)
            .uri("/api/status")
            .header("host", host);
        if let Some(origin) = origin {
            request = request.header("origin", origin);
        }
        if let Some(content_type) = content_type {
            request = request.header("content-type", content_type);
        }
        let response = app
            .clone()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
    }
}

#[tokio::test]
async fn api_exposes_catalog_and_memory_but_never_private_credentials() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(dir.path(), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    runtime
        .store
        .put("credentials", "openai:default", &"secret-should-not-appear")
        .unwrap();
    let app = fenix_server::router(runtime, 4173);
    let get = |uri: &str| {
        Request::builder()
            .uri(uri)
            .header("host", "localhost:4173")
            .body(Body::empty())
            .unwrap()
    };
    let response = app.clone().oneshot(get("/api/providers")).await.unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let catalog: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(catalog.as_array().unwrap().len(), 17);
    let response = app.clone().oneshot(get("/api/status")).await.unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert!(!String::from_utf8_lossy(&bytes).contains("secret-should-not-appear"));
    let request = Request::builder()
        .method("POST")
        .uri("/api/memory")
        .header("host", "localhost:4173")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"title":"Preference","content":"I prefer short answers"}).to_string(),
        ))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response = app.oneshot(get("/api/memory?query=short")).await.unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(
        serde_json::from_slice::<Value>(&bytes)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

async fn request_json(
    app: axum::Router,
    method: &str,
    uri: &str,
    body: Value,
) -> (StatusCode, Value) {
    let response = app
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header("host", "localhost:4173")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn project_endpoints_preserve_scopes_and_expose_task_bindings() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(dir.path(), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let app = fenix_server::router(runtime.clone(), 4173);
    let (status, project) = request_json(
        app.clone(),
        "POST",
        "/api/projects",
        json!({
            "name": "Project A", "aliases": ["alpha"], "description": "Office planning",
            "brief": "Objective: prepare the launch"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let project_id = project["id"].as_str().unwrap();
    let (status, _) = request_json(
        app.clone(),
        "POST",
        "/api/projects",
        json!({"name":"Second", "aliases":["alpha"]}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let task = runtime
        .store
        .enqueue("Review Project A", "web", None)
        .unwrap();
    runtime
        .store
        .bind_task(
            &task.id,
            &fenix_storage::TaskBinding {
                project_id: Some(project_id.into()),
                mode: "project".into(),
                bound: true,
                context_version: project["context_version"].as_u64().unwrap(),
                ..Default::default()
            },
        )
        .unwrap();
    let general = runtime.store.enqueue("Good morning", "web", None).unwrap();
    let (_, detail) = request_json(
        app.clone(),
        "GET",
        &format!("/api/projects/{project_id}"),
        Value::Null,
    )
    .await;
    assert_eq!(detail["tasks"].as_array().unwrap().len(), 1);
    assert_eq!(detail["tasks"][0]["id"], task.id);
    let (_, history) = request_json(app.clone(), "GET", "/api/messages", Value::Null).await;
    assert_eq!(history.as_array().unwrap().len(), 2);
    assert!(
        history
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["task_id"] == task.id && item["binding"]["project_id"] == project_id)
    );
    assert!(
        history
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["task_id"] == general.id && item["binding"]["project_id"].is_null())
    );
    let (_, details) = request_json(
        app.clone(),
        "GET",
        &format!("/api/tasks/{}", task.id),
        Value::Null,
    )
    .await;
    assert_eq!(details["binding"]["project_id"], project_id);
    let (status, updated) = request_json(
        app.clone(),
        "PUT",
        &format!("/api/projects/{project_id}"),
        json!({
            "name":"Project A", "aliases":["alpha"], "brief":"Objective: launch completed"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        updated["context_version"].as_u64().unwrap(),
        project["context_version"].as_u64().unwrap() + 1
    );
    let (status, _) = request_json(
        app,
        "POST",
        "/api/projects",
        json!({"name":"Bad path", "workspace": "/path-that-does-not-exist/fenix"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn memory_endpoints_filter_before_search_and_reject_scope_changes() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(dir.path(), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let app = fenix_server::router(runtime, 4173);
    let (_, project) = request_json(
        app.clone(),
        "POST",
        "/api/projects",
        json!({"name":"Project A"}),
    )
    .await;
    let project_id = project["id"].as_str().unwrap();
    for scope in [Value::Null, json!(project_id)] {
        let (status, _) = request_json(
            app.clone(),
            "POST",
            "/api/memory",
            json!({"title":"Database", "content":"Use SQLite", "project_id":scope}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }
    let (_, global) =
        request_json(app.clone(), "GET", "/api/memory?query=SQLite", Value::Null).await;
    let (_, scoped) = request_json(
        app.clone(),
        "GET",
        &format!("/api/memory?query=SQLite&project_id={project_id}"),
        Value::Null,
    )
    .await;
    assert_eq!(global.as_array().unwrap().len(), 1);
    assert_eq!(scoped.as_array().unwrap().len(), 1);
    assert!(global[0]["project_id"].is_null());
    assert_eq!(scoped[0]["project_id"], project_id);
    let memory_id = scoped[0]["id"].as_str().unwrap();
    let (status, _) = request_json(
        app.clone(),
        "POST",
        "/api/memory",
        json!({"id":memory_id,"title":"Database","content":"Changed without scope"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = request_json(
        app.clone(),
        "DELETE",
        &format!("/api/memory/{memory_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = request_json(
        app,
        "DELETE",
        &format!("/api/memory/{memory_id}?project_id={project_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn durable_events_resume_from_last_event_id_without_replaying_the_model() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(dir.path(), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    runtime
        .store
        .publish_event(None, "fixture.first", &json!({"order":1}))
        .unwrap();
    runtime
        .store
        .publish_event(None, "fixture.second", &json!({"order":2}))
        .unwrap();
    let first = runtime.store.bus_events(0, 10).unwrap()[0].seq;
    let response = fenix_server::router(runtime.clone(), 4173)
        .oneshot(
            Request::builder()
                .uri("/api/events?after=0")
                .header("host", "localhost:4173")
                .header("last-event-id", first.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.headers()["content-type"], "text/event-stream");
    let mut body = response.into_body();
    let frame = tokio::time::timeout(std::time::Duration::from_secs(1), body.frame())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let text = std::str::from_utf8(frame.data_ref().unwrap()).unwrap();
    assert!(text.contains("fixture.second"));
    assert!(!text.contains("fixture.first"));
    runtime.shutdown().await.unwrap();
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(1), body.frame())
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn adaptive_memory_settings_history_and_entities_are_accessible_without_leaking_keys() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(dir.path(), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let app = fenix_server::router(runtime, 4173);
    let (status,_)=request_json(app.clone(),"POST","/api/memory/settings",json!({"automatic_capture":false,"embedding_endpoint":"http://127.0.0.1:9999/embeddings","embedding_model":"fixture","embedding_api_key":"private-embedding-key"})).await;
    assert_eq!(status, StatusCode::OK);
    let (_, settings) = request_json(app.clone(), "GET", "/api/memory/settings", Value::Null).await;
    assert_eq!(settings["settings"]["automatic_capture"], false);
    assert_eq!(settings["settings"]["automatic_review"], true);
    let (status, health) = request_json(
        app.clone(),
        "POST",
        "/api/memory/knowledge",
        json!({"action":"health"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(health["total_memories"], 0);
    assert_eq!(health["destructive_changes"], false);
    assert!(!settings.to_string().contains("private-embedding-key"));
    let (status,entity)=request_json(app.clone(),"POST","/api/memory/knowledge",json!({"action":"entity_save","entity":{"name":"Research note","kind":"paper","aliases":["Reference"]}})).await;
    assert_eq!(status, StatusCode::OK);
    let (status,memory)=request_json(app.clone(),"POST","/api/memory",json!({"title":"Hypothesis","content":"The note proposes a hypothesis","metadata":{"status":"candidate","entity_id":entity["id"]}})).await;
    assert_eq!(status, StatusCode::OK);
    let (_, pack) = request_json(
        app.clone(),
        "POST",
        "/api/memory/knowledge",
        json!({"action":"context_pack","query":"hypothesis"}),
    )
    .await;
    assert!(!pack["text"].as_str().unwrap().contains("The note proposes"));
    let (_, history) = request_json(
        app.clone(),
        "POST",
        "/api/memory/knowledge",
        json!({"action":"history","id":memory["id"]}),
    )
    .await;
    assert_eq!(history.as_array().unwrap().len(), 1);
    let (status,_)=request_json(app,"POST","/api/memory/knowledge",json!({"action":"schema_save","schema":{"name":"supports","description":"supports a claim","status":"active","source_kinds":["paper"],"target_kinds":["hypothesis"]}})).await;
    assert_eq!(status, StatusCode::OK);
}
