use super::*;
use fenix_agent::workflows::WorkflowInput;

pub(crate) fn routes() -> Router<Arc<Runtime>> {
    Router::new()
        .route("/api/workflows", get(list).post(save))
        .route("/api/workflows/{id}", get(get_flow))
        .route("/api/workflows/{id}/run", post(run))
        .route("/api/workflows/{id}/runs", get(runs))
        .route("/api/workflow-runs/{id}", get(get_run))
        .route("/api/workflow-runs/{id}/resume", post(resume))
        .route("/api/workflow-runs/{id}/cancel", post(cancel))
}
async fn list(State(r): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(json!(r.workflows()?)))
}
async fn get_flow(State(r): State<Arc<Runtime>>, Path(id): Path<String>) -> ApiResult<Value> {
    Ok(Json(json!(r.workflow(&id)?)))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Save {
    id: Option<String>,
    expected_version: Option<u64>,
    workflow: WorkflowInput,
}
async fn save(State(r): State<Arc<Runtime>>, Json(input): Json<Save>) -> ApiResult<Value> {
    Ok(Json(json!(
        r.save_workflow(input.id.as_deref(), input.expected_version, input.workflow)
            .await?
    )))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Start {
    #[serde(default)]
    test_mode: bool,
}
async fn run(
    State(r): State<Arc<Runtime>>,
    Path(id): Path<String>,
    Json(input): Json<Start>,
) -> ApiResult<Value> {
    Ok(Json(json!(
        r.enqueue_workflow(&id, input.test_mode, None).await?
    )))
}
async fn runs(State(r): State<Arc<Runtime>>, Path(id): Path<String>) -> ApiResult<Value> {
    r.workflow(&id)?;
    Ok(Json(json!(r.workflow_runs(Some(&id))?)))
}
async fn get_run(State(r): State<Arc<Runtime>>, Path(id): Path<String>) -> ApiResult<Value> {
    Ok(Json(json!(r.workflow_run(&id)?)))
}
async fn resume(State(r): State<Arc<Runtime>>, Path(id): Path<String>) -> ApiResult<Value> {
    Ok(Json(json!(r.resume_workflow(&id).await?)))
}
async fn cancel(State(r): State<Arc<Runtime>>, Path(id): Path<String>) -> ApiResult<Value> {
    Ok(Json(json!(r.cancel_workflow(&id).await?)))
}
