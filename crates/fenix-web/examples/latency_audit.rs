//! Compare cold discovery, warm live verification and ten shared concurrent calls.
use anyhow::{Context, Result};
use fenix_web::{SearchRequest, Web};
use serde_json::{Value, json};
use std::{sync::Arc, time::Instant};
use tokio_util::sync::CancellationToken;

fn memory() -> Value {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let read = |key: &str| {
        status
            .lines()
            .find(|line| line.starts_with(key))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|n| n.parse::<u64>().ok())
    };
    json!({"rss_kib":read("VmRSS:"),"peak_rss_kib":read("VmHWM:")})
}

async fn query(web: Arc<Web>, query: &str, offline: bool) -> Result<Value> {
    let before = web.status().await?.coverage.requests_today;
    let start = Instant::now();
    let response = web
        .search_with(
            SearchRequest {
                query: query.into(),
                discover: !offline,
                read_pages: if offline { 0 } else { 3 },
                ..Default::default()
            },
            &CancellationToken::new(),
        )
        .await?;
    let elapsed = start.elapsed().as_millis();
    let after = web.status().await?.coverage.requests_today;
    eprintln!(
        "{elapsed} ms | {} hits | {} | {query}",
        response.results.len(),
        response.discovery.stop_reason
    );
    let mut value = serde_json::to_value(response)?;
    for hit in value["results"].as_array_mut().context("Missing results")? {
        hit.as_object_mut().unwrap().remove("snippet");
        if let Some(page) = hit.get_mut("page") {
            let count = page["passage"].as_str().unwrap_or_default().chars().count();
            page.as_object_mut().unwrap().remove("passage");
            page["passage_characters"] = json!(count);
        }
    }
    Ok(
        json!({"query":query,"elapsed_ms":elapsed,"http_attempts_during_call":after.saturating_sub(before),"offline":offline,"result":value}),
    )
}

async fn batch(web: Arc<Web>, queries: &[&str]) -> Result<Value> {
    let before = web.status().await?.coverage.requests_today;
    let start = Instant::now();
    let calls =
        futures_util::future::join_all(queries.iter().map(|q| query(web.clone(), q, false))).await;
    let calls = calls.into_iter().collect::<Result<Vec<_>>>()?;
    Ok(
        json!({"wall_ms":start.elapsed().as_millis(),"http_attempts":web.status().await?.coverage.requests_today.saturating_sub(before),"calls":calls}),
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    let output = std::env::args()
        .nth(1)
        .context("Usage: latency_audit <report.json>")?;
    let queries = [
        "React useEffect cleanup",
        "Next.js proxy middleware",
        "Tokio timeout",
        "Python asyncio TaskGroup",
        "SQLite transactions isolation",
        "PostgreSQL window functions",
        "Docker multi stage builds",
        "CSS grid layout",
        "NASA Jupiter moons",
        "WHO dengue symptoms",
    ];
    let dir = tempfile::tempdir()?;
    let initial = memory();
    let web = Web::open(dir.path()).await?;
    let cold = batch(web.clone(), &queries).await?;
    let mut warm_individual = Vec::new();
    for q in [queries[0], queries[3], queries[4], queries[9]] {
        warm_individual.push(query(web.clone(), q, false).await?);
    }
    let warm_parallel = batch(web.clone(), &queries).await?;
    let related = query(web.clone(), "Python pathlib Path", false).await?;
    let offline = query(web.clone(), queries[0], true).await?;
    let report = json!({"performed_at":chrono::Utc::now().to_rfc3339(),"method":"One initially empty persistent temporary corpus. Ten concurrent cold calls, four sequential warm live calls, ten warm concurrent calls, a related new query, then an explicitly offline control. Fresh=true throughout. No model, browser or search provider. Per-call HTTP deltas overlap during batches; use batch totals.","initial_memory":initial,"final_memory":memory(),"cold":cold,"warm_individual":warm_individual,"warm_parallel":warm_parallel,"related_new_query":related,"offline_control":offline,"status":web.status().await?});
    std::fs::write(output, serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}
