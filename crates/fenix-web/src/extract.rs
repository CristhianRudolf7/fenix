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
    pub content_hash: String,
    pub truncated: bool,
    pub indexable: bool,
    pub links: Vec<String>,
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
    let mut language = None;
    let mut indexable = true;
    let (title, mut text) = if content_type.contains("xml") && !content_type.contains("html") {
        let mut reader = Reader::from_str(body);
        reader.config_mut().trim_text(true);
        let mut sitemap = false;
        while links.len() < 100 {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.local_name();
                    if matches!(name.as_ref(), b"urlset" | b"sitemapindex") {
                        sitemap = true;
                    }
                    if name.as_ref() != b"loc" {
                        continue;
                    }
                    let raw = reader.read_text(e.name())?;
                    let value = quick_xml::escape::unescape(&raw)?;
                    // A sitemap describes its own origin; cross-site sitemap spam
                    // does not expand the frontier. Ordinary HTML links can.
                    if let Ok(link) = url.join(&value)
                        && link.origin() == url.origin()
                        && let Ok(link) = crate::fetch::public_url(link.as_str())
                    {
                        links.push(link.to_string());
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }
        if !sitemap {
            bail!("XML is not a supported sitemap");
        }
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
            for e in document.select(&selector("a[href]")) {
                if e.attr("rel").is_some_and(|s| {
                    s.split_whitespace()
                        .any(|s| s.eq_ignore_ascii_case("nofollow"))
                }) {
                    continue;
                }
                if let Ok(link) = url.join(e.attr("href").unwrap_or_default())
                    && let Ok(link) = crate::fetch::public_url(link.as_str())
                    && seen.insert(link.to_string())
                {
                    links.push(link.to_string());
                }
                if links.len() >= 100 {
                    break;
                }
            }
        }
        let root = document
            .select(&selector("main, article, [role=main]"))
            .next()
            .or_else(|| document.select(&selector("body")).next())
            .unwrap_or_else(|| document.root_element());
        let mut output = String::new();
        let mut count = 0;
        for node in root.descendants() {
            let Some(value) = node.value().as_text() else {
                continue;
            };
            let mut ancestors = node.ancestors().filter_map(ElementRef::wrap);
            if ancestors.by_ref().take(256).any(excluded) {
                continue;
            }
            if ancestors.next().is_some() {
                bail!("HTML nesting exceeds the extraction depth limit");
            }
            for word in value.split_whitespace() {
                if !output.is_empty() {
                    output.push(' ');
                    count += 1;
                }
                for ch in word.chars() {
                    output.push(ch);
                    count += 1;
                    if count > max_chars {
                        break;
                    }
                }
                if count > max_chars {
                    break;
                }
            }
            if count > max_chars {
                break;
            }
        }
        (title, output)
    } else if content_type.starts_with("text/plain") || content_type.starts_with("text/markdown") {
        (url.to_string(), clipped(body, max_chars + 1))
    } else {
        bail!("Unsupported content type; only HTML, plain text and XML sitemaps are supported");
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
        content_hash,
        truncated,
        indexable,
        links,
    })
}

pub(crate) fn passage(text: &str, query: &str, limit: usize) -> String {
    // Match in Unicode characters so accents and case expansion cannot produce
    // invalid byte offsets. Keep a bounded window around a relevant query token.
    let chars: Vec<char> = text.chars().collect();
    let terms: Vec<String> = query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.chars().count() > 2)
        .take(12)
        .map(str::to_lowercase)
        .collect();
    let mut first = 0;
    for (i, window) in chars.chunks(256).enumerate() {
        let value: String = window.iter().collect();
        let lower = value.to_lowercase();
        if terms.iter().any(|t| lower.contains(t)) {
            first = (i * 256).saturating_sub(256);
            break;
        }
    }
    chars[first..].iter().take(limit).collect()
}
