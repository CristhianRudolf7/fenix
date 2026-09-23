//! Bounded HTML/text/XML extraction. No JavaScript, model calls or browser process.
use anyhow::{Result, bail};
use quick_xml::{Reader, events::Event};
use reqwest::Url;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Document {
    pub url: String,
    pub title: String,
    pub text: String,
    pub language: Option<String>,
    #[serde(default)]
    pub published_at: Option<i64>,
    pub content_hash: String,
    pub truncated: bool,
    pub indexable: bool,
    pub links: Vec<String>,
    #[serde(default)]
    pub link_labels: Vec<(String, String)>,
}

pub(crate) fn clipped(text: &str, chars: usize) -> String {
    text.chars().take(chars).collect()
}

fn selector(value: &str) -> Selector {
    Selector::parse(value).expect("static extraction selector")
}

fn excluded(element: ElementRef<'_>) -> bool {
    matches!(
        element.value().name(),
        "script"
            | "style"
            | "noscript"
            | "template"
            | "svg"
            | "nav"
            | "footer"
            | "header"
            | "aside"
            | "form"
    ) || element.attr("hidden").is_some()
        || element.attr("aria-hidden") == Some("true")
        || element.attr("style").is_some_and(|s| {
            let s = s.to_ascii_lowercase().replace(' ', "");
            s.contains("display:none") || s.contains("visibility:hidden")
        })
}

pub(crate) fn extract(
    url: &Url,
    body: &str,
    content_type: &str,
    max_chars: usize,
) -> Result<Document> {
    let mut links = Vec::new();
    let mut link_labels = Vec::new();
    let mut language = None;
    let mut published_at = None;
    let mut indexable = true;
    let (title, mut text) = if content_type.contains("xml") && !content_type.contains("html") {
        (links, link_labels) = discovery_links(url, body)?;
        indexable = false;
        (String::new(), String::new())
    } else if content_type.contains("html") {
        if body.bytes().filter(|b| *b == b'<').take(20_001).count() > 20_000 {
            bail!("HTML document exceeds the structural complexity limit");
        }
        let document = Html::parse_document(body);
        let title = document
            .select(&selector("title"))
            .next()
            .map(|e| e.text().collect::<String>())
            .unwrap_or_else(|| url.to_string());
        language = document
            .select(&selector("html[lang]"))
            .next()
            .and_then(|e| e.attr("lang"))
            .map(|s| clipped(s, 40));
        // Publication dates must come from an explicit publisher field. Fetch
        // timestamps, sitemap lastmod and arbitrary time elements are not dates
        // of publication and must never make an old document look recent.
        published_at = document
            .select(&selector("meta[content]"))
            .filter(|e| {
                e.attr("property")
                    .is_some_and(|p| p.eq_ignore_ascii_case("article:published_time"))
                    || e.attr("itemprop").is_some_and(|p| p == "datePublished")
                    || e.attr("name").is_some_and(|p| p == "datePublished")
            })
            .filter_map(|e| publication_date(e.attr("content").unwrap_or_default()))
            .min();
        let structured_date = document
            .select(&selector("script[type='application/ld+json']"))
            .filter_map(|element| {
                let value: serde_json::Value =
                    serde_json::from_str(&element.text().collect::<String>()).ok()?;
                // Only a top-level page record, not arbitrary nested dates from
                // recommended stories or comments elsewhere in a JSON-LD graph.
                publication_date(value.get("datePublished")?.as_str()?)
            })
            .chain(
                document
                    .select(&selector("time[itemprop='datePublished'][datetime]"))
                    .filter_map(|element| publication_date(element.attr("datetime")?)),
            )
            .min();
        published_at = published_at.into_iter().chain(structured_date).min();
        // HN exposes the submission's UTC timestamp on its own permalink.
        // A comment date or a front-page age must not date a different story.
        if url.host_str() == Some("news.ycombinator.com")
            && url.path() == "/item"
            && let Some((_, id)) = url.query_pairs().find(|(key, _)| key == "id")
        {
            published_at = document.select(&selector(".age[title]")).find_map(|age| {
                let matches = age.select(&selector("a[href]")).any(|anchor| {
                    url.join(anchor.attr("href").unwrap_or_default())
                        .is_ok_and(|link| {
                            link.origin() == url.origin()
                                && link.path() == "/item"
                                && link
                                    .query_pairs()
                                    .any(|(key, value)| key == "id" && value == id)
                        })
                });
                if !matches {
                    return None;
                }
                let date = age.attr("title")?.split_whitespace().next()?;
                publication_date(date).or_else(|| publication_date(&format!("{date}Z")))
            });
        }
        let mut follow = true;
        for e in document.select(&selector("meta[name]")) {
            if e.attr("name").is_some_and(|n| {
                n.eq_ignore_ascii_case("robots") || n.eq_ignore_ascii_case("fenixbot")
            }) {
                let rules = e.attr("content").unwrap_or_default().to_ascii_lowercase();
                let rules: Vec<_> = rules
                    .split(|c: char| c == ',' || c.is_whitespace())
                    .collect();
                indexable &= !rules
                    .iter()
                    .any(|r| matches!(*r, "noindex" | "none" | "noarchive"));
                follow &= !rules.iter().any(|r| matches!(*r, "nofollow" | "none"));
            }
        }
        if follow {
            let mut seen = HashSet::new();
            // Static documentation may publish a no-JavaScript contents
            // frame. Only follow observed same-origin navigation URLs.
            for fallback in document.select(&selector("noscript")).take(8) {
                let navigation = fallback
                    .ancestors()
                    .filter_map(ElementRef::wrap)
                    .any(|parent| parent.value().name() == "nav");
                if !navigation {
                    continue;
                }
                let raw = fallback.text().collect::<String>();
                let fragment = Html::parse_fragment(&raw);
                for frame in fragment
                    .select(&selector("iframe[src]"))
                    .chain(fallback.select(&selector("iframe[src]")))
                {
                    if let Ok(link) = url.join(frame.attr("src").unwrap_or_default())
                        && link.origin() == url.origin()
                        && let Ok(link) = crate::fetch::public_url(link.as_str())
                        && seen.insert(link.to_string())
                    {
                        links.push(link.to_string());
                        link_labels.push((link.to_string(), "table of contents".into()));
                    }
                    if links.len() >= 8 {
                        break;
                    }
                }
                if links.len() >= 8 {
                    break;
                }
            }
            for e in document.select(&selector("a[href], link[rel=alternate][href]")) {
                if e.attr("rel").is_some_and(|s| {
                    s.split_whitespace()
                        .any(|s| s.eq_ignore_ascii_case("nofollow"))
                }) {
                    continue;
                }
                if let Ok(link) = url.join(e.attr("href").unwrap_or_default())
                    && let Ok(link) = crate::fetch::public_url(link.as_str())
                {
                    let label = e.text().collect::<Vec<_>>().join(" ");
                    let key = link.to_string();
                    if seen.insert(key.clone()) {
                        links.push(key.clone());
                    }
                    if let Some((_, existing)) =
                        link_labels.iter_mut().find(|(target, _)| target == &key)
                    {
                        if !label.trim().is_empty() && !existing.contains(label.trim()) {
                            *existing = clipped(&format!("{existing} {}", label.trim()), 1_000);
                        }
                    } else {
                        link_labels.push((key, clipped(label.trim(), 1_000)));
                    }
                }
                if links.len() >= 2_000 {
                    break;
                }
            }
            // Link the discussion permalink to its story title, not the
            // uninformative "42 comments" text. Only annotate observed links.
            if url.host_str() == Some("news.ycombinator.com") {
                for story in document.select(&selector("tr.athing[id]")) {
                    if let (Some(id), Some(title)) = (
                        story.attr("id"),
                        story.select(&selector(".titleline")).next(),
                    ) {
                        let permalink = format!("https://news.ycombinator.com/item?id={id}");
                        if links.contains(&permalink) {
                            link_labels.retain(|(link, _)| link != &permalink);
                            link_labels.push((
                                permalink,
                                clipped(&title.text().collect::<Vec<_>>().join(" "), 300),
                            ));
                        }
                    }
                }
            }
        }
        let article_selector = selector("article");
        let mut articles = document.select(&article_selector);
        let article = articles.next().filter(|_| articles.next().is_none());
        let root = article
            .or_else(|| document.select(&selector("main, [role=main]")).next())
            .or_else(|| document.select(&selector("body")).next())
            .unwrap_or_else(|| document.root_element());
        let mut output = String::new();
        let mut count = 0;
        let mut previous_block = None;
        for node in root.descendants() {
            let Some(value) = node.value().as_text() else {
                continue;
            };
            let ancestors: Vec<_> = node
                .ancestors()
                .filter_map(ElementRef::wrap)
                .take(257)
                .collect();
            if ancestors.len() > 256 {
                bail!("HTML nesting exceeds the extraction depth limit");
            }
            if ancestors.iter().any(|element| excluded(*element)) {
                continue;
            }
            let pre = ancestors
                .iter()
                .find(|element| element.value().name() == "pre");
            let block = ancestors.iter().find(|element| {
                matches!(
                    element.value().name(),
                    "pre"
                        | "p"
                        | "div"
                        | "section"
                        | "li"
                        | "h1"
                        | "h2"
                        | "h3"
                        | "h4"
                        | "tr"
                        | "blockquote"
                ) || (pre.is_some()
                    && element.attr("class").is_some_and(|classes| {
                        classes
                            .split_whitespace()
                            .any(|class| matches!(class, "line" | "token-line" | "cm-line"))
                    }))
            });
            let block_id = block.map(|element| element.id());
            let piece = if pre.is_some() {
                value.to_string()
            } else {
                value.split_whitespace().collect::<Vec<_>>().join(" ")
            };
            if piece.is_empty() {
                continue;
            }
            if !output.is_empty() {
                if block_id != previous_block && !output.ends_with('\n') {
                    output.push('\n');
                    count += 1;
                } else if pre.is_none() && !output.ends_with([' ', '\n']) {
                    output.push(' ');
                    count += 1;
                }
            }
            previous_block = block_id;
            for ch in piece.chars().take((max_chars + 1).saturating_sub(count)) {
                output.push(ch);
                count += 1;
            }
            if count > max_chars {
                break;
            }
        }
        (title, output)
    } else if content_type.starts_with("text/plain") || content_type.starts_with("text/markdown") {
        for line in body.lines() {
            let mut rest = line;
            while let Some(start) = rest.find("](") {
                let label = rest[..start]
                    .rsplit_once('[')
                    .map(|(_, label)| label)
                    .unwrap_or("");
                let target = &rest[start + 2..];
                let Some(end) = target.find(')') else {
                    break;
                };
                if let Ok(link) = url.join(target[..end].trim())
                    && let Ok(link) = crate::fetch::public_url(link.as_str())
                    && links.len() < 2_000
                    && !links.contains(&link.to_string())
                {
                    link_labels.push((link.to_string(), clipped(label, 300)));
                    links.push(link.to_string());
                }
                rest = &target[end + 1..];
            }
        }
        let title = body
            .lines()
            .find_map(|line| line.strip_prefix("# "))
            .map(str::to_owned)
            .unwrap_or_else(|| url.to_string());
        (title, clipped(body, max_chars + 1))
    } else {
        bail!(
            "Unsupported content type; only HTML, plain text, XML sitemaps and RSS/Atom feeds are supported"
        );
    };
    let truncated = text.chars().count() > max_chars;
    if truncated {
        text = clipped(&text, max_chars);
    }
    let content_hash = format!("{:x}", Sha256::digest(text.as_bytes()));
    Ok(Document {
        url: url.to_string(),
        title: clipped(title.trim(), 300),
        text,
        language,
        published_at,
        content_hash,
        truncated,
        indexable,
        links,
        link_labels,
    })
}

fn publication_date(value: &str) -> Option<i64> {
    let value = value.trim();
    let seconds = chrono::DateTime::parse_from_rfc3339(value)
        .map(|date| date.timestamp())
        .ok()
        .or_else(|| {
            // Date-only metadata is represented as UTC midnight, preserving its
            // declared day rather than substituting the indexing time.
            (value.len() == 10)
                .then(|| chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
                .flatten()?
                .and_hms_opt(0, 0, 0)
                .map(|date| date.and_utc().timestamp())
        })?;
    (seconds >= 0 && seconds <= chrono::Utc::now().timestamp()).then_some(seconds)
}

/// Sitemaps and feeds discover source URLs; their excerpts are not articles.
type DiscoveryLinks = (Vec<String>, Vec<(String, String)>);

fn discovery_links(url: &Url, body: &str) -> Result<DiscoveryLinks> {
    let mut reader = Reader::from_str(body);
    reader.config_mut().trim_text(true);
    let mut sitemap = false;
    let mut feed = false;
    let mut root_seen = false;
    let mut entry_depth = 0_u32;
    let mut depth = 0_u32;
    let mut links = Vec::new();
    let mut seen = HashSet::new();
    let mut labels = Vec::new();
    let mut entry_title = String::new();
    let mut entry_links = Vec::new();
    while links.len() < 2_000 {
        let event = reader.read_event()?;
        match event {
            Event::Start(ref e) | Event::Empty(ref e) => {
                let name = e.local_name();
                let name = name.as_ref();
                let empty = matches!(event, Event::Empty(_));
                if !empty {
                    depth += 1;
                    if depth > 256 {
                        bail!("XML nesting exceeds the extraction depth limit");
                    }
                }
                if !root_seen {
                    root_seen = true;
                    sitemap = matches!(name, b"urlset" | b"sitemapindex");
                    feed = matches!(name, b"rss" | b"feed" | b"RDF");
                }
                if feed && matches!(name, b"item" | b"entry") && !empty {
                    entry_depth = depth;
                    entry_title.clear();
                    entry_links.clear();
                }
                let mut raw = None;
                if feed && entry_depth > 0 && name == b"title" && !empty {
                    entry_title = clipped(
                        &quick_xml::escape::unescape(&reader.read_text(e.name())?)?,
                        300,
                    );
                    depth -= 1;
                } else if sitemap && name == b"loc" && !empty {
                    raw = Some(
                        quick_xml::escape::unescape(&reader.read_text(e.name())?)?.into_owned(),
                    );
                    depth -= 1;
                } else if feed && entry_depth > 0 && name == b"link" {
                    let mut alternate = true;
                    for attribute in e.attributes().flatten() {
                        if attribute.key.local_name().as_ref() == b"rel" {
                            alternate = attribute.value.as_ref() == b"alternate";
                        }
                        if attribute.key.local_name().as_ref() == b"href" {
                            raw = Some(
                                attribute
                                    .decode_and_unescape_value(reader.decoder())?
                                    .into_owned(),
                            );
                        }
                    }
                    if raw.is_none() && !empty {
                        raw = Some(
                            quick_xml::escape::unescape(&reader.read_text(e.name())?)?.into_owned(),
                        );
                        depth -= 1;
                    }
                    if !alternate {
                        raw = None;
                    }
                }
                if let Some(raw) = raw
                    && let Ok(link) = url.join(raw.trim())
                    // Sitemaps describe their origin; feeds can cite originals.
                    && (!sitemap || link.origin() == url.origin())
                    && let Ok(link) = crate::fetch::public_url(link.as_str())
                    && seen.insert(link.to_string())
                {
                    if entry_depth > 0 {
                        entry_links.push(link.to_string());
                    }
                    links.push(link.to_string());
                }
            }
            Event::End(_) => {
                if depth == entry_depth {
                    labels.extend(
                        entry_links
                            .drain(..)
                            .map(|link| (link, entry_title.clone())),
                    );
                    entry_depth = 0;
                }
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !sitemap && !feed {
        bail!("XML is not a supported sitemap, RSS or Atom feed");
    }
    labels.extend(
        entry_links
            .into_iter()
            .map(|link| (link, entry_title.clone())),
    );
    Ok((links, labels))
}

pub(crate) fn passage(text: &str, query: &str, limit: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= limit {
        return text.to_owned();
    }
    let terms = crate::search::terms(query);
    let mut best = (0, 0);
    for start in (0..chars.len()).step_by(256) {
        let window: String = chars[start..].iter().take(limit).collect();
        let lower = window.to_lowercase();
        let score: usize = terms
            .iter()
            .map(|term| {
                let occurrences = lower.matches(term.as_str()).take(8).count();
                if occurrences > 0 {
                    100 + occurrences
                } else {
                    0
                }
            })
            .sum();
        if score > best.1 {
            best = (start, score);
        }
    }
    // Prefer a line boundary so code indentation survives passage selection.
    let start = (best.0.saturating_sub(256)..best.0)
        .rev()
        .find(|&i| chars[i] == '\n')
        .map_or(best.0, |i| i + 1);
    chars[start..].iter().take(limit).collect()
}
