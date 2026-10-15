//! Loopback-only HTTP adapter. No separate chat state; Runtime owns the conversation.
use anyhow::Context;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, Query, Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use fenix_agent::{
    Runtime,
    config::{Settings, provider_catalog},
};
use fenix_storage::ProjectInput;
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio_stream::wrappers::WatchStream;

mod assets;
pub mod auth;
mod browser;
mod connectors;
mod design_choice;
mod designs;
mod documents;
mod integrations;
mod model_roles;
mod skill_management;
mod workflows;
mod workspace;

fn require(condition: bool, message: &str) -> anyhow::Result<()> {
    anyhow::ensure!(condition, "{message}");
    Ok(())
}

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

/// The production single-owner server. Raw routers below are transport fixtures.
pub fn personal_router(
    runtime: Arc<Runtime>,
    port: u16,
    public_origin: Option<&str>,
) -> anyhow::Result<Router> {
    let app = router_with_public_origin(runtime.clone(), port, public_origin)?;
    let origin = public_origin.map(str::to_owned);
    Ok(
        auth::protect(app, runtime, public_origin.is_some()).layer(middleware::from_fn(
            move |request: Request, next: Next| local_guard(request, next, port, origin.clone()),
        )),
    )
}

pub fn router(runtime: Arc<Runtime>, port: u16) -> Router {
    router_with_public_origin(runtime, port, None).expect("Local router configuration is valid")
}

/// Transport router; production callers must use personal_router for owner authentication.
pub fn router_with_public_origin(
    runtime: Arc<Runtime>,
    port: u16,
    public_origin: Option<&str>,
) -> anyhow::Result<Router> {
    let public_origin = public_origin
        .map(|value| -> anyhow::Result<String> {
            let url = reqwest::Url::parse(value)?;
            anyhow::ensure!(
                url.scheme() == "https"
                    && url.host_str().is_some()
                    && url.username().is_empty()
                    && url.password().is_none()
                    && url.path() == "/"
                    && url.query().is_none()
                    && url.fragment().is_none(),
                "FENIX_PUBLIC_ORIGIN must be an HTTPS origin without a path, credentials or query"
            );
            Ok(url.origin().ascii_serialization())
        })
        .transpose()?;
    runtime.browser.set_control_port(port);
    Ok(Router::new()
        .merge(browser::routes())
        .merge(integrations::routes())
        .merge(connectors::routes())
        .merge(workflows::routes())
        .merge(model_roles::routes())
        .merge(designs::routes())
        .merge(documents::routes())
        .merge(workspace::routes())
        .route("/", get(assets::index))
        .route("/chat", get(assets::index))
        .route("/projects", get(assets::index))
        .route("/memory", get(assets::index))
        .route("/skills", get(assets::index))
        .route("/design", get(assets::index))
        .route("/browser", get(assets::index))
        .route("/integrations", get(assets::index))
        .route("/schedules", get(assets::index))
        .route("/activity", get(assets::index))
        .route("/settings", get(assets::index))
        .route("/profile", get(assets::index))
        .route("/assets/{*path}", get(assets::asset))
        .route("/api/status", get(status))
        .route("/api/account/usage", get(account_usage))
        .route("/api/web/status", get(web_status))
        .route("/api/web/search", post(web_search))
        .route("/api/progress", get(progress))
        .route("/api/events", get(events))
        .route("/api/plans", get(plans))
        .route("/api/plans/{id}", get(plan))
        .route("/api/tasks/{id}/replan", post(replan))
        .route("/api/providers", get(|| async { Json(provider_catalog()) }))
        .route("/api/settings", post(settings))
        .route("/api/telegram", post(telegram))
        .route("/api/messages", get(messages).post(message))
        .route("/api/chat", post(message))
        .route("/api/projects", get(projects).post(create_project))
        .route("/api/projects/{id}", get(project).put(update_project))
        .route("/api/memory", get(memories).post(save_memory))
        .route("/api/memory/knowledge", axum::routing::post(knowledge))
        .route(
            "/api/memory/settings",
            get(memory_settings).post(configure_memory),
        )
        .route("/api/memory/{id}", axum::routing::delete(delete_memory))
        .route(
            "/api/decisions/settings",
            get(decision_settings).post(configure_decisions),
        )
        .route("/api/skills", get(skills))
        .route("/api/skill-categories", get(skill_categories))
        .route("/api/skill-localizations", post(skill_localizations))
        .route("/api/skill-localizations/preview", post(skill_preview))
        .route("/api/skills/{name}", get(skill).put(skill_enabled))
        .route(
            "/api/skills/{name}/source",
            get(skill_management::source)
                .put(skill_management::save)
                .delete(skill_management::remove),
        )
        .route("/api/schedules", get(schedules).post(schedule))
        .route("/api/tasks", get(tasks))
        .route("/api/tasks/{id}", get(task))
        .route(
            "/api/instructions/{name}",
            get(instruction).put(save_instruction),
        )
        .with_state(runtime)
        .layer(DefaultBodyLimit::max(256 * 1024))
        .layer(middleware::from_fn(move |request: Request, next: Next| {
            local_guard(request, next, port, public_origin.clone())
        })))
}

async fn local_guard(
    request: Request,
    next: Next,
    port: u16,
    public_origin: Option<String>,
) -> Response {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let allowed = [format!("127.0.0.1:{port}"), format!("localhost:{port}")];
    let remote = public_origin
        .as_deref()
        .is_some_and(|origin| origin.strip_prefix("https://") == Some(host));
    if !remote && !allowed.iter().any(|h| h == host) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let expected_origin = if remote {
        public_origin.as_deref().unwrap().to_owned()
    } else {
        format!("http://{host}")
    };
    // This GET contains only a static OAuth landing page, never owner data.
    // The token exchange is a separate authenticated same-origin JSON POST.
    let oauth_landing = request.method() == axum::http::Method::GET
        && matches!(
            request.uri().path(),
            "/api/connectors/oauth/callback" | "/api/auth/google/callback"
        );
    if !oauth_landing
        && let Some(origin) = request.headers().get(header::ORIGIN)
        && origin.to_str().ok() != Some(expected_origin.as_str())
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    if !oauth_landing
        && request
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
    headers
        .entry(header::CACHE_CONTROL)
        .or_insert(HeaderValue::from_static("no-store"));
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.entry("content-security-policy").or_insert(HeaderValue::from_static("default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'"));
    response
}
async fn status(State(runtime): State<Arc<Runtime>>) -> ApiResult<Value> {
    let mut value = runtime.status().await?;
    value["model_health"] = runtime.model_health_status().await?;
    Ok(Json(value))
}
#[derive(Deserialize)]
struct UsageQuery {
    days: Option<u32>,
}
async fn account_usage(
    State(runtime): State<Arc<Runtime>>,
    Query(query): Query<UsageQuery>,
) -> ApiResult<Value> {
    let days = query.days.unwrap_or(30);
    Ok(Json(
        tokio::task::spawn_blocking(move || runtime.store.usage_summary(days)).await??,
    ))
}

async fn web_status(State(runtime): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(serde_json::to_value(runtime.web.status().await?)?))
}

async fn web_search(
    State(runtime): State<Arc<Runtime>>,
    Json(mut input): Json<Value>,
) -> ApiResult<Value> {
    // The owner's search UI reads the local index by default. Fetching original
    // pages requires explicit read_pages or source_urls, never another search service.
    input
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("Search requires a JSON object"))?
        .entry("read_pages")
        .or_insert(json!(0));
    let request = serde_json::from_value(input)?;
    let cancel = tokio_util::sync::CancellationToken::new();
    let _cancel_on_drop = cancel.clone().drop_guard();
    Ok(Json(serde_json::to_value(
        runtime.web.search_with(request, &cancel).await?,
    )?))
}

#[derive(Deserialize, Default)]
struct EventCursor {
    after: Option<i64>,
}

async fn events(
    State(runtime): State<Arc<Runtime>>,
    Query(query): Query<EventCursor>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let cursor = headers
        .get("last-event-id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(query.after.unwrap_or(0))
        .max(0);
    let stop = runtime.stop.clone();
    let stream=futures_util::stream::unfold((runtime,cursor,std::collections::VecDeque::new()), |(runtime,mut cursor,mut queued)| async move {
        loop {
            if runtime.stop.is_cancelled() {return None}
            if let Some(event) = queued.pop_front() { return Some((Ok::<_,std::convert::Infallible>(event),(runtime,cursor,queued))); }
            match runtime.store.bus_events(cursor,100) {
                Ok(events) if !events.is_empty() => {
                    for event in events {
                        cursor=event.seq;
                        queued.push_back(Event::default().id(event.seq.to_string()).event("activity").data(serde_json::to_string(&event).expect("Serializable event")));
                    }
                },
                Err(error) => {
                    eprintln!("Event stream: {}",runtime.redactor.text(&error.to_string()));
                    return None;
                },
                _ => tokio::select! { _=runtime.stop.cancelled()=>return None, _=tokio::time::sleep(std::time::Duration::from_millis(150))=>{} },
            }
        }
    }).take_until(stop.cancelled_owned());
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn plans(State(runtime): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(json!(runtime.store.plans()?)))
}
async fn plan(State(runtime): State<Arc<Runtime>>, Path(id): Path<String>) -> ApiResult<Value> {
    let plan = runtime.store.plan(&id)?;
    Ok(Json(
        json!({"plan":plan,"revisions":runtime.store.plan_history(&id)?,"reviews":runtime.store.plan_reviews(&id)?,"review_metrics":runtime.store.plan_review_metrics(&id)?,"completion_audits":runtime.store.completion_audits(&id)?,"evidence":runtime.store.plan_evidence(&id,plan.version)?}),
    ))
}
async fn replan(
    State(runtime): State<Arc<Runtime>>,
    Path(id): Path<String>,
    Json(input): Json<TextInput>,
) -> ApiResult<Value> {
    Ok(Json(runtime.steer_task(&id, &input.text, "web").await?))
}

async fn progress(State(runtime): State<Arc<Runtime>>) -> impl IntoResponse {
    // Watch keeps only the latest bounded snapshot for slow/disconnected clients.
    // Reconnection receives a fresh snapshot; it never replays model requests.
    let stream = WatchStream::new(runtime.subscribe_progress())
        .take_until(runtime.stop.clone().cancelled_owned())
        .map(|snapshot| {
            Ok::<_, std::convert::Infallible>(Event::default().event("progress").data(
                serde_json::to_string(&snapshot).expect("progress contains serializable fields"),
            ))
        });
    Sse::new(stream).keep_alive(KeepAlive::default())
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
    project_id: Option<String>,
}
async fn messages(
    State(runtime): State<Arc<Runtime>>,
    Query(page): Query<Page>,
) -> ApiResult<Value> {
    // Provider replay metadata is internal, never needed by a browser.
    let messages = runtime
        .store
        .conversation(page.before, page.limit.unwrap_or(100))?;
    let mut items = Vec::with_capacity(messages.len());
    for message in messages {
        let binding = match message.task_id.as_deref() {
            Some(id) => serde_json::to_value(runtime.store.task_binding(id)?)?,
            None => Value::Null,
        };
        items.push(json!({"seq":message.seq,"id":message.id,"task_id":message.task_id,"role":message.role,"text":message.text,"origin":message.origin,"created_at":message.created_at,"binding":binding}));
    }
    Ok(Json(json!(items)))
}
#[derive(Deserialize)]
struct TextInput {
    text: String,
}
#[derive(Deserialize)]
struct ChatInput {
    text: String,
    #[serde(default)]
    document_ids: Vec<String>,
    #[serde(default)]
    reply_to_task_id: Option<String>,
    #[serde(default)]
    design_choice: Option<design_choice::DesignChoice>,
}
async fn message(
    State(runtime): State<Arc<Runtime>>,
    Json(input): Json<ChatInput>,
) -> ApiResult<Value> {
    require(
        runtime.model_health_status().await?["can_send"] == true,
        "El modelo no está disponible. Comprueba la conexión antes de enviar mensajes.",
    )?;
    require(input.document_ids.len() <= 20, "Select up to 20 documents")?;
    let mut text = input.text;
    if let Some(choice) = input.design_choice {
        choice.append_to(&mut text)?;
    }
    let mut scope = None;
    for (index, id) in input.document_ids.iter().enumerate() {
        let doc = runtime.store.library_document(id, None)?;
        if index == 0 {
            scope = Some(doc.project_id.clone());
            let project = doc
                .project_id
                .as_deref()
                .map(|id| runtime.store.project(id).map(|p| p.name))
                .transpose()?
                .unwrap_or_else(|| "General".into());
            text.push_str(&format!(
                "\n\nSelected library sources / Fuentes seleccionadas — {}:\n",
                project
            ));
        }
        require(
            scope.as_ref() == Some(&doc.project_id),
            "Select documents from one project at a time",
        )?;
        require(doc.enabled, "Document is disabled")?;
        // Source names are data and rendered as labels, never new instructions.
        let title = doc.title.replace(['[', ']', '\n', '\r'], " ");
        text.push_str(&format!(
            "- [{}](/documents?document={}&revision={})\n",
            title, doc.id, doc.revision
        ));
    }
    Ok(Json(
        runtime
            .conversation_turn(&text, "web", input.reply_to_task_id.as_deref())
            .await?,
    ))
}
async fn projects(State(runtime): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(json!(runtime.store.projects()?)))
}
async fn project(State(runtime): State<Arc<Runtime>>, Path(id): Path<String>) -> ApiResult<Value> {
    Ok(Json(
        json!({"project": runtime.store.project(&id)?, "tasks": runtime.store.tasks_for_project(&id, 100)?}),
    ))
}
async fn create_project(
    State(runtime): State<Arc<Runtime>>,
    Json(input): Json<ProjectInput>,
) -> ApiResult<Value> {
    Ok(Json(json!(runtime.save_project(None, input)?)))
}
async fn update_project(
    State(runtime): State<Arc<Runtime>>,
    Path(id): Path<String>,
    Json(input): Json<ProjectInput>,
) -> ApiResult<Value> {
    Ok(Json(json!(runtime.save_project(Some(&id), input)?)))
}
async fn memories(
    State(runtime): State<Arc<Runtime>>,
    Query(page): Query<Page>,
) -> ApiResult<Value> {
    Ok(Json(json!(runtime.store.memories_scoped(
        page.query.as_deref().unwrap_or(""),
        page.limit.unwrap_or(100),
        page.project_id.as_deref(),
        false,
    )?)))
}
#[derive(Deserialize)]
struct MemoryInput {
    #[serde(default)]
    metadata: Option<fenix_storage::MemoryMetadata>,
    id: Option<String>,
    title: String,
    content: String,
    #[serde(default)]
    project_id: Option<String>,
}
async fn save_memory(
    State(runtime): State<Arc<Runtime>>,
    Json(input): Json<MemoryInput>,
) -> ApiResult<Value> {
    let metadata = input
        .metadata
        .map(
            |metadata| -> anyhow::Result<fenix_storage::MemoryMetadata> {
                let mut value = serde_json::to_value(metadata)?;
                runtime.redactor.value(&mut value);
                Ok(serde_json::from_value(value)?)
            },
        )
        .transpose()?;
    let memory = runtime.store.save_memory_record(
        input.id.as_deref(),
        &runtime.redactor.text(&input.title),
        &runtime.redactor.text(&input.content),
        "web:explicit-owner-edit",
        input.project_id.as_deref(),
        None,
        metadata,
    )?;
    runtime.export_memory()?;
    Ok(Json(json!(memory)))
}
async fn delete_memory(
    State(runtime): State<Arc<Runtime>>,
    Path(id): Path<String>,
    Query(page): Query<Page>,
) -> ApiResult<Value> {
    runtime
        .store
        .delete_memory_scoped(&id, page.project_id.as_deref())?;
    runtime.export_memory()?;
    Ok(Json(json!({"deleted":id})))
}
async fn skills(State(runtime): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(json!(runtime.skills()?)))
}
#[derive(Deserialize, Default)]
struct SkillLanguage {
    language: Option<String>,
}
async fn skill(
    State(runtime): State<Arc<Runtime>>,
    Path(name): Path<String>,
    Query(q): Query<SkillLanguage>,
) -> ApiResult<Value> {
    if let Some(lang) = q.language {
        return Ok(Json(runtime.skill_display(&name, &lang).await?));
    }
    Ok(Json(json!({"content":runtime.skill_load(&name)?})))
}
async fn skill_categories() -> ApiResult<Value> {
    Ok(Json(json!(fenix_agent::skill_categories::catalog()?)))
}
#[derive(Deserialize)]
struct SkillLocalizations {
    #[serde(default)]
    names: Vec<String>,
    #[serde(default)]
    cards: Vec<fenix_agent::skill_localization::Card>,
    language: String,
}
async fn skill_localizations(
    State(runtime): State<Arc<Runtime>>,
    Json(input): Json<SkillLocalizations>,
) -> ApiResult<Value> {
    if !input.names.is_empty() && !input.cards.is_empty() {
        return Err(anyhow::anyhow!("Choose names or cards").into());
    }
    let cards = if input.cards.is_empty() {
        runtime.skill_cards(&input.names, &input.language).await?
    } else {
        runtime
            .localize_cards(&input.cards, &input.language)
            .await?
    };
    Ok(Json(json!({"cards":cards})))
}
#[derive(Deserialize)]
struct SkillPreview {
    content: String,
    language: String,
}
async fn skill_preview(
    State(runtime): State<Arc<Runtime>>,
    Json(input): Json<SkillPreview>,
) -> ApiResult<Value> {
    Ok(Json(
        runtime
            .localize_skill_source(&input.content, &input.language)
            .await?,
    ))
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
    let tasks = runtime.store.tasks(100)?;
    let mut items = Vec::with_capacity(tasks.len());
    for task in tasks {
        let binding = runtime.store.task_binding(&task.id)?;
        let mut item = serde_json::to_value(task)?;
        item["binding"] = serde_json::to_value(binding)?;
        items.push(item);
    }
    Ok(Json(json!(items)))
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
        json!({"task":runtime.store.task(&id)?,"binding":runtime.store.task_binding(&id)?,"subagent":runtime.store.subagent(&id).ok(),"plan":runtime.store.plan_for_task(&id)?,"questions":runtime.store.saved_questions(Some(&id))?,"events":runtime.store.events(&id,page.after.unwrap_or(0),page.limit.unwrap_or(50))?}),
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

async fn knowledge(
    State(runtime): State<Arc<Runtime>>,
    Json(mut input): Json<Value>,
) -> ApiResult<Value> {
    runtime.redactor.value(&mut input);
    Ok(Json(
        runtime.knowledge_action(&input, input["project_id"].as_str())?,
    ))
}
async fn memory_settings(State(runtime): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(
        json!({"settings":runtime.memory_settings()?,"maintenance":runtime.store.get::<Value>("memory_maintenance","latest")?}),
    ))
}
async fn configure_memory(
    State(runtime): State<Arc<Runtime>>,
    Json(mut input): Json<Value>,
) -> ApiResult<Value> {
    let key = input
        .as_object_mut()
        .and_then(|o| o.remove("embedding_api_key"));
    let settings = serde_json::from_value(input)?;
    runtime.configure_memory(settings)?;
    if let Some(key) = key {
        runtime
            .configure_memory_embedding_key(
                key.as_str()
                    .context("Embedding API key must be a string")?
                    .to_owned(),
            )
            .await?;
    }
    Ok(Json(json!({"saved":true})))
}

async fn decision_settings(State(runtime): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(json!({"settings":runtime.decision_settings()?,
        "configured":runtime.store.get::<String>("credentials","decision:jev")?.is_some(),
        "health":runtime.store.get::<Value>("decision_health","latest")?})))
}
async fn configure_decisions(
    State(runtime): State<Arc<Runtime>>,
    Json(mut input): Json<Value>,
) -> ApiResult<Value> {
    let key = input.as_object_mut().and_then(|o| o.remove("api_key"));
    let key = key
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .context("API key must be a string")
        })
        .transpose()?;
    runtime.configure_decisions(serde_json::from_value(input)?, key)?;
    Ok(Json(json!({"saved":true})))
}
