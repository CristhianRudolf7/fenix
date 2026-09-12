//! Loopback-only HTTP adapter. No separate chat state; Runtime owns the conversation.
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, Query, Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use fenix_agent::{
    Runtime,
    config::{Settings, provider_catalog},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

type ApiResult<T> = Result<Json<T>, ApiError>;
pub struct ApiError(anyhow::Error);
impl<E: Into<anyhow::Error>> From<E> for ApiError {
    fn from(error: E) -> Self {
        Self(error.into())
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error":self.0.to_string()})),
        )
            .into_response()
    }
}

pub fn router(runtime: Arc<Runtime>, port: u16) -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { Html(include_str!("../../web/index.html")) }),
        )
        .route(
            "/app.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("../../web/app.js"),
                )
            }),
        )
        .route(
            "/styles.css",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
                    include_str!("../../web/styles.css"),
                )
            }),
        )
        .route("/api/status", get(status))
        .route("/api/providers", get(|| async { Json(provider_catalog()) }))
        .route("/api/settings", post(settings))
        .route("/api/telegram", post(telegram))
        .route("/api/messages", get(messages).post(message))
        .route("/api/memory", get(memories).post(save_memory))
        .route("/api/memory/{id}", axum::routing::delete(delete_memory))
        .route("/api/skills", get(skills))
        .route("/api/skills/{name}", get(skill).put(skill_enabled))
        .route("/api/schedules", get(schedules).post(schedule))
        .route("/api/tasks", get(tasks))
        .route("/api/tasks/{id}", get(task))
        .route(
            "/api/instructions/{name}",
            get(instruction).put(save_instruction),
        )
        .layer(DefaultBodyLimit::max(256 * 1024))
        .layer(middleware::from_fn(move |request: Request, next: Next| {
            local_guard(request, next, port)
        }))
        .with_state(runtime)
}

async fn local_guard(request: Request, next: Next, port: u16) -> Response {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let allowed = [format!("127.0.0.1:{port}"), format!("localhost:{port}")];
    if !allowed.iter().any(|h| h == host) {
        return StatusCode::FORBIDDEN.into_response();
    }
    if let Some(origin) = request.headers().get(header::ORIGIN)
        && origin.to_str().ok() != Some(format!("http://{host}").as_str())
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    if request
        .headers()
        .get("sec-fetch-site")
        .is_some_and(|h| h == "cross-site")
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    if !matches!(
        *request.method(),
        axum::http::Method::GET | axum::http::Method::HEAD
    ) && request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(';').next())
        != Some(Some("application/json"))
    {
        return StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response();
    }
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert("content-security-policy",HeaderValue::from_static("default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'"));
    response
}
async fn status(State(runtime): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(runtime.status().await?))
}
#[derive(Deserialize)]
struct Setup {
    #[serde(flatten)]
    settings: Settings,
    #[serde(default)]
    api_key: Option<String>,
}
async fn settings(
    State(runtime): State<Arc<Runtime>>,
    Json(input): Json<Setup>,
) -> ApiResult<Value> {
    Ok(Json(
        runtime.configure(input.settings, input.api_key).await?,
    ))
}
#[derive(Deserialize)]
struct TelegramInput {
    token: Option<String>,
    enabled: bool,
}
async fn telegram(
    State(runtime): State<Arc<Runtime>>,
    Json(input): Json<TelegramInput>,
) -> ApiResult<Value> {
    Ok(Json(
        runtime
            .configure_telegram(input.token, input.enabled)
            .await?,
    ))
}
#[derive(Deserialize, Default)]
struct Page {
    before: Option<i64>,
    limit: Option<usize>,
    after: Option<i64>,
    query: Option<String>,
    call_id: Option<String>,
    offset: Option<usize>,
    length: Option<usize>,
}
async fn messages(
    State(runtime): State<Arc<Runtime>>,
    Query(page): Query<Page>,
) -> ApiResult<Value> {
    // Provider replay metadata is internal, never needed by a browser.
    Ok(Json(json!(runtime.store.conversation(page.before,page.limit.unwrap_or(100))?.into_iter().map(|m|json!({"seq":m.seq,"id":m.id,"task_id":m.task_id,"role":m.role,"text":m.text,"origin":m.origin,"created_at":m.created_at})).collect::<Vec<_>>())))
}
#[derive(Deserialize)]
struct TextInput {
    text: String,
}
async fn message(
    State(runtime): State<Arc<Runtime>>,
    Json(input): Json<TextInput>,
) -> ApiResult<Value> {
    Ok(Json(runtime.submit(&input.text, "web").await?))
}
async fn memories(
    State(runtime): State<Arc<Runtime>>,
    Query(page): Query<Page>,
) -> ApiResult<Value> {
    Ok(Json(json!(runtime.store.memories(
        page.query.as_deref().unwrap_or(""),
        page.limit.unwrap_or(100)
    )?)))
}
#[derive(Deserialize)]
struct MemoryInput {
    id: Option<String>,
    title: String,
    content: String,
}
async fn save_memory(
    State(runtime): State<Arc<Runtime>>,
    Json(input): Json<MemoryInput>,
) -> ApiResult<Value> {
    let memory = runtime.store.save_memory(
        input.id.as_deref(),
        &runtime.redactor.text(&input.title),
        &runtime.redactor.text(&input.content),
        "web:explicit-owner-edit",
    )?;
    runtime.export_memory()?;
    Ok(Json(json!(memory)))
}
async fn delete_memory(
    State(runtime): State<Arc<Runtime>>,
    Path(id): Path<String>,
) -> ApiResult<Value> {
    runtime.store.delete_memory(&id)?;
    runtime.export_memory()?;
    Ok(Json(json!({"deleted":id})))
}
async fn skills(State(runtime): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(json!(runtime.skills()?)))
}
async fn skill(State(runtime): State<Arc<Runtime>>, Path(name): Path<String>) -> ApiResult<Value> {
    Ok(Json(json!({"content":runtime.skill_load(&name)?})))
}
#[derive(Deserialize)]
struct Enabled {
    enabled: bool,
}
async fn skill_enabled(
    State(runtime): State<Arc<Runtime>>,
    Path(name): Path<String>,
    Json(input): Json<Enabled>,
) -> ApiResult<Value> {
    runtime.set_skill_enabled(&name, input.enabled)?;
    Ok(Json(json!({"enabled":input.enabled})))
}
async fn schedules(State(runtime): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(json!(
        runtime
            .store
            .records::<fenix_agent::schedules::Schedule>("schedules")?
    )))
}
async fn schedule(
    State(runtime): State<Arc<Runtime>>,
    Json(input): Json<Value>,
) -> ApiResult<Value> {
    Ok(Json(runtime.schedule_manage(input).await?))
}
async fn tasks(State(runtime): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(json!(runtime.store.tasks(100)?)))
}
async fn task(
    State(runtime): State<Arc<Runtime>>,
    Path(id): Path<String>,
    Query(page): Query<Page>,
) -> ApiResult<Value> {
    if let Some(call) = page.call_id {
        return Ok(Json(runtime.store.tool_output(
            &id,
            &call,
            page.offset.unwrap_or(0),
            page.length.unwrap_or(8000),
        )?));
    }
    Ok(Json(
        json!({"task":runtime.store.task(&id)?,"events":runtime.store.events(&id,page.after.unwrap_or(0),page.limit.unwrap_or(50))?}),
    ))
}
fn instruction_path(runtime: &Runtime, name: &str) -> Result<std::path::PathBuf, ApiError> {
    if !["AGENTS.md", "SOUL.md", "IDENTITY.md", "USER.md"].contains(&name) {
        return Err(anyhow::anyhow!("Unknown editable instruction file").into());
    }
    Ok(runtime.data_dir.join("instructions").join(name))
}
async fn instruction(
    State(runtime): State<Arc<Runtime>>,
    Path(name): Path<String>,
) -> ApiResult<Value> {
    Ok(Json(
        json!({"text":runtime.redactor.text(&std::fs::read_to_string(instruction_path(&runtime,&name)?)?)}),
    ))
}
async fn save_instruction(
    State(runtime): State<Arc<Runtime>>,
    Path(name): Path<String>,
    Json(input): Json<TextInput>,
) -> ApiResult<Value> {
    if input.text.len() > 8000 {
        return Err(anyhow::anyhow!("Instructions must be at most 8000 bytes per file").into());
    }
    fenix_agent::write_private(
        &instruction_path(&runtime, &name)?,
        &runtime.redactor.text(&input.text),
    )?;
    Ok(Json(json!({"saved":name})))
}
