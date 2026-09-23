//! Query controls for the installation's own lexical index, never a metasearch API.
use crate::{fetch, index::Page};
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SearchRequest {
    pub query: String,
    pub limit: usize,
    pub read_pages: usize,
    pub domains: Vec<String>,
    pub days: Option<u32>,
    pub max_per_domain: usize,
    pub source_urls: Vec<String>,
    /// Follow a bounded set of relevant links before querying the local index.
    /// Library callers opt in; the agent enables this by default.
    pub discover: bool,
    /// Revalidate selected sources over HTTP instead of trusting the TTL cache.
    pub fresh: bool,
}

impl Default for SearchRequest {
    fn default() -> Self {
        Self {
            query: String::new(),
            limit: 5,
            read_pages: 3,
            domains: Vec::new(),
            days: None,
            max_per_domain: 2,
            source_urls: Vec::new(),
            discover: false,
            fresh: true,
        }
    }
}

impl SearchRequest {
    pub(crate) fn needs_new_links(&self) -> bool {
        self.days.is_some()
            || self
                .query
                .to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .any(|word| {
                    matches!(
                        word,
                        "news"
                            | "noticias"
                            | "recent"
                            | "reciente"
                            | "recientes"
                            | "today"
                            | "hoy"
                            | "últimas"
                            | "últimos"
                            | "latest"
                            | "newest"
                            | "novedades"
                            | "trending"
                    )
                })
    }

    pub(crate) fn validate(&mut self) -> Result<()> {
        self.query = self.query.trim().to_owned();
        if self.query.is_empty()
            || self.query.chars().count() > 400
            || !self.query.chars().any(char::is_alphanumeric)
            || !(1..=10).contains(&self.limit)
            || self.read_pages > 3
            || !(1..=10).contains(&self.max_per_domain)
            || self.domains.len() > 8
            || self.source_urls.len() > 5
            || self.days.is_some_and(|days| !(1..=365).contains(&days))
        {
            bail!(
                "Use query 1–400 characters with letters or numbers, limit 1–10, read_pages 0–3, max_per_domain 1–10, up to 8 domains, up to 5 source_urls and days 1–365"
            );
        }
        for domain in &mut self.domains {
            let normalized = domain.trim().trim_end_matches('.').to_lowercase();
            if normalized.len() > 253
                || normalized.split('.').any(|label| {
                    label.is_empty()
                        || label.len() > 63
                        || label.starts_with('-')
                        || label.ends_with('-')
                        || !label
                            .bytes()
                            .all(|c| c.is_ascii_alphanumeric() || c == b'-')
                })
            {
                bail!("Domain filters must be plain public hostnames, without URLs or wildcards");
            }
            fetch::public_url(&format!("https://{normalized}/"))?;
            *domain = normalized;
        }
        self.domains.sort();
        self.domains.dedup();
        for value in &mut self.source_urls {
            *value = fetch::public_url(value)?.to_string();
        }
        let mut seen = HashSet::new();
        self.source_urls.retain(|url| seen.insert(url.clone()));
        Ok(())
    }
}

pub(crate) struct Selection {
    seen_urls: HashSet<String>,
    seen_content: HashSet<String>,
    hosts: HashMap<String, usize>,
}

impl Selection {
    pub(crate) fn new() -> Self {
        Self {
            seen_urls: HashSet::new(),
            seen_content: HashSet::new(),
            hosts: HashMap::new(),
        }
    }

    pub(crate) fn accept(&mut self, page: &Page, request: &SearchRequest) -> bool {
        if !self.seen_urls.insert(page.document.url.clone())
            || self.seen_content.contains(&page.document.content_hash)
        {
            return false;
        }
        // Apply the current URL policy to legacy indexed pages too. A withdrawn
        // endpoint must not reappear through an old cached search result.
        let Ok(url) = fetch::public_url(&page.document.url) else {
            return false;
        };
        let host = url.host_str().unwrap_or_default();
        let domain = request
            .domains
            .iter()
            .filter(|domain| host == *domain || host.ends_with(&format!(".{domain}")))
            .max_by_key(|domain| domain.len())
            .map(String::as_str)
            .unwrap_or_else(|| host.strip_prefix("www.").unwrap_or(host));
        let count = self.hosts.entry(domain.to_owned()).or_default();
        if *count >= request.max_per_domain {
            return false;
        }
        *count += 1;
        self.seen_content.insert(page.document.content_hash.clone());
        true
    }
}

pub(crate) fn stopword(word: &str) -> bool {
    // A small transparent lexical normalization list, without model calls or
    // generated synonyms. Preserve the original tokens if all are stopwords.
    matches!(
        word,
        "search"
            | "find"
            | "latest"
            | "current"
            | "documentation"
            | "docs"
            | "information"
            | "updated"
            | "busca"
            | "buscar"
            | "búsqueda"
            | "actualizada"
            | "actualizado"
            | "actual"
            | "documentación"
            | "información"
            | "explica"
            | "dime"
            | "quiero"
            | "necesito"
            | "internet"
            | "web"
            | "please"
            | "the"
            | "a"
            | "an"
            | "and"
            | "or"
            | "of"
            | "in"
            | "on"
            | "at"
            | "to"
            | "for"
            | "from"
            | "with"
            | "is"
            | "are"
            | "was"
            | "were"
            | "how"
            | "what"
            | "which"
            | "who"
            | "about"
            | "el"
            | "la"
            | "los"
            | "las"
            | "un"
            | "una"
            | "unos"
            | "unas"
            | "de"
            | "del"
            | "en"
            | "y"
            | "o"
            | "por"
            | "para"
            | "con"
            | "es"
            | "son"
            | "que"
            | "qué"
            | "cómo"
            | "cual"
            | "cuál"
            | "sobre"
    )
}

/// Shared lexical normalization for retrieval, link ranking and passages.
pub(crate) fn terms(query: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    // Product punctuation must not turn Next.js into the generic words 'next'
    // and 'js', which otherwise favor unrelated JavaScript configuration pages.
    let normalized = query
        .to_lowercase()
        .replace("next.js", "nextjs")
        .replace("node.js", "nodejs")
        .replace("vue.js", "vuejs");
    normalized
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_lowercase)
        .filter(|s| !stopword(s) && seen.insert(s.clone()))
        .take(20)
        .collect()
}

/// Reject weak OR matches. This measures lexical support, not factual truth.
pub(crate) fn relevant(title: &str, url: &str, text: &str, query: &str) -> bool {
    let all = terms(query);
    let host = url::Url::parse(url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
        .unwrap_or_default();
    let mut topical: Vec<_> = all
        .iter()
        .filter(|term| term.len() < 3 || !host.split('.').any(|label| label == term.as_str()))
        .collect();
    if topical.is_empty() {
        topical = all.iter().collect();
    }
    if topical.is_empty() {
        return false;
    }
    if topical.len() < all.len() && topical.len() >= 2 {
        let heading = format!(
            "{title} {}",
            url::Url::parse(url)
                .map(|u| u.path().to_owned())
                .unwrap_or_default()
        )
        .to_lowercase();
        if !heading
            .split(|c: char| !c.is_alphanumeric())
            .any(|word| topical.iter().any(|term| equivalent(word, term)))
        {
            return false;
        }
    }
    let haystack = format!("{title} {text}").to_lowercase();
    let words: Vec<_> = haystack
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect();
    let required = if topical.len() <= 3 {
        topical.len()
    } else {
        (topical.len() * 3).div_ceil(5)
    };
    // Require topic terms near each other, not scattered through unrelated cards
    // or a long navigation list. Hostname matches cannot substitute for the topic.
    words.chunks(100).enumerate().any(|(i, _)| {
        let window = &words[i * 100..(i * 100 + 200).min(words.len())];
        topical
            .iter()
            .filter(|term| window.iter().any(|word| equivalent(word, term)))
            .count()
            >= required
    })
}

pub(crate) fn equivalent(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    // Most documentation is ASCII. Avoid allocating two normalized Strings
    // for every word/term comparison in each overlapping passage window.
    if a.is_ascii() && b.is_ascii() {
        return (a.len() > 4 && a.strip_suffix('s') == Some(b))
            || (b.len() > 4 && b.strip_suffix('s') == Some(a));
    }
    fn fold(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                'á' | 'à' | 'â' | 'ä' => 'a',
                'é' | 'è' | 'ê' | 'ë' => 'e',
                'í' | 'ì' | 'î' | 'ï' => 'i',
                'ó' | 'ò' | 'ô' | 'ö' => 'o',
                'ú' | 'ù' | 'û' | 'ü' => 'u',
                _ => c,
            })
            .collect()
    }
    let a = fold(a);
    let b = fold(b);
    a == b
        || (a.len() > 4 && a.strip_suffix('s') == Some(b.as_str()))
        || (b.len() > 4 && b.strip_suffix('s') == Some(a.as_str()))
}

pub(crate) fn topic_terms(query: &str, url: &str) -> Vec<String> {
    let all = terms(query);
    let host = url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_owned))
        .unwrap_or_default();
    let topical: Vec<_> = all
        .iter()
        .filter(|term| term.len() < 3 || !host.split('.').any(|label| label == term.as_str()))
        .cloned()
        .collect();
    if topical.is_empty() { all } else { topical }
}

/// A work-budget heuristic, not a guarantee that the question is answered.
pub(crate) fn strong_source(title: &str, url: &str, text: &str, query: &str) -> bool {
    if text.len() < 400 || !relevant(title, url, text, query) {
        return false;
    }
    let topical = topic_terms(query, url);
    let heading = format!("{title} {url}").to_lowercase();
    let body = text.to_lowercase();
    !topical.is_empty()
        && topical
            .iter()
            .all(|term| body.matches(term.as_str()).take(2).count() >= 2)
        && topical.iter().any(|term| heading.contains(term.as_str()))
}
