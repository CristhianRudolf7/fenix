//! Live documentation audit of the existing engine, with disposable isolated corpora.
//! No production configuration, agent provider or authenticated session is used.
use anyhow::{Context, Result};
use fenix_web::{Config, SearchRequest, Web};
use serde_json::{Value, json};
use std::{path::Path, time::Instant};
use tokio_util::sync::CancellationToken;

fn memory() -> Value {
    let raw = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let get = |key: &str| {
        raw.lines()
            .find(|l| l.starts_with(key))
            .and_then(|l| l.split_whitespace().nth(1))
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
                    object.insert(format!("{name}_evidence"), json!({
                        "has_useeffect":text.contains("useEffect"), "has_cleanup":text.to_lowercase().contains("cleanup"),
                        "has_proxy":text.to_lowercase().contains("proxy"), "has_middleware":text.to_lowercase().contains("middleware"),
                        "has_timeout":text.to_lowercase().contains("timeout"), "has_v2_context":text.contains("/v2/context"),
                        "has_v16":text.contains("v16.0.0") || text.contains("Next.js 16"),
                    }));
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
fn disk_bytes(path: &Path) -> u64 {
    std::fs::read_dir(path)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                disk_bytes(&path)
            } else {
                entry.metadata().map(|m| m.len()).unwrap_or(0)
            }
        })
        .sum()
}
async fn one(label: &str, request: SearchRequest, cache_minutes: u64) -> Result<Value> {
    let directory = tempfile::tempdir()?;
    let config = Config {
        requests_per_day: 100,
        cache_minutes,
        ..Default::default()
    };
    std::fs::write(
        directory.path().join("web-search.json"),
        serde_json::to_vec(&config)?,
    )?;
    let web = Web::open(directory.path()).await?;
    let cancel = CancellationToken::new();
    let at = Instant::now();
    let response = match web.search_with(request.clone(), &cancel).await {
        Ok(value) => json!({"ok":true,"response":value}),
        Err(error) => json!({"ok":false,"error":error.to_string()}),
    };
    let elapsed_ms = at.elapsed().as_millis();
    let status = web.status().await?;
    let mut probes = vec![];
    if let Some(url) = request.source_urls.first() {
        // Same public read tool the agent calls: repeat immediately and count actual requests.
        for n in 1..=2 {
            let before = web.status().await?.coverage.requests_today;
            let at = Instant::now();
            let result = match web.read(url, &request.query, &cancel).await {
                Ok(value) => json!({"ok":true,"page":value}),
                Err(error) => json!({"ok":false,"error":error.to_string()}),
            };
            probes.push(json!({"attempt":n,"elapsed_ms":at.elapsed().as_millis(),"http_requests":web.status().await?.coverage.requests_today-before,"result":result}));
        }
        if label == "exact_tokio" {
            let filtered = web
                .search_with(
                    SearchRequest {
                        days: Some(30),
                        discover: false,
                        read_pages: 0,
                        source_urls: vec![],
                        ..request.clone()
                    },
                    &cancel,
                )
                .await?;
            probes.push(json!({"kind":"last_30_days_on_documentation","response":filtered}));
        }
    }
    eprintln!(
        "{label}: {elapsed_ms} ms, {} hits, {} HTTP calls, {} indexed pages",
        response["response"]["results"]
            .as_array()
            .map_or(0, Vec::len),
        status.coverage.requests_today,
        status.coverage.indexed_pages
    );
    Ok(
        json!({"label":label,"request":request,"cache_minutes":cache_minutes,"elapsed_ms":elapsed_ms,"result":response,"status_after_search":status,"read_probes":probes,"corpus_bytes_before_cleanup":disk_bytes(directory.path())}),
    )
}
#[tokio::main]
async fn main() -> Result<()> {
    let output = std::env::args()
        .nth(1)
        .context("Supply a JSON report path")?;
    let cases = [
        (
            "unguided_react",
            "Busca documentación actualizada de React useEffect cleanup",
            "",
            "",
            15,
        ),
        (
            "unguided_next",
            "Next.js 16 proxy middleware documentation",
            "",
            "",
            15,
        ),
        (
            "unguided_tokio",
            "Tokio timeout latest documentation",
            "",
            "",
            15,
        ),
        ("domain_react", "useEffect cleanup", "react.dev", "", 15),
        (
            "domain_next",
            "proxy middleware Next.js 16",
            "nextjs.org",
            "",
            15,
        ),
        (
            "domain_context7",
            "API v2 context documentation",
            "context7.com",
            "",
            15,
        ),
        (
            "exact_react",
            "useEffect cleanup",
            "react.dev",
            "https://react.dev/reference/react/useEffect",
            15,
        ),
        (
            "exact_next",
            "proxy middleware Next.js 16",
            "nextjs.org",
            "https://nextjs.org/docs/app/api-reference/file-conventions/proxy",
            15,
        ),
        (
            "exact_tokio",
            "tokio timeout",
            "docs.rs",
            "https://docs.rs/tokio/latest/tokio/time/fn.timeout.html",
            15,
        ),
        (
            "exact_context7",
            "API v2 context",
            "context7.com",
            "https://context7.com/docs/api-guide",
            15,
        ),
    ];
    let started = Instant::now();
    let initial = memory();
    let mut rows = vec![];
    // Three independent fresh-corpus searches at a time; do not contaminate unguided cases with known answer URLs.
    for batch in cases.chunks(3) {
        let results = futures_util::future::join_all(batch.iter().map(
            |(label, query, domain, url, cache)| {
                one(
                    label,
                    SearchRequest {
                        query: (*query).into(),
                        domains: if domain.is_empty() {
                            vec![]
                        } else {
                            vec![(*domain).into()]
                        },
                        source_urls: if url.is_empty() {
                            vec![]
                        } else {
                            vec![(*url).into()]
                        },
                        discover: true,
                        ..Default::default()
                    },
                    *cache,
                )
            },
        ))
        .await;
        for result in results {
            rows.push(result?);
        }
    }
    rows.push(
        one(
            "zero_cache_control",
            SearchRequest {
                query: "tokio timeout".into(),
                domains: vec!["docs.rs".into()],
                source_urls: vec!["https://docs.rs/tokio/latest/tokio/time/fn.timeout.html".into()],
                discover: false,
                ..Default::default()
            },
            0,
        )
        .await?,
    );
    let mut report = json!({"performed_at":chrono::Utc::now().to_rfc3339(),"method":"10 live searches matching agent discover=true defaults, isolated fresh corpora per case, three concurrently; default seeds and limits, no background crawler or model, no external search API. Known-source cases are assisted controls, not autonomous discovery. Repeated public web_read calls measure caching; a separate cache_minutes=0 control measures revalidation. Temporary corpora are deleted; report stores metadata and evidence flags, not documentation bodies.","initial_memory":initial,"final_memory":memory(),"wall_ms":started.elapsed().as_millis(),"cases":rows});
    compact(&mut report);
    if let Some(parent) = Path::new(&output).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output, serde_json::to_vec_pretty(&report)?)?;
    println!("Saved {output}");
    Ok(())
}
