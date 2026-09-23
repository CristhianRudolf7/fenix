use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use fenix_agent::{Runtime, config::Settings};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use std::{path::Path, sync::Arc};
use tower::ServiceExt;

async fn account(root: &Path) -> Arc<Runtime> {
    std::fs::create_dir_all(root.join("workspace")).unwrap();
    let mut rt = Runtime::open(&root.join("state"), Some(&root.join("skills")))
        .await
        .unwrap();
    Arc::get_mut(&mut rt).unwrap().tenant_root = Some(root.to_owned());
    rt.store
        .put(
            "settings",
            "active",
            &Settings::initial(root.join("workspace")),
        )
        .unwrap();
    rt
}
async fn request(app: &axum::Router, path: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(path)
                .header("host", "127.0.0.1:4173")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(json!(String::from_utf8_lossy(&bytes))),
    )
}
#[tokio::test]
async fn workspace_routes_keep_accounts_separate_and_reject_path_escapes() {
    let dir = tempfile::tempdir().unwrap();
    let a = account(&dir.path().join("alice")).await;
    let b = account(&dir.path().join("bob")).await;
    let aw = dir.path().join("alice/workspace");
    let bw = dir.path().join("bob/workspace");
    std::fs::write(aw.join("report.md"), "Alice report").unwrap();
    std::fs::write(bw.join("report.md"), "Bob private report").unwrap();
    std::fs::write(aw.join(".env"), "private-key").unwrap();
    std::os::unix::fs::symlink(&bw, aw.join("linked-user")).unwrap();
    std::fs::write(bw.join("link-source.md"), "Other private file").unwrap();
    std::fs::hard_link(bw.join("link-source.md"), aw.join("linked-file.md")).unwrap();
    let app_a = fenix_server::router(a.clone(), 4173);
    let app_b = fenix_server::router(b.clone(), 4173);
    let (status, list) = request(&app_a, "/api/workspace/files").await;
    assert_eq!(status, StatusCode::OK, "{list}");
    assert!(!list.to_string().contains("linked-user"));
    assert!(!list.to_string().contains(".env"));
    assert!(!list.to_string().contains("linked-file"));
    assert_eq!(
        request(&app_a, "/api/workspace/preview?path=report.md")
            .await
            .1["text"],
        "Alice report"
    );
    assert_eq!(
        request(&app_b, "/api/workspace/content?path=report.md")
            .await
            .1,
        "Bob private report"
    );
    for endpoint in ["files", "preview", "content"] {
        for path in [
            "../bob/workspace/report.md",
            "/etc/passwd",
            "linked-user/report.md",
            ".env",
            "../state/fenix.sqlite3",
        ] {
            assert_eq!(
                request(&app_a, &format!("/api/workspace/{endpoint}?path={path}"))
                    .await
                    .0,
                StatusCode::BAD_REQUEST,
                "{endpoint}: {path}"
            );
        }
        assert_eq!(
            request(
                &app_a,
                &format!(
                    "/api/workspace/{endpoint}?project_id=unknown-other-account&path=report.md"
                )
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
    }
    assert_eq!(
        request(&app_a, "/api/workspace/content?path=linked-file.md")
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    // Even corrupted stored settings cannot make the browser cross the account root.
    a.store
        .put("settings", "active", &Settings::initial(bw))
        .unwrap();
    assert_eq!(
        request(&app_a, "/api/workspace/files").await.0,
        StatusCode::BAD_REQUEST
    );
    a.shutdown().await.unwrap();
    b.shutdown().await.unwrap();
}
#[tokio::test]
async fn workspace_paginates_files_and_extracts_the_requested_slide() {
    let dir = tempfile::tempdir().unwrap();
    let rt = account(&dir.path().join("user")).await;
    let workspace = dir.path().join("user/workspace");
    for i in 0..105 {
        std::fs::write(workspace.join(format!("note-{i:03}.md")), "text").unwrap();
    }
    let app = fenix_server::router(rt.clone(), 4173);
    let first = request(&app, "/api/workspace/files?query=note-").await.1;
    assert_eq!(first["entries"].as_array().unwrap().len(), 100);
    assert_eq!(first["next_offset"], 100);
    assert_eq!(
        request(&app, "/api/workspace/files?query=note-&offset=100")
            .await
            .1["entries"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
    let spec = serde_json::from_value(
        json!({"format":"pptx","slides":[{"title":"First slide","bullets":["Overview"]},{"title":"Second slide","bullets":["Requested content"]}]}),
    );
    // Use the same native authoring schema as the agent.
    let spec: fenix_documents::DocumentSpec = spec.unwrap();
    std::fs::write(
        workspace.join("talk.pptx"),
        fenix_documents::create(spec).unwrap(),
    )
    .unwrap();
    let (status, preview) = request(&app, "/api/workspace/preview?path=talk.pptx&page=2").await;
    assert_eq!(status, StatusCode::OK, "{preview}");
    assert!(
        preview["extraction"]["blocks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|b| b["part"] == "ppt/slides/slide2.xml")
    );
    assert!(preview.to_string().contains("Requested content"));
    assert!(!preview.to_string().contains("First slide"));
    rt.shutdown().await.unwrap();
}
