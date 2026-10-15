use super::*;
use fenix_agent::interlocutor::FastModelSettings;
pub(crate) fn routes() -> Router<Arc<Runtime>> {
    Router::new()
        .route("/api/model-health", get(health).post(check_health))
        .route("/api/model-roles", get(roles))
        .route("/api/model-roles/fast", get(status).post(configure))
}
#[derive(Deserialize)]
struct Setup {
    #[serde(flatten)]
    settings: FastModelSettings,
    api_key: Option<String>,
}
async fn status(State(r): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(r.fast_model_status()?))
}
async fn configure(State(r): State<Arc<Runtime>>, Json(s): Json<Setup>) -> ApiResult<Value> {
    Ok(Json(r.configure_fast_model(s.settings, s.api_key).await?))
}

async fn roles(State(r): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(r.model_roles().await?))
}

async fn health(State(r): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(r.model_health_status().await?))
}
async fn check_health(State(r): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(r.check_model_health(true).await?))
}
