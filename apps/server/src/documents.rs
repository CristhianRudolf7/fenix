//! Account-local library routes inherit the application's authentication/origin checks.
use super::*;
use base64::Engine;
use fenix_agent::library::{ImportOptions, Search};

pub fn routes() -> Router<Arc<Runtime>> {
    Router::new()
        .route("/documents", get(assets::index))
        .route("/api/documents", get(list))
        .route(
            "/api/documents/import",
            post(import).layer(DefaultBodyLimit::max(90 * 1024 * 1024)),
        )
        .route("/api/documents/search", post(search))
        .route("/api/documents/settings", get(settings).put(configure))
        .route(
            "/api/documents/{id}",
            get(detail).put(metadata).delete(delete),
        )
        .route("/api/documents/{id}/retry", post(retry))
        .route("/api/documents/{id}/original", get(original))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
}
#[derive(Default, Deserialize)]
struct Listing {
    project_id: Option<String>,
    all: Option<bool>,
    offset: Option<usize>,
    revision: Option<u64>,
    chunk: Option<String>,
}
async fn list(State(rt): State<Arc<Runtime>>, Query(q): Query<Listing>) -> ApiResult<Value> {
    let offset = q.offset.unwrap_or(0);
    let docs =
        rt.store
            .library_documents(q.project_id.as_deref(), q.all.unwrap_or(false), offset, 51)?;
    let more = docs.len() > 50;
    Ok(Json(
        json!({"documents":docs.into_iter().take(50).collect::<Vec<_>>(),"next_offset":more.then_some(offset+50)}),
    ))
}
#[derive(Deserialize)]
struct Upload {
    filename: String,
    data: String,
    #[serde(flatten)]
    options: ImportOptions,
}
async fn import(State(rt): State<Arc<Runtime>>, Json(v): Json<Upload>) -> ApiResult<Value> {
    super::require(v.data.len() <= 89_478_488, "File exceeds 64 MiB")?;
    let bytes = base64::engine::general_purpose::STANDARD.decode(&v.data)?;
    Ok(Json(json!(
        rt.library_import(&v.filename, &bytes, v.options).await?
    )))
}
async fn detail(
    State(rt): State<Arc<Runtime>>,
    Path(id): Path<String>,
    Query(q): Query<Listing>,
) -> ApiResult<Value> {
    let mut offset = q.offset.unwrap_or(0);
    let mut revision = q.revision;
    if let Some(chunk) = &q.chunk {
        let source = rt.store.library_chunk(chunk)?;
        super::require(
            source.document_id == id && q.revision.is_none_or(|r| r == source.revision),
            "Citation belongs to another document revision",
        )?;
        offset = source.ordinal / 20 * 20;
        revision = Some(source.revision);
    }
    let mut value = rt.library_read(&id, revision, offset)?;
    value["revisions"] = json!(rt.store.library_revisions(&id)?);
    Ok(Json(value))
}
async fn search(State(rt): State<Arc<Runtime>>, Json(v): Json<Search>) -> ApiResult<Value> {
    Ok(Json(rt.library_search(v).await?))
}
async fn settings(State(rt): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(rt.library_settings()?))
}
#[derive(Deserialize)]
struct Configuration {
    enabled: bool,
    api_key: Option<String>,
}
async fn configure(
    State(rt): State<Arc<Runtime>>,
    Json(v): Json<Configuration>,
) -> ApiResult<Value> {
    Ok(Json(rt.library_configure(v.enabled, v.api_key)?))
}
#[derive(Deserialize)]
struct Revision {
    expected_revision: u64,
}
async fn retry(
    State(rt): State<Arc<Runtime>>,
    Path(id): Path<String>,
    Json(v): Json<Revision>,
) -> ApiResult<Value> {
    Ok(Json(json!(
        rt.library_retry(&id, v.expected_revision).await?
    )))
}
async fn delete(
    State(rt): State<Arc<Runtime>>,
    Path(id): Path<String>,
    Json(v): Json<Revision>,
) -> ApiResult<Value> {
    rt.library_delete(&id, v.expected_revision).await?;
    Ok(Json(json!({"deleted":true})))
}
#[derive(Deserialize)]
struct Metadata {
    expected_revision: u64,
    title: String,
    collection: String,
    tags: Vec<String>,
    enabled: bool,
}
async fn metadata(
    State(rt): State<Arc<Runtime>>,
    Path(id): Path<String>,
    Json(v): Json<Metadata>,
) -> ApiResult<Value> {
    Ok(Json(json!(rt.store.library_metadata(
        &id,
        v.expected_revision,
        v.title,
        v.collection,
        v.tags,
        v.enabled
    )?)))
}
async fn original(
    State(rt): State<Arc<Runtime>>,
    Path(id): Path<String>,
    Query(q): Query<Listing>,
) -> Result<Response, ApiError> {
    let doc = rt.store.library_document(&id, q.revision)?;
    let bytes = rt.library_original(&doc)?;
    let mime = match doc.format.as_str() {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        _ => "application/octet-stream",
    };
    let mut r = bytes.into_response();
    let h = r.headers_mut();
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
    let name = doc
        .filename
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || "._- ".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    h.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{name}\""))?,
    );
    Ok(r)
}
