//! Fénix Sentinel and the shared skill exchange. No third-party scanner code or runtime.
mod reviewer;
mod scanner;
mod store;
#[cfg(test)]
mod tests;

use anyhow::{Context, Result};
use axum::{
    Extension, Json, Router,
    extract::{Path, Query, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use fenix_agent::{Runtime, skills::bundle::Bundle};
use reviewer::Reviewer;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    path::Path as FsPath,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Clone)]
pub(super) struct Owner(pub String);
pub(super) struct Service {
    store: store::Store,
    reviewer: Reviewer,
    requests: Mutex<HashMap<String, (Instant, u32)>>,
}
impl Service {
    pub(super) fn hosted(database: &FsPath, config: Option<&FsPath>) -> Result<Arc<Self>> {
        Self::open(database, Reviewer::hosted(config)?)
    }
    fn open(database: &FsPath, reviewer: Reviewer) -> Result<Arc<Self>> {
        // Database/WAL live in a private host directory, outside tenant mounts.
        let service = Arc::new(Self {
            store: store::Store::open(database)?,
            reviewer,
            requests: Mutex::new(HashMap::new()),
        });
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(database, std::fs::Permissions::from_mode(0o600))?;
        for _ in 0..2 {
            let weak = Arc::downgrade(&service);
            tokio::spawn(async move {
                loop {
                    let Some(service) = weak.upgrade() else {
                        break;
                    };
                    if let Ok(Some(job)) = service.store.claim() {
                        let review = tokio::time::timeout(
                            Duration::from_secs(600),
                            service.reviewer.review(&job.bundle),
                        );
                        tokio::pin!(review);
                        let mut cancellation_check =
                            tokio::time::interval(Duration::from_millis(500));
                        let result = loop {
                            tokio::select! {
                                result=&mut review => break Some(result),
                                _=cancellation_check.tick() => {
                                    if !service.store.is_current(&job.id,job.attempt).unwrap_or(false) {break None;}
                                }
                            }
                        };
                        let Some(result) = result else {
                            continue;
                        };
                        match result {
                            Ok(Ok(outcome)) => {
                                let _ = service.store.finish(
                                    &job.id,
                                    job.attempt,
                                    outcome.status,
                                    outcome.candidate.as_ref(),
                                    &outcome.report,
                                );
                            }
                            // Never store provider errors: they may contain credentials or raw skill text.
                            _ => {
                                let _=service.store.finish(&job.id,job.attempt,"failed",None,&json!({"policy":scanner::POLICY,"reason":"Review failed or timed out. Nothing was published. Check the operator's Nemotron configuration or retry later."}));
                            }
                        }
                    }
                    drop(service);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });
        }
        Ok(service)
    }
    pub(super) fn routes(self: Arc<Self>) -> Router {
        Router::new()
            .route("/api/skill-exchange/status", get(status))
            .route("/api/skill-exchange/mine", get(mine))
            .route("/api/skill-exchange/catalog", get(catalog))
            .route("/api/skill-exchange/publications", post(submit))
            .route("/api/skill-exchange/publications/{id}", get(report))
            .route(
                "/api/skill-exchange/publications/{id}/visibility",
                post(visibility),
            )
            .route("/api/skill-exchange/publications/{id}/retry", post(retry))
            .route("/api/skill-exchange/publications/{id}/bundle", get(bundle))
            .layer(middleware::from_fn_with_state(self.clone(), rate_limit))
            .with_state(self)
    }
}
pub(super) fn local_routes(runtime: &Arc<Runtime>) -> Result<Router> {
    let service = Service::open(
        &runtime.data_dir.join("skill-exchange.sqlite"),
        Reviewer::Local(Arc::downgrade(runtime)),
    )?;
    Ok(service.routes().layer(Extension(Owner("local".into()))))
}
async fn rate_limit(State(service): State<Arc<Service>>, request: Request, next: Next) -> Response {
    let Some(owner) = request.extensions().get::<Owner>() else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let allowed = (|| {
        let mut buckets = service.requests.lock().ok()?;
        buckets.retain(|_, (at, _)| at.elapsed() < Duration::from_secs(60));
        if buckets.len() >= 10_000 && !buckets.contains_key(&owner.0) {
            return None;
        }
        let bucket = buckets
            .entry(owner.0.clone())
            .or_insert((Instant::now(), 0));
        bucket.1 += 1;
        (bucket.1 <= 120).then_some(())
    })()
    .is_some();
    if !allowed {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [("retry-after", "60")],
            Json(json!({"error":"Too many marketplace requests; try again in one minute"})),
        )
            .into_response();
    }
    next.run(request).await
}
struct Error(anyhow::Error);
impl<E: Into<anyhow::Error>> From<E> for Error {
    fn from(e: E) -> Self {
        Self(e.into())
    }
}
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let message = self.0.to_string();
        let status = if message.contains("quota") {
            StatusCode::TOO_MANY_REQUESTS
        } else if message.contains("no rows") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::BAD_REQUEST
        };
        (status, Json(json!({"error":message}))).into_response()
    }
}
type Api<T> = std::result::Result<Json<T>, Error>;
#[derive(Deserialize, Default)]
struct Search {
    #[serde(default)]
    q: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    offset: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Submission {
    bundle: Bundle,
    category: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Visibility {
    public: bool,
}
#[derive(Deserialize, Default)]
struct Download {
    #[serde(default)]
    original: bool,
}
async fn status(State(s): State<Arc<Service>>) -> Json<Value> {
    Json(json!({"name":"Fénix Sentinel","configured":s.reviewer.ready(),"policy":scanner::POLICY}))
}
async fn mine(
    State(s): State<Arc<Service>>,
    Extension(owner): Extension<Owner>,
    Query(q): Query<Search>,
) -> Api<Value> {
    Ok(Json(json!({"items":s.store.mine(&owner.0,q.offset)?})))
}
async fn catalog(State(s): State<Arc<Service>>, Query(q): Query<Search>) -> Api<Value> {
    if q.q.len() > 200 || q.category.len() > 40 {
        return Err(anyhow::anyhow!("Search is too long").into());
    }
    Ok(Json(
        json!({"items":s.store.catalog(&q.q,&q.category,q.offset)?}),
    ))
}
async fn submit(
    State(s): State<Arc<Service>>,
    Extension(owner): Extension<Owner>,
    Json(input): Json<Submission>,
) -> Api<Value> {
    Ok(Json(serde_json::to_value(s.store.submit(
        &owner.0,
        &input.bundle,
        &input.category,
    )?)?))
}
async fn report(
    State(s): State<Arc<Service>>,
    Extension(owner): Extension<Owner>,
    Path(id): Path<String>,
) -> Api<Value> {
    Ok(Json(
        json!({"publication":s.store.get(&owner.0,&id)?,"report":s.store.report(&owner.0,&id)?}),
    ))
}
async fn visibility(
    State(s): State<Arc<Service>>,
    Extension(owner): Extension<Owner>,
    Path(id): Path<String>,
    Json(input): Json<Visibility>,
) -> Api<Value> {
    Ok(Json(serde_json::to_value(s.store.visibility(
        &owner.0,
        &id,
        input.public,
    )?)?))
}
async fn retry(
    State(s): State<Arc<Service>>,
    Extension(owner): Extension<Owner>,
    Path(id): Path<String>,
) -> Api<Value> {
    s.store.retry(&owner.0, &id)?;
    Ok(Json(serde_json::to_value(s.store.get(&owner.0, &id)?)?))
}
async fn bundle(
    State(s): State<Arc<Service>>,
    Extension(owner): Extension<Owner>,
    Path(id): Path<String>,
    Query(q): Query<Download>,
) -> Api<Value> {
    Ok(Json(serde_json::to_value(
        s.store.bundle(&owner.0, &id, q.original)?,
    )?))
}

pub(super) fn tenant_routes() -> Router<Arc<Runtime>> {
    Router::new()
        .route("/api/skills/{name}/bundle", get(export))
        .route("/api/skills/install", post(install))
}
async fn export(
    State(rt): State<Arc<Runtime>>,
    Path(name): Path<String>,
) -> crate::ApiResult<Value> {
    Ok(Json(serde_json::to_value(rt.skill_bundle(&name)?)?))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Installation {
    bundle: Bundle,
    name: String,
}
async fn install(
    State(rt): State<Arc<Runtime>>,
    Json(input): Json<Installation>,
) -> crate::ApiResult<Value> {
    rt.install_skill_bundle(&input.bundle, &input.name)
        .context("Cannot install private copy")?;
    Ok(Json(json!({"name":input.name,"private":true})))
}
