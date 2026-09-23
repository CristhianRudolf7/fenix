//! Tenant-local Design API. All routes share the existing account/origin boundary.
use super::*;
use base64::Engine;
use fenix_agent::designs::{design_templates, sample_video, template};
use fenix_storage::DesignInput;

pub fn routes() -> Router<Arc<Runtime>> {
    Router::new()
        .route("/api/design/templates", get(templates))
        .route("/api/design/templates/{id}", get(template_detail))
        .route("/api/design/templates/{id}/use", post(use_template))
        .route("/api/design/artifacts", get(list).post(create))
        .route("/api/design/artifacts/{id}", get(detail).put(update))
        .route("/api/design/favorites", get(favorites))
        .route("/api/design/offers", get(offers))
        .route("/api/design/favorites/{id}", axum::routing::put(favorite))
        .route("/api/design/brand", get(brand).put(save_brand))
        .route("/api/design/assets/{id}", get(asset))
        .route("/api/design/samples/{id}", get(sample))
        .route(
            "/api/design/import",
            post(import).layer(DefaultBodyLimit::max(90 * 1024 * 1024)),
        )
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
}
#[derive(Default, Deserialize)]
struct Listing {
    query: Option<String>,
    kind: Option<String>,
    project_id: Option<String>,
    offset: Option<usize>,
    revision: Option<u64>,
}
async fn templates(Query(q): Query<Listing>) -> Json<Value> {
    let query = q.query.unwrap_or_default().to_lowercase();
    let filtered = design_templates()
        .iter()
        .filter(|t| {
            q.kind.as_ref().is_none_or(|k| t["kind"] == *k)
                && (query.is_empty()
                    || format!("{} {} {}", t["title"], t["title_es"], t["tags"])
                        .to_lowercase()
                        .contains(&query))
        })
        .collect::<Vec<_>>();
    let offset = q.offset.unwrap_or(0);
    let rows = filtered
        .iter()
        .skip(offset)
        .take(24)
        .map(|t| {
            let mut t = (*t).clone();
            t.as_object_mut().unwrap().remove("content");
            t
        })
        .collect::<Vec<_>>();
    Json(
        json!({"templates":rows,"total":filtered.len(),"next_offset":(offset+rows.len()<filtered.len()).then_some(offset+rows.len())}),
    )
}
async fn template_detail(Path(id): Path<String>) -> ApiResult<Value> {
    Ok(Json(template(&id)?))
}
#[derive(Deserialize)]
struct Selection {
    project_id: Option<String>,
}
async fn use_template(
    State(rt): State<Arc<Runtime>>,
    Path(id): Path<String>,
    Json(v): Json<Selection>,
) -> ApiResult<fenix_storage::Design> {
    Ok(Json(rt.use_design_template(&id, v.project_id)?))
}
async fn list(State(rt): State<Arc<Runtime>>, Query(q): Query<Listing>) -> ApiResult<Value> {
    let offset = q.offset.unwrap_or(0);
    let rows = rt
        .store
        .designs(q.project_id.as_deref(), q.project_id.is_some(), offset, 25)?;
    let has_more = rows.len() > 24;
    let rows = rows.into_iter().take(24).collect::<Vec<_>>();
    Ok(Json(
        json!({"designs":rows,"next_offset":has_more.then_some(offset+24)}),
    ))
}
async fn detail(
    State(rt): State<Arc<Runtime>>,
    Path(id): Path<String>,
    Query(q): Query<Listing>,
) -> ApiResult<fenix_storage::Design> {
    let design = rt.store.design(&id, q.revision)?;
    rt.archive_diagram(&design)?;
    Ok(Json(design))
}
async fn create(
    State(rt): State<Arc<Runtime>>,
    Json(v): Json<DesignInput>,
) -> ApiResult<fenix_storage::Design> {
    Ok(Json(rt.save_visual(None, None, v)?))
}
async fn update(
    State(rt): State<Arc<Runtime>>,
    Path(id): Path<String>,
    Json(v): Json<DesignInput>,
) -> ApiResult<fenix_storage::Design> {
    if v.kind == "diagram" || rt.store.design(&id, None)?.kind == "diagram" {
        return Err(anyhow::anyhow!(
            "Diagrams are edited through the agent. Ask for changes in chat."
        )
        .into());
    }
    Ok(Json(rt.save_visual(Some(&id), None, v)?))
}
async fn favorites(State(rt): State<Arc<Runtime>>) -> ApiResult<Vec<Value>> {
    Ok(Json(rt.store.records("design_favorite")?))
}
async fn favorite(
    State(rt): State<Arc<Runtime>>,
    Path(id): Path<String>,
    Json(v): Json<Value>,
) -> ApiResult<Value> {
    template(&id)?;
    if !v["enabled"].is_boolean() {
        return Err(anyhow::anyhow!("enabled must be boolean").into());
    }
    if v["enabled"] == true {
        rt.store.put("design_favorite", &id, &json!({"id":id}))?;
    } else {
        rt.store.delete("design_favorite", &id)?;
    }
    Ok(Json(json!({"id":id,"enabled":v["enabled"]})))
}
async fn brand(State(rt): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(rt.design_brand()?))
}
async fn save_brand(State(rt): State<Arc<Runtime>>, Json(v): Json<Value>) -> ApiResult<Value> {
    Ok(Json(rt.save_design_brand(&v)?))
}
#[derive(Deserialize)]
struct Upload {
    filename: String,
    data: String,
    project_id: Option<String>,
}
async fn import(
    State(rt): State<Arc<Runtime>>,
    Json(v): Json<Upload>,
) -> ApiResult<fenix_storage::Design> {
    if v.data.len() > 89_478_488 {
        return Err(anyhow::anyhow!("File exceeds 64 MiB").into());
    }
    let bytes = base64::engine::general_purpose::STANDARD.decode(v.data)?;
    Ok(Json(rt.import_visual(
        &v.filename,
        &bytes,
        v.project_id,
        None,
    )?))
}
fn media(bytes: Vec<u8>, mime: &str, headers: &axum::http::HeaderMap) -> Response {
    let size = bytes.len();
    let range = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("bytes="));
    let selected = range.and_then(|s| s.split_once('-')).and_then(|(a, b)| {
        if a.is_empty() {
            let n = b.parse::<usize>().ok()?;
            Some((size.saturating_sub(n), size.checked_sub(1)?))
        } else {
            let start = a.parse::<usize>().ok()?;
            let end = if b.is_empty() {
                size.checked_sub(1)?
            } else {
                b.parse::<usize>().ok()?.min(size.checked_sub(1)?)
            };
            Some((start, end))
        }
    });
    let mut response = if range.is_some() {
        if let Some((start, end)) = selected.filter(|(a, b)| a <= b && *a < size) {
            (
                StatusCode::PARTIAL_CONTENT,
                [(header::CONTENT_RANGE, format!("bytes {start}-{end}/{size}"))],
                bytes[start..=end].to_vec(),
            )
                .into_response()
        } else {
            (
                StatusCode::RANGE_NOT_SATISFIABLE,
                [(header::CONTENT_RANGE, format!("bytes */{size}"))],
            )
                .into_response()
        }
    } else {
        bytes.into_response()
    };
    let h = response.headers_mut();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_str(mime).unwrap());
    h.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    h.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    h.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    h.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'; sandbox"),
    );
    response
}
async fn asset(
    State(rt): State<Arc<Runtime>>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Response, ApiError> {
    let (meta, bytes) = rt.design_asset(&id)?;
    Ok(media(
        bytes,
        meta["mime"].as_str().unwrap_or("application/octet-stream"),
        &headers,
    ))
}
async fn sample(
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Response, ApiError> {
    Ok(media(sample_video(&id)?.to_vec(), "video/webm", &headers))
}

async fn offers(State(rt): State<Arc<Runtime>>) -> ApiResult<Vec<Value>> {
    Ok(Json(
        rt.store
            .records::<Value>("design_offer")?
            .into_iter()
            .take(3)
            .collect(),
    ))
}
