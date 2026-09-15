//! Opt-in live benchmark of a shared Web instance; no model or search provider.
use anyhow::Result;
use fenix_web::{Config, Web};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

const TOPICS: [(&str, &str, &str); 10] = [
    (
        "Astronomy",
        "solar system",
        "https://science.nasa.gov/solar-system/",
    ),
    (
        "Health",
        "malaria",
        "https://www.who.int/news-room/fact-sheets/detail/malaria",
    ),
    (
        "Climate",
        "climate change",
        "https://www.un.org/en/climatechange/what-is-climate-change",
    ),
    (
        "Economics",
        "poverty",
        "https://www.worldbank.org/en/topic/poverty/overview",
    ),
    (
        "History",
        "Roman Empire",
        "https://en.wikipedia.org/wiki/Roman_Empire",
    ),
    (
        "Biology",
        "photosynthesis",
        "https://en.wikipedia.org/wiki/Photosynthesis",
    ),
    (
        "Programming",
        "installation Rust",
        "https://doc.rust-lang.org/book/ch01-01-installation.html",
    ),
    (
        "Web design",
        "CSS",
        "https://developer.mozilla.org/en-US/docs/Web/CSS",
    ),
    (
        "AI research",
        "attention transformer",
        "https://arxiv.org/abs/1706.03762",
    ),
    (
        "Mathematics",
        "Pythagorean theorem",
        "https://en.wikipedia.org/wiki/Pythagorean_theorem",
    ),
];

fn rss() -> u64 {
    fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find(|l| l.starts_with("VmRSS:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}
fn cpu_ticks() -> u64 {
    let s = fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let fields: Vec<_> = s
        .rsplit_once(')')
        .map(|(_, tail)| tail.split_whitespace().collect())
        .unwrap_or_default();
    [11, 12]
        .iter()
        .filter_map(|i| fields.get(*i)?.parse::<u64>().ok())
        .sum()
}
fn disk(path: &Path) -> u64 {
    fs::read_dir(path)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| {
            let p = e.path();
            if p.is_dir() {
                disk(&p)
            } else {
                e.metadata().map(|m| m.len()).unwrap_or(0)
            }
        })
        .sum()
}
async fn query(web: Arc<Web>, topic: &'static str, query: &'static str, reads: usize) -> Value {
    let now = Instant::now();
    match web.search(query, 5, reads, &CancellationToken::new()).await {
        Ok(result) => {
            json!({"topic":topic,"query":query,"elapsed_ms":now.elapsed().as_secs_f64()*1000.0,
            "hits":result.results.len(),"results":result.results.iter().map(|r| json!({"url":r.url,"title":r.title,"read_status":r.read_status,"read_error":r.read_error})).collect::<Vec<_>>()})
        }
        Err(e) => {
            json!({"topic":topic,"query":query,"elapsed_ms":now.elapsed().as_secs_f64()*1000.0,"error":e.to_string()})
        }
    }
}
async fn phase(web: Arc<Web>, name: &str, concurrency: usize, reads: usize) -> Result<Value> {
    let before = web.status().await?;
    let start_rss = rss();
    let cpu = cpu_ticks();
    let start = Instant::now();
    let stop = CancellationToken::new();
    let token = stop.clone();
    let sampler = tokio::spawn(async move {
        let mut peak = rss();
        loop {
            tokio::select! { _ = token.cancelled() => return peak.max(rss()), _ = tokio::time::sleep(Duration::from_millis(20)) => peak = peak.max(rss()) }
        }
    });
    let mut rows = Vec::new();
    for chunk in TOPICS.chunks(concurrency) {
        let mut tasks = Vec::new();
        for &(topic, q, _) in chunk {
            tasks.push(tokio::spawn(query(web.clone(), topic, q, reads)));
        }
        for task in tasks {
            rows.push(task.await?);
        }
    }
    let elapsed = start.elapsed().as_secs_f64();
    stop.cancel();
    let peak = sampler.await?;
    let after = web.status().await?;
    let result = json!({"name":name,"concurrency":concurrency,"read_pages":reads,"elapsed_seconds":elapsed,
        "cpu_ticks":cpu_ticks()-cpu,"rss_start_kib":start_rss,"rss_end_kib":rss(),"peak_sampled_rss_kib":peak,
        "http_attempts":after.coverage.requests_today-before.coverage.requests_today,"coverage":after.coverage,"queries":rows});
    println!(
        "{}",
        json!({"phase":name,"seconds":elapsed,"peak_rss_kib":peak,"http_attempts":result["http_attempts"]})
    );
    Ok(result)
}

#[tokio::main(worker_threads = 2)]
async fn main() -> Result<()> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/verification/web-benchmark.json".into());
    let dir = tempfile::tempdir()?;
    let mut report = json!({"method":"single shared Web instance; live public HTTP; unoptimized build; no LLM; 20 ms RSS sampling; fixed ten English queries; not a search relevance benchmark","topics":TOPICS,"phases":[],"bootstrap":[]});
    let mut phases = Vec::new();
    let web = Web::open(dir.path()).await?;
    phases.push(phase(web.clone(), "empty_index", 1, 3).await?);
    let stop = CancellationToken::new();
    let worker = tokio::spawn(web.clone().run(stop.clone()));
    let start = Instant::now();
    for _ in 0..6 {
        tokio::time::sleep(Duration::from_secs(10)).await;
        println!(
            "{}",
            json!({"crawl_seconds":start.elapsed().as_secs(),"status":web.status().await?,"rss_kib":rss()})
        );
    }
    stop.cancel();
    worker.await?;
    report["default_crawl"] = json!({"seconds":start.elapsed().as_secs_f64(),"status":web.status().await?,"rss_kib":rss(),"disk_bytes":disk(dir.path())});
    phases.push(phase(web.clone(), "default_corpus", 1, 3).await?);
    // Known URLs enrich the corpus to exercise source reads; this is NOT query-based discovery.
    let mut bootstrap = Vec::new();
    for (topic, _, url) in TOPICS {
        let start = Instant::now();
        let result = web.read(url, "", &CancellationToken::new()).await;
        let row = match result {
            Ok(p) => {
                json!({"topic":topic,"url":url,"seconds":start.elapsed().as_secs_f64(),"source":p.source,"passage_chars":p.passage.chars().count()})
            }
            Err(e) => {
                json!({"topic":topic,"url":url,"seconds":start.elapsed().as_secs_f64(),"error":e.to_string()})
            }
        };
        println!("{}", json!({"bootstrap":row}));
        bootstrap.push(row);
    }
    report["bootstrap"] = json!(bootstrap);
    for (name, c, r) in [
        ("index_sequential", 1, 0),
        ("index_parallel_4", 4, 0),
        ("index_parallel_10", 10, 0),
        ("cached_reads_sequential", 1, 3),
        ("cached_reads_parallel_4", 4, 3),
        ("cached_reads_parallel_10", 10, 3),
    ] {
        phases.push(phase(web.clone(), name, c, r).await?);
    }
    drop(web);
    let config = Config {
        cache_minutes: 0,
        ..Config::default()
    };
    fs::write(
        dir.path().join("web-search.json"),
        serde_json::to_vec(&config)?,
    )?;
    let web = Web::open(dir.path()).await?;
    for (name, c) in [
        ("fresh_reads_sequential", 1),
        ("fresh_reads_parallel_4", 4),
        ("fresh_reads_parallel_10", 10),
    ] {
        phases.push(phase(web.clone(), name, c, 3).await?);
    }
    report["phases"] = json!(phases);
    report["final_status"] = serde_json::to_value(web.status().await?)?;
    report["disk_bytes"] = json!(disk(dir.path()));
    report["clock_ticks_per_second"] = json!(
        std::process::Command::new("getconf")
            .arg("CLK_TCK")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
            .unwrap_or_default()
    );
    if let Some(parent) = Path::new(&output).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output, serde_json::to_vec_pretty(&report)?)?;
    println!("Report: {output}");
    Ok(())
}
