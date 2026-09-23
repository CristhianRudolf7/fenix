use super::*;

pub(crate) fn routes() -> Router<Arc<Runtime>> {
    Router::new()
        .route("/api/integrations", get(list))
        .route("/api/integrations/whatsapp/connect", post(connect))
        .route("/api/integrations/whatsapp/pause", post(pause))
        .route("/api/integrations/whatsapp/forget", post(forget))
        .route("/api/integrations/whatsapp/qr", get(qr))
}
async fn list(State(r): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(
        json!({"connections":[r.browser.whatsapp_status().await?]}),
    ))
}
async fn connect(State(r): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(r.browser.whatsapp_connect().await?))
}
async fn pause(State(r): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(r.browser.whatsapp_pause().await?))
}
async fn forget(State(r): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(r.browser.whatsapp_forget().await?))
}
async fn qr(State(r): State<Arc<Runtime>>) -> Result<Response, ApiError> {
    let bytes = r.browser.whatsapp_qr().await?;
    Ok((
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, "no-store, private, max-age=0"),
            (header::PRAGMA, "no-cache"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        bytes,
    )
        .into_response())
}
