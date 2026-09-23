use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use base64::Engine;
use fenix_agent::Runtime;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;
async fn call(app: &axum::Router, method: &str, path: &str, value: Value) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header("host", "127.0.0.1:4173")
        .header("content-type", "application/json")
        .body(if method == "GET" {
            Body::empty()
        } else {
            Body::from(value.to_string())
        })
        .unwrap();
    let result = app.clone().oneshot(request).await.unwrap();
    let status = result.status();
    let bytes = result.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(json!(String::from_utf8_lossy(&bytes))),
    )
}
#[tokio::test]
async fn document_upload_queue_citations_versions_and_settings_use_private_routes() {
    let dir = tempfile::tempdir().unwrap();
    let rt = Runtime::open(dir.path(), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let workers = rt.start();
    let app = fenix_server::router(rt.clone(), 4173);
    let upload = |text: &[u8]| json!({"filename":"agreement.md","data":base64::engine::general_purpose::STANDARD.encode(text),"title":"Agreement","collection":"Legal"});
    // Larger than the ordinary API body limit; import has its own validated limit.
    let mut text = b"# Agreement\n\nCancellation requires thirty days notice.\n\n".to_vec();
    text.extend(vec![b' '; 300_000]);
    let (status, doc) = call(&app, "POST", "/api/documents/import", upload(&text)).await;
    assert_eq!(status, StatusCode::OK, "{doc}");
    let id = doc["id"].as_str().unwrap();
    let path = format!("/api/documents/{id}");
    for _ in 0..200 {
        if rt.store.library_document(id, None).unwrap().status == "ready" {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert_eq!(rt.store.library_document(id, None).unwrap().status, "ready");
    let (status, found) = call(
        &app,
        "POST",
        "/api/documents/search",
        json!({"query":"cancellation","semantic":false}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{found}");
    let hit = &found["results"][0];
    assert_eq!(hit["document_id"], id);
    let chunk = hit["id"].as_str().unwrap();
    let (_, detail) = call(&app, "GET", &format!("{path}?chunk={chunk}"), Value::Null).await;
    assert_eq!(detail["document"]["revision"], 1);
    assert_eq!(detail["chunks"][1]["id"], chunk);
    let mut revision = upload(b"Cancellation requires sixty days notice.");
    revision["id"] = json!(id);
    revision["expected_revision"] = json!(1);
    assert_eq!(
        call(&app, "POST", "/api/documents/import", revision.clone())
            .await
            .1["revision"],
        2
    );
    assert_eq!(
        call(&app, "POST", "/api/documents/import", revision)
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(&app, "GET", &format!("{path}?chunk={chunk}"), Value::Null)
            .await
            .1["document"]["revision"],
        1
    );
    assert_eq!(
        call(
            &app,
            "GET",
            &format!("{path}?chunk={chunk}&revision=2"),
            Value::Null
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(
            &app,
            "GET",
            &format!("{path}/original?revision=1"),
            Value::Null
        )
        .await
        .0,
        StatusCode::OK
    );
    let (_, settings) = call(
        &app,
        "PUT",
        "/api/documents/settings",
        json!({"enabled":false,"api_key":"private-test-key"}),
    )
    .await;
    assert_eq!(settings["model"], "text-embedding-3-small");
    assert_eq!(settings["configured"], true);
    assert!(!settings.to_string().contains("private-test-key"));
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/documents")
                .header("host", "attacker.invalid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let other_dir = tempfile::tempdir().unwrap();
    let other = Runtime::open(other_dir.path(), Some(&other_dir.path().join("skills")))
        .await
        .unwrap();
    let other_app = fenix_server::router(other, 4173);
    assert_eq!(
        call(&other_app, "GET", &path, Value::Null).await.0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(&app, "DELETE", &path, json!({"expected_revision":2}))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &app,
            "GET",
            &format!("{path}/original?revision=1"),
            Value::Null
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    rt.stop.cancel();
    for worker in workers {
        worker.await.unwrap();
    }
}
