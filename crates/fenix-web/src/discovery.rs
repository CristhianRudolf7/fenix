//! Bounded query-directed traversal of original sites, without search services.
use crate::{SearchRequest, SourceRead, Web, fetch};
use anyhow::Result;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default, Serialize)]
pub struct DiscoveryReport {
    pub enabled: bool,
    pub entrypoints: Vec<String>,
    pub sources: Vec<SourceRead>,
    pub candidates_considered: usize,
    pub followed_urls: usize,
    pub unvisited_domains: Vec<String>,
    pub stop_reason: &'static str,
    pub background_queued: usize,
}

impl Web {
    pub(crate) async fn discover(
        &self,
        request: &SearchRequest,
        deadline: tokio::time::Instant,
    ) -> Result<DiscoveryReport> {
        let mut report = DiscoveryReport {
            enabled: request.discover,
            stop_reason: "disabled",
            ..Default::default()
        };
        if !request.discover {
            return Ok(report);
        }
        report.stop_reason = "link_budget";
        // Use indexed pages as locators, not as a substitute for current reads.
        // Publication-window and explicit recency queries still discover new URLs.
        if request.source_urls.is_empty() && !request.needs_new_links() {
            let known = self.index.search_with(request).await?;
            let targets: Vec<_> = known
                .pages
                .iter()
                .filter(|page| {
                    fetch::public_url(&page.document.url)
                        .is_ok_and(|u| !u.path().trim_matches('/').is_empty())
                        && crate::search::strong_source(
                            &page.document.title,
                            &page.document.url,
                            &page.document.text,
                            &request.query,
                        )
                })
                .take(request.read_pages.clamp(1, 3))
                .map(|page| page.document.url.clone())
                .collect();
            let verification_deadline =
                deadline.min(tokio::time::Instant::now() + std::time::Duration::from_secs(15));
            report.sources = futures_util::future::join_all(
                targets
                    .iter()
                    .map(|url| self.read_source(url, verification_deadline, request.fresh)),
            )
            .await;
            let checked: HashSet<_> = report
                .sources
                .iter()
                .filter(|s| s.status == "read" && s.indexed)
                .filter_map(|s| s.resolved_url.as_deref())
                .collect();
            let current = self.index.search_with(request).await?;
            if current.pages.iter().any(|page| {
                checked.contains(page.document.url.as_str())
                    && crate::search::strong_source(
                        &page.document.title,
                        &page.document.url,
                        &page.document.text,
                        &request.query,
                    )
            }) {
                report.entrypoints = targets;
                report.stop_reason = "indexed_sources";
                let visited: HashSet<_> = report
                    .sources
                    .iter()
                    .flat_map(|s| std::iter::once(s.url.clone()).chain(s.resolved_url.clone()))
                    .collect();
                let origins = visited
                    .iter()
                    .filter_map(|u| fetch::public_url(u).ok())
                    .map(|u| u.origin().ascii_serialization())
                    .collect();
                report.background_queued = self.defer_discovery(request, origins, &visited).await?;
                Self::unvisited_domains(request, &mut report);
                return Ok(report);
            }
        }
        // Navigation may use its short TTL cache; final evidence still honors
        // fresh=true. A failed/changed indexed source or a date-filtered lookup
        // forces fresh navigation so we can discover replacement/new pages.
        let refresh_navigation =
            request.fresh && (request.needs_new_links() || !report.sources.is_empty());
        // Explicit source pages have already been read. Otherwise visit at most
        // four roots/seeds; never manufacture article URLs or query endpoints.
        if request.source_urls.is_empty() {
            if request.domains.is_empty() {
                let terms: Vec<_> = request
                    .query
                    .split_whitespace()
                    .map(str::to_lowercase)
                    .collect();
                let mut seeds = self.config.seeds.clone();
                seeds.sort_by_key(|url| {
                    std::cmp::Reverse(
                        terms
                            .iter()
                            .filter(|term| url.to_lowercase().contains(term.as_str()))
                            .count(),
                    )
                });
                let routed = if self.config.seeds.is_empty() {
                    vec![]
                } else {
                    crate::sources::roots(&request.query)
                };
                if !routed.is_empty() {
                    seeds = routed;
                }
                let mut origins = HashSet::new();
                report.entrypoints = seeds
                    .into_iter()
                    .filter(|value| {
                        fetch::public_url(value).is_ok_and(|url| origins.insert(url.origin()))
                    })
                    .take(4)
                    .collect();
            } else {
                report.entrypoints = request
                    .domains
                    .iter()
                    .take(4)
                    .map(|domain| match domain.as_str() {
                        "reddit.com" | "www.reddit.com" => {
                            "https://www.reddit.com/r/all/.rss".into()
                        }
                        _ => format!("https://{domain}/"),
                    })
                    .collect();
                report.unvisited_domains = request.domains.iter().skip(4).cloned().collect();
            }
            report.sources.extend(
                futures_util::future::join_all(
                    report
                        .entrypoints
                        .iter()
                        .map(|url| self.read_source(url, deadline, refresh_navigation)),
                )
                .await,
            );
        }
        let mut visited: HashSet<_> = request
            .source_urls
            .iter()
            .chain(report.entrypoints.iter())
            .cloned()
            .collect();
        for source in &report.sources {
            visited.insert(source.url.clone());
            if let Some(url) = &source.resolved_url {
                visited.insert(url.clone());
            }
        }
        let mut navigation_origins: Vec<_> = visited
            .iter()
            .filter_map(|value| fetch::public_url(value).ok())
            .map(|url| url.origin().ascii_serialization())
            .collect();
        let mut origins = HashMap::<String, usize>::new();
        let mut considered = HashSet::new();
        // Best-first bounded expansion: four waves of up to four reads. Each
        // origin can contribute eight pages, so a documentation root can lead
        // through a catalog and section to the requested reference.
        for wave in 1..=4 {
            if tokio::time::Instant::now() >= deadline {
                report.stop_reason = "time_budget";
                break;
            }
            let candidates = self
                .index
                .candidates(request, navigation_origins.clone())
                .await?;
            considered.extend(candidates.iter().cloned());
            let targets: Vec<_> = candidates
                .into_iter()
                .filter(|value| {
                    let Ok(url) = fetch::public_url(value) else {
                        return false;
                    };
                    if visited.contains(value) {
                        return false;
                    }
                    let count = origins
                        .entry(url.origin().ascii_serialization())
                        .or_default();
                    if *count >= wave * 2 {
                        return false;
                    }
                    *count += 1;
                    visited.insert(value.clone());
                    true
                })
                .take((16 - report.followed_urls).min(4))
                .collect();
            if targets.is_empty() {
                report.stop_reason = "no_unvisited_relevant_links";
                break;
            }
            report.followed_urls += targets.len();
            let reads = futures_util::future::join_all(
                targets
                    .iter()
                    .map(|url| self.read_source(url, deadline, refresh_navigation)),
            )
            .await;
            for read in &reads {
                if let Some(url) = &read.resolved_url {
                    visited.insert(url.clone());
                    if let Ok(url) = fetch::public_url(url) {
                        let origin = url.origin().ascii_serialization();
                        if !navigation_origins.contains(&origin) {
                            navigation_origins.push(origin);
                        }
                    }
                }
            }
            report.sources.extend(reads);
            // Stop once an actual source, rather than just a navigation page,
            // has strong lexical support. This is a work budget heuristic, not
            // a claim that the source is correct or research is exhaustive.
            let found = self.index.search_with(request).await?;
            // A matching landing page can expose a more specific reference.
            // Give those newly observed links one expansion before stopping.
            if wave >= 2
                && found.pages.iter().any(|page| {
                    visited.contains(&page.document.url)
                        && crate::search::strong_source(
                            &page.document.title,
                            &page.document.url,
                            &page.document.text,
                            &request.query,
                        )
                })
            {
                report.stop_reason = "strong_lexical_source";
                break;
            }
            if report.followed_urls == 16 {
                break;
            }
        }
        report.candidates_considered = considered.len();
        report.background_queued = self
            .defer_discovery(request, navigation_origins, &visited)
            .await?;
        Self::unvisited_domains(request, &mut report);
        Ok(report)
    }

    async fn defer_discovery(
        &self,
        request: &SearchRequest,
        origins: Vec<String>,
        visited: &HashSet<String>,
    ) -> Result<usize> {
        let pending = self
            .index
            .candidates(request, origins)
            .await?
            .into_iter()
            .filter(|url| !visited.contains(url))
            .take(16)
            .collect();
        self.index.prioritize(pending).await
    }

    fn unvisited_domains(request: &SearchRequest, report: &mut DiscoveryReport) {
        report.unvisited_domains = request
            .domains
            .iter()
            .filter(|domain| {
                !request
                    .source_urls
                    .iter()
                    .chain(report.sources.iter().map(|source| &source.url))
                    .any(|value| {
                        fetch::public_url(value).is_ok_and(|url| {
                            url.host_str().is_some_and(|host| {
                                host == domain.as_str() || host.ends_with(&format!(".{domain}"))
                            })
                        })
                    })
            })
            .cloned()
            .collect();
    }
}
