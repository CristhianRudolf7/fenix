//! Optional filesystem assets for local development and standalone installations.
//! Production serves the frontend independently through Caddy.
use super::*;
use std::path::{Component, Path as FilePath, PathBuf};

fn web_root() -> Option<PathBuf> {
    match std::env::var_os("FENIX_WEB_DIST") {
        Some(path) if path.is_empty() => None,
        Some(path) => Some(PathBuf::from(path)),
        None => Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../web/dist")),
    }
}

async fn serve_from(root: &FilePath, path: &str) -> Response {
    if !FilePath::new(path)
        .components()
        .all(|part| matches!(part, Component::Normal(_)))
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Ok(root) = tokio::fs::canonicalize(root).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(file) = tokio::fs::canonicalize(root.join(path)).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if !file.starts_with(&root) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let mime = match file.extension().and_then(|value| value.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("css") => "text/css; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        Some("webm") => "video/webm",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        _ => "application/octet-stream",
    };
    match tokio::fs::read(file).await {
        Ok(bytes) => (
            [
                (header::CONTENT_TYPE, mime),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            bytes,
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
async fn serve(path: &str) -> Response {
    match web_root() {
        Some(root) => serve_from(&root, path).await,
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
pub async fn index() -> Response {
    serve("index.html").await
}
pub async fn asset(Path(path): Path<String>) -> Response {
    serve(&format!("assets/{path}")).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;
    #[tokio::test]
    async fn assets_update_without_recompilation_and_preserve_pdf_mime_types() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("index.html"), "first").unwrap();
        let response = serve_from(root.path(), "index.html").await;
        assert_eq!(response.status(), StatusCode::OK);
        std::fs::write(root.path().join("index.html"), "second").unwrap();
        let response = serve_from(root.path(), "index.html").await;
        assert_eq!(
            response.into_body().collect().await.unwrap().to_bytes(),
            "second"
        );
        for (file, mime) in [
            ("worker.mjs", "text/javascript; charset=utf-8"),
            ("decoder.wasm", "application/wasm"),
        ] {
            std::fs::write(root.path().join(file), "test").unwrap();
            assert_eq!(
                serve_from(root.path(), file).await.headers()[header::CONTENT_TYPE],
                mime
            );
        }
    }
    #[tokio::test]
    async fn assets_cannot_escape_the_static_root() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("escape")).unwrap();
        for path in ["../secret", "/etc/passwd", "escape", "missing.js"] {
            assert_eq!(
                serve_from(root.path(), path).await.status(),
                StatusCode::NOT_FOUND
            );
        }
    }
}
