//! Public HTTP only. Resolve and pin approved addresses before every request,
//! including robots and redirect hops; never inherit proxies or authentication.
use crate::{Config, concurrency::Entries, index::Index};
use anyhow::{Context, Result, bail};
use reqwest::{
    StatusCode, Url,
    header::{self, HeaderMap},
};
use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use tokio::sync::Semaphore;

pub(crate) const AGENT: &str = "FenixBot";

pub(crate) fn public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let [a, b, c, _] = ip.octets();
            !(a == 0
                || a == 10
                || a == 127
                || a >= 224
                || (a == 100 && (64..=127).contains(&b))
                || (a == 169 && b == 254)
                || (a == 172 && (16..=31).contains(&b))
                || (a == 192
                    && (b == 168 || (b == 0 && (c == 0 || c == 2)) || (b == 88 && c == 99)))
                || (a == 198 && (b == 18 || b == 19 || (b == 51 && c == 100)))
                || (a == 203 && b == 0 && c == 113))
        }
        IpAddr::V6(ip) => {
            if let Some(ip) = ip.to_ipv4_mapped() {
                return public_ip(IpAddr::V4(ip));
            }
            let s = ip.segments();
            // Global unicast only; reject special-use, documentation and IPv4
            // transition prefixes that can encode otherwise forbidden targets.
            (s[0] & 0xe000) == 0x2000
                && !(s[0] == 0x2001 && (s[1] < 0x0200 || s[1] == 0x0db8))
                && s[0] != 0x2002
                && !(s[0] == 0x3fff && s[1] < 0x1000)
        }
    }
}

pub(crate) fn public_url(value: &str) -> Result<Url> {
    if value.len() > 2048 {
        bail!("URL exceeds 2048 bytes");
    }
    let mut url = Url::parse(value).context("Expected an absolute HTTP or HTTPS URL")?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        bail!("Only public HTTP(S) URLs without credentials are allowed");
    }
    if !matches!(url.port_or_known_default(), Some(80 | 443)) {
        bail!("Only public web ports 80 and 443 are allowed");
    }
    match url.host().context("URL requires a host")? {
        url::Host::Ipv4(ip) if !public_ip(IpAddr::V4(ip)) => {
            bail!("Private, local and reserved web addresses are not allowed")
        }
        url::Host::Ipv6(ip) if !public_ip(IpAddr::V6(ip)) => {
            bail!("Private, local and reserved web addresses are not allowed")
        }
        url::Host::Domain(host) => {
            let host = host.trim_end_matches('.');
            if !host.contains('.')
                || [".localhost", ".local", ".internal", ".home", ".onion"]
                    .iter()
                    .any(|suffix| host.ends_with(suffix))
            {
                bail!("Only public internet hostnames are allowed");
            }
        }
        _ => {}
    }
    if url.query_pairs().any(|(name, _)| {
        matches!(
            name.to_ascii_lowercase().as_str(),
            "access_token"
                | "token"
                | "api_key"
                | "apikey"
                | "password"
                | "secret"
                | "signature"
                | "x-amz-signature"
        )
    }) {
        bail!("URLs containing credential parameters cannot enter the public web index");
    }
    url.set_fragment(None);
    Ok(url)
}

pub(crate) struct Response {
    pub url: Url,
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

pub(crate) struct Network {
    index: Index,
    config: Config,
    /// Global cap shared by foreground calls and the crawler, held through streaming.
    downloads: Semaphore,
    origins: Entries<()>,
    robots_loads: Entries<()>,
    #[cfg(test)]
    pub(crate) test_address: Option<SocketAddr>,
}

impl Network {
    pub(crate) fn new(index: Index, config: Config) -> Self {
        Self {
            index,
            downloads: Semaphore::new(config.max_concurrent_searches),
            config,
            origins: Entries::default(),
            robots_loads: Entries::default(),
            #[cfg(test)]
            test_address: None,
        }
    }

    pub(crate) async fn page(
        &self,
        url: Url,
        etag: Option<&str>,
        modified: Option<&str>,
    ) -> Result<Response> {
        tokio::time::timeout(Duration::from_secs(30), async {
            let mut current = url;
            for hop in 0..=5 {
                let robots = self.robots(&current).await?;
                if !robotstxt::DefaultMatcher::default().one_agent_allowed_by_robots(
                    &robots,
                    AGENT,
                    current.as_str(),
                ) {
                    bail!("robots.txt does not permit reading this URL");
                }
                let delay = crawl_delay(&robots).max(self.config.request_interval_ms);
                if delay > 60_000 {
                    bail!("Site requests a crawl delay longer than this fetch budget");
                }
                let response = self
                    .once(
                        current.clone(),
                        self.config.max_body_bytes,
                        delay,
                        if hop == 0 { etag } else { None },
                        if hop == 0 { modified } else { None },
                    )
                    .await?;
                if is_redirect(response.status) {
                    let location = response
                        .headers
                        .get(header::LOCATION)
                        .context("Redirect has no Location")?
                        .to_str()?;
                    current = public_url(current.join(location)?.as_str())?;
                    continue;
                }
                return Ok(response);
            }
            bail!("Too many redirects");
        })
        .await
        .context("Web fetch timed out (30 seconds)")?
    }

    async fn robots(&self, url: &Url) -> Result<String> {
        let origin = url.origin().ascii_serialization();
        let entry = self.robots_loads.get(&origin)?;
        let _load = entry.lock().await;
        if let Some(body) = self.index.robots(&origin).await? {
            return Ok(body);
        }
        let mut current = Url::parse(&format!("{origin}/robots.txt"))?;
        let attempt = async {
            for _ in 0..=5 {
                let response = self
                    .once(
                        current.clone(),
                        512 * 1024,
                        self.config.request_interval_ms,
                        None,
                        None,
                    )
                    .await?;
                if is_redirect(response.status) {
                    let location = response
                        .headers
                        .get(header::LOCATION)
                        .context("Robots redirect has no Location")?
                        .to_str()?;
                    current = public_url(current.join(location)?.as_str())?;
                    continue;
                }
                let code = response.status.as_u16();
                if matches!(code, 401 | 403 | 429) || code >= 500 {
                    bail!("Site robots policy is unavailable or access is restricted");
                }
                if (400..500).contains(&code) {
                    return Ok(String::new());
                }
                if !response.status.is_success() {
                    bail!("Unexpected robots response");
                }
                let body = String::from_utf8_lossy(&response.body).into_owned();
                if body
                    .trim_start()
                    .to_ascii_lowercase()
                    .starts_with("<!doctype html")
                    || body.trim_start().starts_with("<html")
                {
                    bail!("robots.txt returned an HTML page instead of a policy");
                }
                return Ok(body);
            }
            bail!("Too many robots redirects");
        }
        .await;
        match attempt {
            Ok(body) => {
                self.index
                    .save_robots(&origin, body.clone(), 86_400)
                    .await?;
                let sitemaps = body
                    .lines()
                    .filter_map(|line| {
                        let (key, value) = line.split_once(':')?;
                        if !key.trim().eq_ignore_ascii_case("sitemap") {
                            return None;
                        }
                        let map = public_url(value.trim()).ok()?;
                        (map.origin() == url.origin()).then(|| map.to_string())
                    })
                    .take(10)
                    .collect();
                self.index.enqueue(sitemaps, 1).await?;
                Ok(body)
            }
            Err(error) => {
                // Cache a conservative temporary denial, not a persistent allow
                // after a network failure. Retry after five minutes.
                self.index
                    .save_robots(&origin, "User-agent: *\nDisallow: /\n".into(), 300)
                    .await?;
                Err(error)
            }
        }
    }

    async fn once(
        &self,
        url: Url,
        max_bytes: usize,
        delay_ms: u64,
        etag: Option<&str>,
        modified: Option<&str>,
    ) -> Result<Response> {
        let origin = url.origin().ascii_serialization();
        // Wait for this site's lane and rate limit before acquiring scarce global
        // capacity. A slow/throttled origin cannot occupy all download permits.
        let entry = self.origins.get(&origin)?;
        let _origin = entry.lock().await;
        let _download = loop {
            let wait = self.index.request_wait(&origin).await?;
            if wait > 0 {
                tokio::time::sleep(Duration::from_millis(wait)).await;
            }
            let permit = self.downloads.acquire().await?;
            let wait = self.index.reserve_request(&origin, delay_ms).await?;
            if wait == 0 {
                break permit;
            }
            drop(permit);
            tokio::time::sleep(Duration::from_millis(wait)).await;
        };
        let host = url
            .host_str()
            .context("Missing host")?
            .trim_matches(['[', ']']);
        let port = url.port_or_known_default().context("Missing web port")?;
        #[cfg(test)]
        let addresses = if self.test_address.is_some() {
            vec![SocketAddr::from(([93, 184, 216, 34], port))]
        } else {
            resolve(host, port).await?
        };
        #[cfg(not(test))]
        let addresses = resolve(host, port).await?;
        if addresses.is_empty() || addresses.iter().any(|a| !public_ip(a.ip())) {
            bail!("Web host resolves to a private, local or reserved address");
        }
        let builder = reqwest::Client::builder()
            .user_agent("FenixBot/0.2 (+https://github.com/CristhianRudolf7/fenix)")
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .referer(false)
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(12))
            .pool_max_idle_per_host(0)
            .resolve_to_addrs(host, &addresses);
        #[cfg(test)]
        let request_url = if let Some(address) = self.test_address {
            let mut local = url.clone();
            local.set_host(Some(&address.ip().to_string()))?;
            local
                .set_port(Some(address.port()))
                .map_err(|_| anyhow::anyhow!("Test port"))?;
            local
        } else {
            url.clone()
        };
        #[cfg(not(test))]
        let request_url = url.clone();
        let client = builder.build()?;
        let mut request = client.get(request_url).header(
            header::ACCEPT,
            "text/html,application/xhtml+xml,text/plain,application/xml,text/xml;q=0.9",
        );
        if let Some(value) = etag {
            request = request.header(header::IF_NONE_MATCH, value);
        }
        if let Some(value) = modified {
            request = request.header(header::IF_MODIFIED_SINCE, value);
        }
        let mut response = request
            .send()
            .await
            .map_err(|e| e.without_url())
            .context("Public HTTP request failed")?;
        let status = response.status();
        let headers = response.headers().clone();
        if response
            .content_length()
            .is_some_and(|s| s > max_bytes as u64)
        {
            bail!("Web response exceeds the download byte limit");
        }
        let mut body = Vec::new();
        if status.is_success() {
            while let Some(chunk) = response.chunk().await.map_err(|e| e.without_url())? {
                if body.len().saturating_add(chunk.len()) > max_bytes {
                    bail!("Web response exceeds the download byte limit");
                }
                body.extend_from_slice(&chunk);
            }
        }
        Ok(Response {
            url,
            status,
            headers,
            body,
        })
    }

    pub(crate) fn active_downloads(&self) -> usize {
        self.config.max_concurrent_searches - self.downloads.available_permits()
    }
}

async fn resolve(host: &str, port: u16) -> Result<Vec<SocketAddr>> {
    Ok(tokio::net::lookup_host((host, port))
        .await
        .context("Could not resolve public web host")?
        .take(32)
        .collect())
}

fn is_redirect(status: StatusCode) -> bool {
    matches!(status.as_u16(), 301 | 302 | 303 | 307 | 308)
}

fn crawl_delay(robots: &str) -> u64 {
    // Conservatively honor the largest delay, even in another agent's group.
    robots
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once(':')?;
            if !key.trim().eq_ignore_ascii_case("crawl-delay") {
                return None;
            }
            let seconds = value.split('#').next()?.trim().parse::<f64>().ok()?;
            (seconds.is_finite() && seconds > 0.0).then_some((seconds * 1000.0) as u64)
        })
        .max()
        .unwrap_or(0)
}
