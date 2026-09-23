//! Live checks against original public pages, never a hosted search service.
//! Explicit source URLs seed a temporary corpus; this does not measure global discovery.
use anyhow::Result;
use fenix_web::{Config, SearchRequest, Web};
use serde_json::json;
use std::{path::PathBuf, time::Instant};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<()> {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/verification/independent-search.json"));
    let directory = tempfile::tempdir()?;
    std::fs::write(
        directory.path().join("web-search.json"),
        serde_json::to_vec(&Config {
            seeds: vec![],
            requests_per_day: 100,
            ..Config::default()
        })?,
    )?;
    let web = Web::open(directory.path()).await?;
    let cancel = CancellationToken::new();
    let cases = [
        (
            "Rust ownership",
            "ownership",
            "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html",
            "doc.rust-lang.org",
        ),
        (
            "Python concurrency",
            "asyncio",
            "https://docs.python.org/3/library/asyncio-task.html",
            "docs.python.org",
        ),
        (
            "SQLite durability",
            "WAL",
            "https://www.sqlite.org/wal.html",
            "sqlite.org",
        ),
        (
            "Web layout",
            "grid",
            "https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_grid_layout",
            "developer.mozilla.org",
        ),
        (
            "Astronomy",
            "exoplanets",
            "https://science.nasa.gov/exoplanets/",
            "science.nasa.gov",
        ),
        (
            "Public health",
            "malaria",
            "https://www.who.int/news-room/fact-sheets/detail/malaria",
            "who.int",
        ),
        (
            "Community front page",
            "Show HN",
            "https://news.ycombinator.com/",
            "news.ycombinator.com",
        ),
        (
            "Public Reddit page",
            "Rust",
            "https://www.reddit.com/r/rust/",
            "reddit.com",
        ),
    ];
    let empty = web.status().await?;
    let start = Instant::now();
    let mut rows = vec![];
    // A small number of parallel original-source reads respects shared admission.
    for chunk in cases.chunks(3) {
        let reads = chunk.iter().map(|(name, query, url, domain)| {
            let web = web.clone(); let cancel = cancel.clone();
            async move {
                let at = Instant::now();
                let result = web.search_with(SearchRequest {
                    query: (*query).into(), read_pages: 0,
                    source_urls: vec![(*url).into()], domains: vec![(*domain).into()],
                    ..Default::default()
                }, &cancel).await;
                match result {
                    Ok(result) => json!({"case":name,"url":url,"elapsed_ms":at.elapsed().as_millis(),"matching_results":result.results.len(),"response":result}),
                    Err(error) => json!({"case":name,"url":url,"elapsed_ms":at.elapsed().as_millis(),"error":error.to_string()}),
                }
            }
        });
        rows.extend(futures_util::future::join_all(reads).await);
    }
    let before = web.status().await?.coverage.requests_today;
    let mut offline = vec![];
    for (name, query, _, domain) in &cases {
        let at = Instant::now();
        let result = web
            .search_with(
                SearchRequest {
                    query: (*query).into(),
                    read_pages: 0,
                    domains: vec![(*domain).into()],
                    ..Default::default()
                },
                &cancel,
            )
            .await?;
        offline.push(json!({"case":name,"elapsed_us":at.elapsed().as_micros(),"matching_results":result.results.len()}));
    }
    let after = web.status().await?;
    let report = json!({
        "performed_at":chrono::Utc::now().to_rfc3339(),
        "method":"Eight explicit known original-source URLs, temporary empty corpus, no crawler, no LLM, no external search provider. This checks ingestion and local retrieval, not global coverage or answer correctness. Blocked sources remain reported as failures.",
        "initial":empty,"source_queries":rows,"offline_queries":offline,
        "offline_http_attempts":after.coverage.requests_today.saturating_sub(before),
        "elapsed_ms":start.elapsed().as_millis(),"final":after,
    });
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output, serde_json::to_vec_pretty(&report)?)?;
    println!("Saved {}", output.display());
    anyhow::ensure!(
        report["offline_http_attempts"] == 0,
        "Index-only queries made HTTP requests"
    );
    Ok(())
}
