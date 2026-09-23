//! Opt-in real-source audit; no external search engines or account sessions.
use anyhow::{Context, Result};
use fenix_web::{Config, SearchRequest, Web};
use serde_json::{Value, json};
use std::{path::PathBuf, time::Instant};
use tokio_util::sync::CancellationToken;

fn memory_kib() -> Value {
    let value = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let get = |key: &str| {
        value
            .lines()
            .find(|line| line.starts_with(key))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|n| n.parse::<u64>().ok())
    };
    json!({"rss_kib":get("VmRSS:"),"peak_rss_kib":get("VmHWM:")})
}

fn compact(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for name in ["passage", "snippet"] {
                if let Some(Value::String(text)) = object.remove(name) {
                    object.insert(format!("{name}_characters"), json!(text.chars().count()));
                }
            }
            for value in object.values_mut() {
                compact(value);
            }
        }
        Value::Array(values) => {
            for value in values {
                compact(value);
            }
        }
        _ => {}
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let output = PathBuf::from(
        std::env::args()
            .nth(1)
            .context("Supply an output JSON path")?,
    );
    let directory = tempfile::tempdir()?;
    std::fs::write(
        directory.path().join("web-search.json"),
        serde_json::to_vec(&Config {
            seeds: vec![],
            requests_per_day: 80,
            ..Default::default()
        })?,
    )?;
    let web = Web::open(directory.path()).await?;
    let initial_memory = memory_kib();
    let defaults = vec![
        SearchRequest {
            query: "ownership borrowing".into(),
            domains: vec!["doc.rust-lang.org".into()],
            source_urls: vec!["https://doc.rust-lang.org/book/".into()],
            discover: true,
            ..Default::default()
        },
        SearchRequest {
            query: "Show HN".into(),
            domains: vec!["news.ycombinator.com".into()],
            days: Some(30),
            discover: true,
            ..Default::default()
        },
        SearchRequest {
            query: "Rust".into(),
            domains: vec!["reddit.com".into()],
            days: Some(30),
            discover: true,
            ..Default::default()
        },
        SearchRequest {
            query: "transaction".into(),
            domains: vec!["sqlite.org".into()],
            discover: true,
            ..Default::default()
        },
    ];
    // Optional input supports focused follow-up audits without repeating other sites.
    let requests: Vec<SearchRequest> = if let Some(input) = std::env::args().nth(2) {
        anyhow::ensure!(
            std::fs::metadata(&input)?.len() <= 64 * 1024,
            "Request file exceeds 64 KiB"
        );
        serde_json::from_slice(&std::fs::read(input)?)?
    } else {
        defaults
    };
    anyhow::ensure!((1..=10).contains(&requests.len()), "Use 1–10 audit queries");
    let started = Instant::now();
    let results = futures_util::future::join_all(requests.iter().map(|request| {
        let web = web.clone();
        async move {
            let at = Instant::now();
            let result = match web
                .search_with(request.clone(), &CancellationToken::new())
                .await
            {
                Ok(response) => json!({"response":response}),
                Err(error) => json!({"error":error.to_string()}),
            };
            eprintln!(
                "{} completed in {} ms",
                request.query,
                at.elapsed().as_millis()
            );
            json!({"request":request,"elapsed_ms":at.elapsed().as_millis(),"result":result})
        }
    }))
    .await;
    let wall_ms = started.elapsed().as_millis();
    let before_cache = web.status().await?.coverage.requests_today;
    let mut cached = vec![];
    for request in requests {
        let request = SearchRequest {
            discover: false,
            read_pages: 0,
            source_urls: vec![],
            ..request
        };
        let at = Instant::now();
        let result = web
            .search_with(request.clone(), &CancellationToken::new())
            .await?;
        cached.push(json!({"query":request.query,"elapsed_us":at.elapsed().as_micros(),"hits":result.results.len()}));
    }
    let final_status = web.status().await?;
    let cache_http_requests = final_status.coverage.requests_today - before_cache;
    let mut report = json!({
        "performed_at":chrono::Utc::now().to_rfc3339(),
        "method":"Simultaneous cold-corpus queries; only original sites and discovered links. No model, no background crawler, no provider. Rust book URL is a known entry listing, not a known answer URL. Cache phase explicitly disables discovery and reads. This measures bounded discovery, not global search recall or model answer quality.",
        "initial_memory":initial_memory,"final_memory":memory_kib(),"wall_ms":wall_ms,
        "searches":results,"offline_repeats":cached,"offline_http_requests":cache_http_requests,"final_status":final_status
    });
    compact(&mut report);
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output, serde_json::to_vec_pretty(&report)?)?;
    anyhow::ensure!(
        cache_http_requests == 0,
        "Offline repeat made HTTP requests"
    );
    println!("Saved {}", output.display());
    Ok(())
}
