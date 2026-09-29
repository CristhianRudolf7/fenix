//! Private skill management; publication snapshots are never modified here.
use super::*;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Update {
    content: String,
    language: Option<String>,
    revision: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Delete {
    revision: String,
}
#[derive(Deserialize)]
pub(crate) struct SourceQuery {
    language: Option<String>,
}
pub(crate) async fn source(
    State(rt): State<Arc<Runtime>>,
    Path(name): Path<String>,
    Query(query): Query<SourceQuery>,
) -> ApiResult<Value> {
    Ok(Json(
        tokio::task::spawn_blocking(move || match query.language {
            Some(language) => rt.editable_skill_language(&name, &language),
            None => rt.editable_skill(&name),
        })
        .await??,
    ))
}
pub(crate) async fn save(
    State(rt): State<Arc<Runtime>>,
    Path(name): Path<String>,
    Json(input): Json<Update>,
) -> ApiResult<Value> {
    if let Some(language) = input.language {
        return Ok(Json(
            rt.save_skill_language(&name, &input.revision, &input.content, &language)
                .await?,
        ));
    }
    Ok(Json(
        tokio::task::spawn_blocking(move || {
            rt.save_skill_source(&name, &input.revision, &input.content)
        })
        .await??,
    ))
}
pub(crate) async fn remove(
    State(rt): State<Arc<Runtime>>,
    Path(name): Path<String>,
    Json(input): Json<Delete>,
) -> ApiResult<Value> {
    tokio::task::spawn_blocking(move || rt.delete_private_skill(&name, &input.revision)).await??;
    Ok(Json(json!({"deleted":true})))
}
