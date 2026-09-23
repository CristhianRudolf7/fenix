use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// Resource budgets, not a claim of worldwide search coverage. No search API keys.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub enabled: bool,
    /// Exact destinations configured by the owner, never by a tool call.
    /// None preserves public-web access; Some([]) denies every HTTP request.
    pub allowed_hosts: Option<Vec<String>>,
    pub max_concurrent_searches: usize,
    pub seeds: Vec<String>,
    pub max_pages: usize,
    pub max_frontier: usize,
    pub max_urls_per_origin: usize,
    pub max_body_bytes: usize,
    pub max_text_chars: usize,
    pub request_interval_ms: u64,
    pub requests_per_day: usize,
    /// Best-effort reserve: stop starting crawls before all daily requests are
    /// consumed. An already-started crawl may still fetch robots or redirects.
    pub foreground_reserve_percent: u8,
    pub refresh_hours: u64,
    pub cache_minutes: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_hosts: None,
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
                "https://news.ycombinator.com/rss",
                "https://www.reddit.com/r/all/.rss",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            max_pages: 5_000,
            max_frontier: 20_000,
            max_urls_per_origin: 2_000,
            max_body_bytes: 2 * 1024 * 1024,
            max_text_chars: 64_000,
            request_interval_ms: 2_000,
            requests_per_day: 2_000,
            foreground_reserve_percent: 20,
            refresh_hours: 72,
            cache_minutes: 15,
        }
    }
}

impl Config {
    /// Daily shared-counter threshold for starting background crawl work.
    /// Permit a first crawl with tiny budgets; all HTTP calls still honor the
    /// full request limit in the network layer.
    pub fn crawler_request_budget(&self) -> usize {
        (self.requests_per_day * usize::from(100 - self.foreground_reserve_percent) / 100).max(1)
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(hosts) = &self.allowed_hosts {
            if hosts.len() > 128 {
                bail!("At most 128 allowed web hosts are supported");
            }
            for host in hosts {
                let url = crate::fetch::public_url(&format!("https://{host}/"))?;
                if host.contains('*')
                    || url.host_str() != Some(host.as_str())
                    || url.path() != "/"
                    || url.query().is_some()
                    || url.fragment().is_some()
                    || host.ends_with('.')
                {
                    bail!(
                        "Allowed web hosts must be exact canonical names, without paths, ports or wildcards"
                    );
                }
            }
        }
        if !(1..=10).contains(&self.max_concurrent_searches)
            || self.seeds.len() > 100
            || !(1..=100_000).contains(&self.max_pages)
            || !(1..=200_000).contains(&self.max_frontier)
            || !(1..=10_000).contains(&self.max_urls_per_origin)
            || !(1_024..=2 * 1024 * 1024).contains(&self.max_body_bytes)
            || !(256..=64_000).contains(&self.max_text_chars)
            || !(100..=60_000).contains(&self.request_interval_ms)
            || !(1..=100_000).contains(&self.requests_per_day)
            || self.foreground_reserve_percent > 90
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

    pub(crate) fn authorize_url(&self, url: &url::Url) -> Result<()> {
        if let Some(hosts) = &self.allowed_hosts {
            let host = url.host_str().unwrap_or_default().trim_end_matches('.');
            if !hosts.iter().any(|allowed| allowed == host) {
                bail!("Destination is outside the configured web host policy");
            }
        }
        Ok(())
    }
}
