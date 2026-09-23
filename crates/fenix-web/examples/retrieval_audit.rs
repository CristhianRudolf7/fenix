//! Reproducible randomly sampled query-only live searches through the real engine.
//! No answer URLs, model calls, external search providers or production data.
use anyhow::{Context, Result};
use fenix_web::{Config, SearchRequest, Web};
use serde_json::{Value, json};
use std::{sync::Arc, time::Instant};
use tokio_util::sync::CancellationToken;

fn memory() -> Value {
    let raw = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let get = |key: &str| {
        raw.lines()
            .find(|l| l.starts_with(key))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|v| v.parse::<u64>().ok())
    };
    json!({"rss_kib":get("VmRSS:"),"peak_rss_kib":get("VmHWM:")})
}

async fn run(web: Arc<Web>, query: &str) -> Value {
    let at = Instant::now();
    let result = web
        .search_with(
            SearchRequest {
                query: query.into(),
                discover: true,
                ..Default::default()
            },
            &CancellationToken::new(),
        )
        .await;
    let elapsed_ms = at.elapsed().as_millis();
    match result {
        Ok(result) => {
            eprintln!("{elapsed_ms} ms | {} hits | {query}", result.results.len());
            json!({"query":query,"elapsed_ms":elapsed_ms,"result":result})
        }
        Err(error) => json!({"query":query,"elapsed_ms":elapsed_ms,"error":error.to_string()}),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let output = args
        .next()
        .context("Usage: retrieval_audit <report.json> [queries.json]")?;
    let mut queries: Vec<String> = [
        "React useEffect cleanup",
        "Next.js proxy middleware",
        "Tokio timeout",
        "Python asyncio TaskGroup",
        "SQLite transactions isolation",
        "PostgreSQL window functions",
        "Rust ownership borrowing",
        "Docker multi stage builds",
        "Kubernetes deployment rolling update",
        "Git rebase conflicts",
        "TypeScript generics",
        "CSS grid layout",
        "NASA Mars atmosphere",
        "NASA Jupiter moons",
        "NASA solar eclipse",
        "WHO malaria prevention",
        "WHO dengue symptoms",
        "World Bank poverty",
        "IPCC climate change",
        "Context7 API context",
        "Busca documentación actualizada de React useEffect cleanup",
        "Python pathlib Path",
        "receta peruana de ají de gallina",
        "historia del imperio inca",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let supplied = args.next();
    if let Some(path) = &supplied {
        queries = serde_json::from_slice(&std::fs::read(path)?)?;
    } else {
        // Fixed seed, recorded for reproduction; select before seeing results.
        let mut state = 20260920_u64;
        for i in (1..queries.len()).rev() {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            queries.swap(i, (state % (i + 1) as u64) as usize);
        }
        queries.truncate(12);
    }
    let at = Instant::now();
    let initial = memory();
    let mut cold = Vec::new();
    for batch in queries.chunks(3) {
        let results = futures_util::future::join_all(batch.iter().map(|query| async {
            let dir = tempfile::tempdir()?;
            let web = Web::open(dir.path()).await?;
            Ok::<_, anyhow::Error>(run(web, query).await)
        }))
        .await;
        for result in results {
            cold.push(result?);
        }
    }
    // A separate installation handles ten simultaneous agents sharing the same
    // index and per-origin pacing; these queries may share discovered pages.
    let dir = tempfile::tempdir()?;
    let config = Config {
        requests_per_day: 2_000,
        ..Default::default()
    };
    std::fs::write(
        dir.path().join("web-search.json"),
        serde_json::to_vec(&config)?,
    )?;
    let web = Web::open(dir.path()).await?;
    let batch_at = Instant::now();
    let parallel = futures_util::future::join_all(
        queries
            .iter()
            .cycle()
            .take(10)
            .map(|query| run(web.clone(), query)),
    )
    .await;
    let report = json!({"performed_at":chrono::Utc::now().to_rfc3339(),"sampling_seed":20260920,"custom_queries":supplied.is_some(),"method":"Query-only searches, fresh=true, discover=true, no answer URLs. Cold cases have isolated empty corpora, three at a time. Separate cold shared installation executes ten concurrent calls. Bootstrap source catalog is enabled. Nonempty results alone are not a relevance pass. No model or Chromium included in memory measurements.","initial_memory":initial,"final_memory":memory(),"wall_ms":at.elapsed().as_millis(),"cold":cold,"parallel_wall_ms":batch_at.elapsed().as_millis(),"parallel":parallel,"status":web.status().await?});
    std::fs::write(output, serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}
