//! Opt-in live audit of community-source access and own-index retrieval.
//! Never queries platform search APIs or external search engines.
use anyhow::{Context, Result};
use fenix_web::{Config, SearchRequest, Web};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

async fn query(web: &Web, request: SearchRequest) -> Value {
    let started = Instant::now();
    let input = serde_json::to_value(&request).unwrap();
    match web.search_with(request, &CancellationToken::new()).await {
        Ok(result) => {
            json!({"input":input,"elapsed_ms":started.elapsed().as_millis(),"response":result})
        }
        Err(error) => {
            json!({"input":input,"elapsed_ms":started.elapsed().as_millis(),"error":error.to_string()})
        }
    }
}

fn compact_evidence(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for field in ["passage", "snippet"] {
                if let Some(Value::String(text)) = object.remove(field) {
                    object.insert(format!("{field}_characters"), json!(text.chars().count()));
                }
            }
            for value in object.values_mut() {
                compact_evidence(value);
            }
        }
        Value::Array(values) => {
            for value in values {
                compact_evidence(value);
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
    let paced = std::env::args().any(|argument| argument == "--paced");
    let mut last_hn: Option<Instant> = None;
    let directory = tempfile::tempdir()?;
    std::fs::write(
        directory.path().join("web-search.json"),
        serde_json::to_vec(&Config {
            seeds: vec![],
            requests_per_day: 60,
            ..Config::default()
        })?,
    )?;
    let web = Web::open(directory.path()).await?;
    let started = Instant::now();
    let initial = web.status().await?;
    let mut source_checks = vec![];
    for (url, term, domain) in [
        (
            "https://news.ycombinator.com/rss",
            "Show HN",
            "news.ycombinator.com",
        ),
        (
            "https://news.ycombinator.com/",
            "Show HN",
            "news.ycombinator.com",
        ),
        ("https://www.reddit.com/r/rust/", "Rust", "reddit.com"),
        ("https://www.reddit.com/r/all/.rss", "Rust", "reddit.com"),
        ("https://github.com/rust-lang/rust", "Rust", "github.com"),
    ] {
        if paced
            && domain == "news.ycombinator.com"
            && let Some(last) = last_hn
        {
            tokio::time::sleep(Duration::from_secs(31).saturating_sub(last.elapsed())).await;
        }
        source_checks.push(
            query(
                &web,
                SearchRequest {
                    query: term.into(),
                    domains: vec![domain.into()],
                    source_urls: vec![url.into()],
                    read_pages: 0,
                    ..Default::default()
                },
            )
            .await,
        );
        eprintln!("Checked source: {url}");
        if domain == "news.ycombinator.com" {
            last_hn = Some(Instant::now());
        }
    }
    // Follow actual thread URLs discovered by our extractor, not guessed IDs.
    let db = rusqlite::Connection::open_with_flags(
        directory.path().join("web/index.sqlite3"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?;
    let threads = db.prepare("SELECT url FROM frontier WHERE url LIKE 'https://news.ycombinator.com/item?id=%' ORDER BY rowid LIMIT 3")?
        .query_map([], |row| row.get::<_, String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
    let mut thread_checks = vec![];
    for url in threads {
        if paced && let Some(last) = last_hn {
            tokio::time::sleep(Duration::from_secs(31).saturating_sub(last.elapsed())).await;
        }
        let at = Instant::now();
        match web.read(&url, "", &CancellationToken::new()).await {
            Ok(page) => {
                // Derive a control query from the live thread's actual title.
                let term = page.title.split('|').next().unwrap_or(&page.title)
                    .split_whitespace().take(7).collect::<Vec<_>>().join(" ");
                let fetch_ms = at.elapsed().as_millis();
                let request = SearchRequest {
                    query: term.clone(), domains: vec!["news.ycombinator.com".into()],
                    read_pages: 0, limit: 10, max_per_domain: 10, ..Default::default()
                };
                let retrieved = query(&web, request.clone()).await;
                let found = retrieved["response"]["results"].as_array()
                    .is_some_and(|hits| hits.iter().any(|hit| hit["url"] == page.url));
                let recent = query(&web, SearchRequest {days:Some(30), ..request}).await;
                thread_checks.push(json!({"url":url,"title":page.title,"fetch_ms":fetch_ms,"read":page,
                    "title_control_query":term,"own_thread_retrieved":found,"search":retrieved,"last30days":recent}));
            }
            Err(error) => thread_checks.push(json!({"url":url,"fetch_ms":at.elapsed().as_millis(),"read_error":error.to_string()})),
        }
        last_hn = Some(Instant::now());
        eprintln!("Checked thread: {url}");
    }
    let before = web.status().await?.coverage.requests_today;
    let mut topic_checks = vec![];
    for (term, domain) in [
        ("SQLite", "news.ycombinator.com"),
        ("Rust", "news.ycombinator.com"),
        ("Show HN", "news.ycombinator.com"),
        ("Rust", "reddit.com"),
        ("password manager", "reddit.com"),
        ("Rust", "github.com"),
    ] {
        topic_checks.push(
            query(
                &web,
                SearchRequest {
                    query: term.into(),
                    domains: vec![domain.into()],
                    read_pages: 0,
                    ..Default::default()
                },
            )
            .await,
        );
    }
    let final_status = web.status().await?;
    let delta = final_status.coverage.requests_today.saturating_sub(before);
    let policies = db
        .prepare("SELECT origin, robots FROM hosts WHERE robots IS NOT NULL")?
        .query_map([], |row| {
            Ok(json!({"origin":row.get::<_,String>(0)?,"robots":row.get::<_,String>(1)?}))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut report = json!({
        "performed_at":chrono::Utc::now().to_rfc3339(),
        "method":"Temporary empty corpus; direct original-source reads and native link discovery, no background crawler, no model, no external search service. Tests source access separately from lexical topic coverage and title-based positive controls. Does not claim full platform search or answer accuracy.",
        "paced_hn_requests":paced,"robots_policies":policies,"initial":initial,"source_checks":source_checks,"thread_checks":thread_checks,
        "topic_checks":topic_checks,"topic_query_http_attempts":delta,"final":final_status,
        "elapsed_ms":started.elapsed().as_millis(),
    });
    compact_evidence(&mut report);
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output, serde_json::to_vec_pretty(&report)?)?;
    println!("Saved {}", output.display());
    anyhow::ensure!(delta == 0, "Index-only topic queries made HTTP requests");
    Ok(())
}
