use super::*;

pub(crate) fn routes() -> Router<Arc<Runtime>> {
    Router::new()
        .route(
            "/browser.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("../../web/browser.js"),
                )
            }),
        )
        .route("/api/browser/status", get(status))
        .route("/api/browser/profiles", post(profile))
        .route("/api/browser/command", post(command))
        .route("/api/browser/frame", get(frame))
        .route("/api/browser/artifacts/{profile}/{name}", get(artifact))
        .route("/api/browser/credentials", post(credentials))
        .route("/api/browser/downloads", get(downloads))
        .route("/api/browser/downloads/{profile}/{id}", get(download))
}
async fn status(State(r): State<Arc<Runtime>>) -> ApiResult<Value> {
    Ok(Json(r.browser.status().await?))
}
#[derive(Deserialize)]
struct Profile {
    name: String,
    #[serde(default)]
    allow_agent: bool,
}
async fn profile(State(r): State<Arc<Runtime>>, Json(p): Json<Profile>) -> ApiResult<Value> {
    Ok(Json(r.browser.profile(&p.name, p.allow_agent).await?))
}
#[derive(Deserialize)]
struct Command {
    tool: String,
    args: Value,
}
async fn command(State(r): State<Arc<Runtime>>, Json(c): Json<Command>) -> ApiResult<Value> {
    // Direct owner controls bypass the model and its conversation/tool journal.
    if c.tool == "browser_act" && c.args["action"] == "upload" {
        return Err(anyhow::anyhow!("Use an authorized workspace upload through the agent").into());
    }
    let cancel = tokio_util::sync::CancellationToken::new();
    Ok(Json(
        r.browser
            .execute(&c.tool, &c.args, true, "owner", &cancel)
            .await?,
    ))
}
#[derive(Deserialize)]
struct Frame {
    profile: String,
    tab: Option<String>,
}
async fn frame(
    State(r): State<Arc<Runtime>>,
    Query(q): Query<Frame>,
) -> Result<Response, ApiError> {
    Ok((
        [(header::CONTENT_TYPE, "image/png")],
        r.browser.screenshot(&q.profile, q.tab.as_deref()).await?,
    )
        .into_response())
}
async fn artifact(
    State(r): State<Arc<Runtime>>,
    Path((profile, name)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    Ok((
        [(header::CONTENT_TYPE, "image/png")],
        r.browser.artifact(&profile, &name)?,
    )
        .into_response())
}
async fn credentials(State(r): State<Arc<Runtime>>, Json(args): Json<Value>) -> ApiResult<Value> {
    Ok(Json(r.browser.credentials(args).await?))
}

async fn downloads(State(r): State<Arc<Runtime>>, Query(q): Query<Frame>) -> ApiResult<Value> {
    Ok(Json(r.browser.downloads(&q.profile)?))
}
async fn download(
    State(r): State<Arc<Runtime>>,
    Path((profile, id)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    use tokio::io::AsyncReadExt;
    let file = tokio::fs::File::open(r.browser.download_path(&profile, &id)?).await?;
    let stream = futures_util::stream::try_unfold(file, |mut file| async move {
        let mut buffer = vec![0; 65536];
        let n = file.read(&mut buffer).await?;
        buffer.truncate(n);
        Ok::<_, std::io::Error>(if n == 0 { None } else { Some((buffer, file)) })
    });
    Ok((
        [
            (header::CONTENT_TYPE, "application/octet-stream".to_owned()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{id}\""),
            ),
        ],
        axum::body::Body::from_stream(stream),
    )
        .into_response())
}
