//! Explicit live HTTP smoke check. Uses a temporary corpus and no model provider.
use anyhow::{Context, Result};
use fenix_web::{Config, Web};
use tokio_util::sync::CancellationToken;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let url = args
        .next()
        .context("Usage: read_search <public-url> <query>")?;
    let query = args
        .next()
        .context("Supply a query to search the downloaded text")?;
    let directory = tempfile::tempdir()?;
    let config = Config {
        seeds: vec![],
        requests_per_day: 12,
        ..Config::default()
    };
    std::fs::write(
        directory.path().join("web-search.json"),
        serde_json::to_vec(&config)?,
    )?;
    let web = Web::open(directory.path()).await?;
    let cancel = CancellationToken::new();
    let page = web.read(&url, &query, &cancel).await?;
    println!("Read: {} ({})", page.url, page.source);
    let search = web.search(&query, 5, 3, &cancel).await?;
    println!("{}", serde_json::to_string_pretty(&search)?);
    anyhow::ensure!(
        !search.results.is_empty(),
        "The downloaded text did not match the query"
    );
    Ok(())
}
