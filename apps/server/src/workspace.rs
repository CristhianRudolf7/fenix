//! Read-only workspace routes under the existing account/origin boundary.
use super::*;
pub fn routes() -> Router<Arc<Runtime>> {
    Router::new()
        .route("/api/workspace/files", get(files))
        .route("/api/workspace/preview", get(preview))
        .route("/api/workspace/content", get(content))
}
#[derive(Deserialize, Default)]
struct Browse {
    project_id: Option<String>,
    #[serde(default)]
    path: String,
    #[serde(default)]
    query: String,
    #[serde(default)]
    offset: usize,
    page: Option<u64>,
}
async fn files(State(rt): State<Arc<Runtime>>, Query(q): Query<Browse>) -> ApiResult<Value> {
    Ok(Json(
        tokio::task::spawn_blocking(move || {
            rt.workspace_files(q.project_id.as_deref(), &q.path, &q.query, q.offset)
        })
        .await??,
    ))
}
async fn preview(State(rt): State<Arc<Runtime>>, Query(q): Query<Browse>) -> ApiResult<Value> {
    Ok(Json(
        tokio::task::spawn_blocking(move || {
            rt.workspace_preview(q.project_id.as_deref(), &q.path, q.offset, q.page)
        })
        .await??,
    ))
}
async fn content(
    State(rt): State<Arc<Runtime>>,
    Query(q): Query<Browse>,
) -> Result<Response, ApiError> {
    let path = q.path.clone();
    let bytes =
        tokio::task::spawn_blocking(move || rt.workspace_bytes(q.project_id.as_deref(), &q.path))
            .await??;
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mime = match ext.as_str() {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    };
    let mut response = bytes.into_response();
    let h = response.headers_mut();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static(mime));
    h.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    h.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    h.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'; sandbox"),
    );
    Ok(response)
}
