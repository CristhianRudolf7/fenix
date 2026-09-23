use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::time::Duration;
pub const MODEL: &str = "text-embedding-3-small";
pub const DIMENSIONS: usize = 1536;
pub const IDENTITY: &str = "openai:text-embedding-3-small:1536:v1";

/// Retry only provider throttling/transient transport failures, never invalid credentials/data.
pub async fn request(
    endpoint: &str,
    key: &str,
    organization: Option<&str>,
    project: Option<&str>,
    texts: &[String],
) -> Result<(Vec<Vec<f32>>, u64)> {
    for attempt in 0..3 {
        match request_once(endpoint, key, organization, project, texts).await {
            Ok(value) => return Ok(value),
            Err(error) => {
                let transient = error.downcast_ref::<reqwest::Error>().is_some_and(|e| {
                    e.is_timeout()
                        || e.is_connect()
                        || e.status()
                            .is_some_and(|s| s.as_u16() == 429 || s.is_server_error())
                });
                if !transient || attempt == 2 {
                    return Err(error);
                }
                tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
            }
        }
    }
    unreachable!()
}

async fn request_once(
    endpoint: &str,
    key: &str,
    organization: Option<&str>,
    project: Option<&str>,
    texts: &[String],
) -> Result<(Vec<Vec<f32>>, u64)> {
    ensure!(
        !texts.is_empty()
            && texts.len() <= 16
            && texts
                .iter()
                .all(|t| !t.trim().is_empty() && t.chars().count() <= 2400 && t.len() <= 8000),
        "Invalid embedding batch"
    );
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(45))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let mut request = client.post(endpoint).bearer_auth(key).json(
        &json!({"model":MODEL,"input":texts,"encoding_format":"float","dimensions":DIMENSIONS}),
    );
    if let Some(v) = organization {
        request = request.header("OpenAI-Organization", v);
    }
    if let Some(v) = project {
        request = request.header("OpenAI-Project", v);
    }
    let mut response = request.send().await?.error_for_status()?;
    let mut bytes = vec![];
    while let Some(chunk) = response.chunk().await? {
        ensure!(
            bytes.len() + chunk.len() <= 8_000_000,
            "Embedding response too large"
        );
        bytes.extend_from_slice(&chunk);
    }
    let value: Value = serde_json::from_slice(&bytes)?;
    decode(&value, texts.len())
}
pub fn decode(value: &Value, count: usize) -> Result<(Vec<Vec<f32>>, u64)> {
    let rows = value["data"].as_array().context("Missing embedding data")?;
    ensure!(rows.len() == count, "Embedding count mismatch");
    let mut vectors = vec![None; count];
    for row in rows {
        let index = row["index"].as_u64().context("Missing embedding index")? as usize;
        ensure!(
            index < count && vectors[index].is_none(),
            "Invalid or duplicate embedding index"
        );
        let vector: Vec<f32> = serde_json::from_value(row["embedding"].clone())?;
        ensure!(
            vector.len() == DIMENSIONS
                && vector.iter().all(|v| v.is_finite())
                && vector.iter().any(|v| *v != 0.0),
            "Invalid embedding dimensions or values"
        );
        vectors[index] = Some(vector);
    }
    let tokens = value["usage"]["total_tokens"]
        .as_u64()
        .context("Missing embedding usage")?;
    Ok((vectors.into_iter().map(Option::unwrap).collect(), tokens))
}
