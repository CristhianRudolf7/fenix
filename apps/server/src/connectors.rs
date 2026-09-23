use super::*;
use axum::{http::HeaderMap, response::Html};

pub(crate) fn routes() -> Router<Arc<Runtime>> {
    Router::new()
        .route("/api/connectors", get(list).post(save))
        .route("/api/connectors/{id}/test", post(test))
        .route("/api/connectors/{id}/forget", post(forget))
        .route("/api/connectors/{id}/oauth", post(start))
        .route(
            "/api/connectors/oauth/callback",
            get(oauth_landing).post(callback),
        )
}
// A provider's cross-site redirect does not carry the Strict session cookie.
// Serve only a static landing page; its same-origin POST performs the exchange
// through the authenticated owner proxy with one-use OAuth state and PKCE.
pub(crate) async fn oauth_landing() -> Response {
    let nonce = uuid::Uuid::new_v4().simple().to_string();
    let html = include_str!("../resources/oauth-callback.html").replace("{{nonce}}", &nonce);
    let mut response = Html(html).into_response();
    response.headers_mut().insert(
        "content-security-policy",
        format!("default-src 'none'; script-src 'nonce-{nonce}'; style-src 'unsafe-inline'; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'").parse().unwrap(),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
        .headers_mut()
        .insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    response
}
async fn list(State(r): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(
        json!({"catalog":fenix_connectors::catalog(),"connections":r.connectors.list()?,"telegram":r.telegram_settings()?.map(|s|json!({"enabled":s.enabled}))}),
    ))
}
async fn save(
    State(r): State<Arc<Runtime>>,
    Json(input): Json<fenix_connectors::ConnectionInput>,
) -> ApiResult<Value> {
    if let Some(fields) = input.credentials.as_object() {
        for value in fields.values().filter_map(Value::as_str) {
            r.redactor.add(value);
        }
    }
    Ok(Json(r.connectors.save(input)?))
}
async fn test(State(r): State<Arc<Runtime>>, Path(id): Path<String>) -> ApiResult<Value> {
    let settings = r.settings()?.context("Configure the workspace first")?;
    r.validate_connection_workspace(&settings.workspace)?;
    Ok(Json(r.connectors.test(&id, &settings.workspace).await?))
}
async fn forget(State(r): State<Arc<Runtime>>, Path(id): Path<String>) -> ApiResult<Value> {
    r.connectors.forget(&id)?;
    Ok(Json(json!({"disconnected":true})))
}
async fn start(
    State(r): State<Arc<Runtime>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Value> {
    let origin = headers
        .get(header::ORIGIN)
        .context("Origin header required")?
        .to_str()?;
    Ok(Json(
        r.connectors
            .oauth_start(&id, &format!("{origin}/api/connectors/oauth/callback"))
            .await?,
    ))
}
#[derive(Deserialize)]
struct Callback {
    state: Option<String>,
    code: Option<String>,
    error: Option<String>,
}
async fn callback(State(r): State<Arc<Runtime>>, Json(query): Json<Callback>) -> ApiResult<Value> {
    if query.error.is_some() {
        return Ok(Json(
            json!({"redirect":"/integrations?connection=declined"}),
        ));
    }
    let id = r
        .connectors
        .oauth_finish(
            query.state.as_deref().context("OAuth state required")?,
            query.code.as_deref().context("OAuth code required")?,
        )
        .await?;
    Ok(Json(
        json!({"redirect":format!("/integrations?connected={id}")}),
    ))
}
