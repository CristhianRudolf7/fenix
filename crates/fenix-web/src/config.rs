use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// Resource budgets, not a claim of worldwide search coverage. No search API keys.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub enabled: bool,
    pub max_concurrent_searches: usize,
    pub seeds: Vec<String>,
    pub max_pages: usize,
    pub max_frontier: usize,
    pub max_urls_per_origin: usize,
    pub max_body_bytes: usize,
    pub max_text_chars: usize,
    pub request_interval_ms: u64,
    pub requests_per_day: usize,
    pub refresh_hours: u64,
    pub cache_minutes: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent_searches: 10,
            // Bootstrap addresses, not an allowlist. Outgoing public links expand
            // the corpus across topics and domains without querying a search engine.
            seeds: [
                "https://es.wikipedia.org/wiki/Portal:Portada",
                "https://en.wikipedia.org/wiki/Main_Page",
                "https://www.gob.pe/",
                "https://www.nasa.gov/",
                "https://www.un.org/en/",
                "https://www.who.int/",
                "https://www.worldbank.org/",
                "https://www.si.edu/",
                "https://www.bbc.com/mundo",
                "https://arxiv.org/",
                "https://developer.mozilla.org/en-US/docs/Web",
                "https://doc.rust-lang.org/book/",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            max_pages: 5_000,
            max_frontier: 20_000,
            max_urls_per_origin: 200,
            max_body_bytes: 512 * 1024,
            max_text_chars: 24_000,
            request_interval_ms: 2_000,
            requests_per_day: 2_000,
            refresh_hours: 72,
            cache_minutes: 15,
        }
    }
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        if !(1..=10).contains(&self.max_concurrent_searches)
            || self.seeds.len() > 100
            || !(1..=100_000).contains(&self.max_pages)
            || !(1..=200_000).contains(&self.max_frontier)
            || !(1..=10_000).contains(&self.max_urls_per_origin)
            || !(1_024..=2 * 1024 * 1024).contains(&self.max_body_bytes)
            || !(256..=64_000).contains(&self.max_text_chars)
            || !(100..=60_000).contains(&self.request_interval_ms)
            || !(1..=100_000).contains(&self.requests_per_day)
            || !(1..=720).contains(&self.refresh_hours)
            || self.cache_minutes > 1_440
        {
            bail!("Invalid web-search resource limits; see docs/en/web-search.md");
        }
        for seed in &self.seeds {
            crate::fetch::public_url(seed)?;
        }
        Ok(())
    }
}
