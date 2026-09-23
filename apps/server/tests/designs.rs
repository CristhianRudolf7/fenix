use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use base64::Engine;
use fenix_agent::Runtime;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

async fn call(app: &axum::Router, method: &str, path: &str, body: Value) -> (StatusCode, Value) {
    let req = Request::builder()
        .method(method)
        .uri(path)
        .header("host", "127.0.0.1:4173")
        .header("content-type", "application/json")
        .body(if method == "GET" {
            Body::empty()
        } else {
            Body::from(body.to_string())
        })
        .unwrap();
    let r = app.clone().oneshot(req).await.unwrap();
    let status = r.status();
    let bytes = r.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(json!(String::from_utf8_lossy(&bytes))),
    )
}
#[tokio::test]
async fn design_api_persists_versions_favorites_brand_and_private_assets() {
    let dir = tempfile::tempdir().unwrap();
    let rt = Runtime::open(dir.path(), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let app = fenix_server::router(rt.clone(), 4173);
    let (code, p) = call(&app, "GET", "/api/design/templates", Value::Null).await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(p["total"], 72);
    assert_eq!(p["templates"].as_array().unwrap().len(), 24);
    assert!(p["templates"][0].get("content").is_none());
    let (_, d) = call(
        &app,
        "POST",
        "/api/design/templates/decision-flow/use",
        json!({"project_id":null}),
    )
    .await;
    let path = format!("/api/design/artifacts/{}", d["id"].as_str().unwrap());
    let edit = json!({"title":"Revised","kind":"diagram","content":{"source":"flowchart LR\n A --> B"},"expected_revision":1});
    assert_eq!(
        call(&app, "PUT", &path, edit.clone()).await.0,
        StatusCode::BAD_REQUEST
    );
    // Diagram revisions go through the agent, never through direct UI edits.
    let saved = rt
        .save_visual(
            Some(d["id"].as_str().unwrap()),
            None,
            serde_json::from_value(edit.clone()).unwrap(),
        )
        .unwrap();
    assert_eq!(saved.revision, 2);
    assert!(
        std::path::Path::new(
            rt.diagram_files(&saved).unwrap()["document"]
                .as_str()
                .unwrap()
        )
        .exists()
    );
    assert_eq!(
        call(&app, "PUT", &path, edit).await.0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(&app, "GET", &format!("{path}?revision=1"), Value::Null)
            .await
            .1["title"],
        "Decision flow"
    );
    assert_eq!(
        call(
            &app,
            "PUT",
            "/api/design/favorites/decision-flow",
            json!({"enabled":true})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, "GET", "/api/design/favorites", Value::Null)
            .await
            .1[0]["id"],
        "decision-flow"
    );
    let brand = json!({"name":"Example","accent":"#112233","background":"#ffffff","text":"#111111","font":"serif"});
    assert_eq!(
        call(&app, "PUT", "/api/design/brand", brand.clone())
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, "GET", "/api/design/brand", Value::Null).await.1["name"],
        brand["name"]
    );
    // Exceeds the ordinary API body limit; the isolated import route must accept it.
    let mut pdf = b"%PDF-1.4\n".to_vec();
    pdf.extend(vec![b' '; 300_000]);
    pdf.extend_from_slice(b"%%EOF");
    let upload = json!({"filename":"large.pdf","data":base64::engine::general_purpose::STANDARD.encode(&pdf),"project_id":null});
    let (status, v) = call(&app, "POST", "/api/design/import", upload).await;
    assert_eq!(status, StatusCode::OK, "{v}");
    let asset = format!(
        "/api/design/assets/{}",
        v["content"]["asset_id"].as_str().unwrap()
    );
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&asset)
                .header("host", "127.0.0.1:4173")
                .header("range", "bytes=0-7")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(response.headers()["cache-control"], "private, no-store");
    assert_eq!(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .as_ref(),
        &pdf[..8]
    );
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&path)
                .header("host", "evil.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    rt.shutdown().await.unwrap();
}
