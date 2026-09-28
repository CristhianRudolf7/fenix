use super::*;
use fenix_agent::live::{LiveInput, LiveSettings};
pub(crate) fn routes() -> Router<Arc<Runtime>> {
    Router::new()
        .route("/api/live/settings", get(status).post(configure))
        .route("/api/model-roles", get(roles))
        .route("/api/model-roles/fast", get(status).post(configure))
        .route("/api/live/sessions", post(start))
        .route("/api/live/sessions/{id}/turn", post(turn))
        .route("/api/live/sessions/{id}", axum::routing::delete(end))
}
#[derive(Deserialize)]
struct Setup {
    #[serde(flatten)]
    settings: LiveSettings,
    api_key: Option<String>,
}
async fn status(State(r): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(r.live_status()?))
}
async fn configure(State(r): State<Arc<Runtime>>, Json(s): Json<Setup>) -> ApiResult<Value> {
    Ok(Json(r.configure_live(s.settings, s.api_key).await?))
}
async fn start(State(r): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(r.start_live().await?))
}
async fn turn(
    State(r): State<Arc<Runtime>>,
    Path(id): Path<String>,
    Json(input): Json<LiveInput>,
) -> ApiResult<Value> {
    Ok(Json(r.live_turn(&id, input).await?))
}
async fn end(State(r): State<Arc<Runtime>>, Path(id): Path<String>) -> ApiResult<Value> {
    Ok(Json(r.end_live(&id).await?))
}

async fn roles(State(r): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(r.model_roles().await?))
}
