//! Independent disk-backed public corpus; never reads conversation or project memory.
use crate::{Config, extract::Document};
use anyhow::{Context, Result, bail};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::Path,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub(crate) struct Index {
    connection: Arc<Mutex<Connection>>,
    config: Config,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Page {
    pub document: Document,
    pub fetched_at: i64,
    pub etag: Option<String>,
    pub modified: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Coverage {
    pub indexed_pages: usize,
    pub indexed_origins: usize,
    pub known_urls: usize,
    pub due_urls: usize,
    pub last_fetch_at: Option<i64>,
    pub requests_today: usize,
    pub max_pages: usize,
    pub page_limit_reached: bool,
}

impl Index {
    pub(crate) fn open(path: &Path, config: &Config) -> Result<Self> {
        let connection = Connection::open(path)?;
        connection.busy_timeout(std::time::Duration::from_secs(3))?;
        connection.execute_batch("PRAGMA journal_mode=WAL;
            PRAGMA cache_size=-2048; PRAGMA mmap_size=0; PRAGMA temp_store=FILE;
            PRAGMA wal_autocheckpoint=256; PRAGMA journal_size_limit=4194304;
            CREATE TABLE IF NOT EXISTS pages (
              id INTEGER PRIMARY KEY, url TEXT NOT NULL UNIQUE, origin TEXT NOT NULL,
              title TEXT NOT NULL, text TEXT NOT NULL, hash TEXT NOT NULL,
              language TEXT, truncated INTEGER NOT NULL, fetched_at INTEGER NOT NULL,
              etag TEXT, modified TEXT);
            CREATE VIRTUAL TABLE IF NOT EXISTS page_fts USING fts5(
              title, text, content='pages', content_rowid='id', tokenize='unicode61 remove_diacritics 2');
            CREATE TRIGGER IF NOT EXISTS pages_ai AFTER INSERT ON pages BEGIN
              INSERT INTO page_fts(rowid,title,text) VALUES(new.id,new.title,new.text); END;
            CREATE TRIGGER IF NOT EXISTS pages_ad AFTER DELETE ON pages BEGIN
              INSERT INTO page_fts(page_fts,rowid,title,text) VALUES('delete',old.id,old.title,old.text); END;
            CREATE TRIGGER IF NOT EXISTS pages_au AFTER UPDATE OF title,text ON pages BEGIN
              INSERT INTO page_fts(page_fts,rowid,title,text) VALUES('delete',old.id,old.title,old.text);
              INSERT INTO page_fts(rowid,title,text) VALUES(new.id,new.title,new.text); END;
            CREATE TABLE IF NOT EXISTS frontier (
              url TEXT PRIMARY KEY, origin TEXT NOT NULL, depth INTEGER NOT NULL,
              due_at INTEGER NOT NULL DEFAULT 0, attempts INTEGER NOT NULL DEFAULT 0,
              last_error TEXT);
            CREATE INDEX IF NOT EXISTS frontier_due ON frontier(due_at,depth);
            CREATE INDEX IF NOT EXISTS frontier_origin ON frontier(origin);
            CREATE TABLE IF NOT EXISTS hosts (
              origin TEXT PRIMARY KEY, robots TEXT, robots_expires INTEGER NOT NULL DEFAULT 0,
              next_request_ms INTEGER NOT NULL DEFAULT 0);
            CREATE TABLE IF NOT EXISTS traffic (
              id INTEGER PRIMARY KEY CHECK(id=1), day TEXT NOT NULL, requests INTEGER NOT NULL);
            PRAGMA user_version=1;")?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            config: config.clone(),
        })
    }

    pub(crate) async fn call<T: Send + 'static>(
        &self,
        f: impl FnOnce(&mut Connection) -> Result<T> + Send + 'static,
    ) -> Result<T> {
        let connection = self.connection.clone();
        tokio::task::spawn_blocking(move || {
            let mut db = connection
                .lock()
                .map_err(|_| anyhow::anyhow!("Web index lock poisoned"))?;
            f(&mut db)
        })
        .await?
    }

    pub(crate) async fn enqueue(&self, urls: Vec<String>, depth: u32) -> Result<()> {
        let config = self.config.clone();
        self.call(move |db| {
            let tx = db.transaction()?;
            let mut count: usize =
                tx.query_row("SELECT count(*) FROM frontier", [], |r| r.get(0))?;
            for value in urls.into_iter().take(100) {
                if count >= config.max_frontier {
                    break;
                }
                let url = reqwest::Url::parse(&value)?;
                let origin = url.origin().ascii_serialization();
                let origin_count: usize = tx.query_row(
                    "SELECT count(*) FROM frontier WHERE origin=?",
                    [&origin],
                    |r| r.get(0),
                )?;
                if origin_count >= config.max_urls_per_origin {
                    continue;
                }
                count += tx.execute(
                    "INSERT OR IGNORE INTO frontier(url,origin,depth) VALUES(?,?,?)",
                    params![value, origin, depth],
                )?;
            }
            tx.commit()?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn next(&self) -> Result<Option<(String, u32)>> {
        let max_pages = self.config.max_pages;
        self.call(move |db| {
            let full: bool =
                db.query_row("SELECT count(*) >= ? FROM pages", [max_pages], |r| r.get(0))?;
            // Lease before network activity; an interrupted crawl becomes due again
            // after one minute, with no permanent 'running' state to restore.
            let tx = db.transaction()?;
            let next = tx
                .query_row(
                    "SELECT url,depth FROM frontier
                WHERE due_at<=? AND (?=0 OR url IN (SELECT url FROM pages))
                ORDER BY due_at,depth,rowid LIMIT 1",
                    params![Utc::now().timestamp(), full],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?)),
                )
                .optional()?;
            if let Some((url, _)) = &next {
                tx.execute(
                    "UPDATE frontier SET due_at=? WHERE url=?",
                    params![Utc::now().timestamp() + 60, url],
                )?;
            }
            tx.commit()?;
            Ok(next)
        })
        .await
    }

    pub(crate) async fn finish(&self, url: &str, error: Option<String>) -> Result<()> {
        let url = url.to_owned();
        let refresh = self.config.refresh_hours as i64 * 3600;
        self.call(move |db| {
            let attempts: u32 = db
                .query_row("SELECT attempts FROM frontier WHERE url=?", [&url], |r| {
                    r.get(0)
                })
                .optional()?
                .unwrap_or(0);
            let delay = if error.is_some() {
                (300_i64 * 2_i64.pow(attempts.min(8))).min(86_400)
            } else {
                refresh
            };
            db.execute(
                "UPDATE frontier SET due_at=?,attempts=?,last_error=? WHERE url=?",
                params![
                    Utc::now().timestamp() + delay,
                    if error.is_some() {
                        attempts.saturating_add(1)
                    } else {
                        0
                    },
                    error.map(|e| crate::extract::clipped(&e, 250)),
                    url
                ],
            )?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn save(&self, page: Page) -> Result<()> {
        let max_pages = self.config.max_pages;
        self.call(move |db| {
            let p = page.document;
            if !p.indexable || p.text.trim().is_empty() {
                db.execute("DELETE FROM pages WHERE url=?", [&p.url])?;
                return Ok(());
            }
            // Existing URLs refresh at capacity; the corpus never grows past its cap.
            db.execute("INSERT INTO pages(url,origin,title,text,hash,language,truncated,fetched_at,etag,modified)
                SELECT ?1,?2,?3,?4,?5,?6,?7,?8,?9,?10
                WHERE (SELECT count(*) FROM pages) < ?11 OR EXISTS(SELECT 1 FROM pages WHERE url=?1)
                ON CONFLICT(url) DO UPDATE SET title=excluded.title,text=excluded.text,
                hash=excluded.hash,language=excluded.language,truncated=excluded.truncated,
                fetched_at=excluded.fetched_at,etag=excluded.etag,modified=excluded.modified",
                params![p.url, reqwest::Url::parse(&p.url)?.origin().ascii_serialization(),p.title,p.text,p.content_hash,p.language,p.truncated,page.fetched_at,page.etag,page.modified,max_pages])?;
            Ok(())
        }).await
    }

    pub(crate) async fn remove(&self, url: &str) -> Result<()> {
        let url = url.to_owned();
        self.call(move |db| {
            db.execute("DELETE FROM pages WHERE url=?", [url])?;
            Ok(())
        })
        .await
    }

    fn page(row: &rusqlite::Row<'_>) -> rusqlite::Result<Page> {
        Ok(Page {
            document: Document {
                url: row.get(0)?,
                title: row.get(1)?,
                text: row.get(2)?,
                content_hash: row.get(3)?,
                language: row.get(4)?,
                truncated: row.get(5)?,
                links: vec![],
                indexable: true,
            },
            fetched_at: row.get(6)?,
            etag: row.get(7)?,
            modified: row.get(8)?,
        })
    }

    pub(crate) async fn get(&self, url: &str) -> Result<Option<Page>> {
        let url = url.to_owned();
        self.call(move |db| Ok(db.query_row("SELECT url,title,text,hash,language,truncated,fetched_at,etag,modified FROM pages WHERE url=?", [url], Self::page).optional()?)).await
    }

    pub(crate) async fn search(&self, query: &str, limit: usize) -> Result<Vec<Page>> {
        let terms: Vec<_> = query
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .take(20)
            .map(|s| format!("\"{s}\""))
            .collect();
        if terms.is_empty() {
            bail!("Search query must contain letters or numbers");
        }
        let exact = terms.join(" AND ");
        let broad = terms.join(" OR ");
        self.call(move |db| {
            let mut statement = db.prepare("SELECT p.url,p.title,p.text,p.hash,p.language,p.truncated,p.fetched_at,p.etag,p.modified
                FROM page_fts JOIN pages p ON p.id=page_fts.rowid
                WHERE page_fts MATCH ? ORDER BY bm25(page_fts,5.0,1.0) LIMIT ?")?;
            let mut pages = Vec::new();
            let mut seen = HashSet::new();
            for expression in [exact, broad] {
                let rows = statement.query_map(params![expression,limit*10], Self::page)?;
                for row in rows {
                    let page = row?;
                    if seen.insert(page.document.content_hash.clone()) { pages.push(page); }
                    if pages.len() >= limit { return Ok(pages); }
                }
            }
            Ok(pages)
        }).await
    }

    pub(crate) async fn coverage(&self) -> Result<Coverage> {
        let max_pages = self.config.max_pages;
        self.call(move |db| {
            let (pages, origins, last): (usize, usize, Option<i64>) = db.query_row(
                "SELECT count(*),count(DISTINCT origin),max(fetched_at) FROM pages",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )?;
            let known = db.query_row("SELECT count(*) FROM frontier", [], |r| r.get(0))?;
            let due = db.query_row(
                "SELECT count(*) FROM frontier WHERE due_at<=?",
                [Utc::now().timestamp()],
                |r| r.get(0),
            )?;
            let requests = db
                .query_row(
                    "SELECT requests FROM traffic WHERE id=1 AND day=?",
                    [Utc::now().format("%Y-%m-%d").to_string()],
                    |r| r.get(0),
                )
                .optional()?
                .unwrap_or(0);
            Ok(Coverage {
                indexed_pages: pages,
                indexed_origins: origins,
                known_urls: known,
                due_urls: due,
                last_fetch_at: last,
                requests_today: requests,
                max_pages,
                page_limit_reached: pages >= max_pages,
            })
        })
        .await
    }

    pub(crate) async fn robots(&self, origin: &str) -> Result<Option<String>> {
        let origin = origin.to_owned();
        self.call(move |db| Ok(db.query_row("SELECT robots FROM hosts WHERE origin=? AND robots IS NOT NULL AND robots_expires>?", params![origin,Utc::now().timestamp()], |r| r.get(0)).optional()?)).await
    }

    pub(crate) async fn save_robots(&self, origin: &str, body: String, ttl: i64) -> Result<()> {
        let origin = origin.to_owned();
        self.call(move |db| {
            db.execute("INSERT INTO hosts(origin,robots,robots_expires) VALUES(?,?,?) ON CONFLICT(origin) DO UPDATE SET robots=excluded.robots,robots_expires=excluded.robots_expires", params![origin,body,Utc::now().timestamp()+ttl])?;
            // Bound ancillary metadata for explicit reads of many unrelated URLs.
            db.execute("DELETE FROM hosts WHERE origin IN (SELECT origin FROM hosts ORDER BY next_request_ms DESC LIMIT -1 OFFSET 2000)", [])?;
            Ok(())
        }).await
    }

    pub(crate) async fn request_wait(&self, origin: &str) -> Result<u64> {
        let origin = origin.to_owned();
        self.call(move |db| {
            let next: i64 = db
                .query_row(
                    "SELECT next_request_ms FROM hosts WHERE origin=?",
                    [origin],
                    |r| r.get(0),
                )
                .optional()?
                .unwrap_or(0);
            Ok(next.saturating_sub(Utc::now().timestamp_millis()).max(0) as u64)
        })
        .await
    }

    /// Charge only an attempt ready to start. A rate-limited waiter neither
    /// advances the host clock nor consumes the daily budget.
    pub(crate) async fn reserve_request(&self, origin: &str, delay_ms: u64) -> Result<u64> {
        let origin = origin.to_owned();
        let budget = self.config.requests_per_day;
        self.call(move |db| {
            let tx = db.transaction()?;
            let day = Utc::now().format("%Y-%m-%d").to_string();
            tx.execute("INSERT INTO traffic(id,day,requests) VALUES(1,?,0) ON CONFLICT(id) DO UPDATE SET day=excluded.day,requests=0 WHERE traffic.day<>excluded.day", [day])?;
            let requests: usize = tx.query_row("SELECT requests FROM traffic WHERE id=1", [], |r| r.get(0))?;
            if requests >= budget { bail!("Daily web request budget reached; cached search is still available"); }
            let now = Utc::now().timestamp_millis();
            let next: i64 = tx.query_row("SELECT next_request_ms FROM hosts WHERE origin=?", [&origin], |r| r.get(0)).optional()?.unwrap_or(now);
            let wait = next.saturating_sub(now).max(0) as u64;
            if wait > 0 { return Ok(wait); }
            tx.execute("INSERT INTO hosts(origin,next_request_ms) VALUES(?,?) ON CONFLICT(origin) DO UPDATE SET next_request_ms=excluded.next_request_ms", params![origin, now.max(next)+delay_ms as i64])?;
            tx.execute("UPDATE traffic SET requests=requests+1 WHERE id=1", [])?;
            tx.commit().context("Persist web request budget")?;
            Ok(wait)
        }).await
    }
}
