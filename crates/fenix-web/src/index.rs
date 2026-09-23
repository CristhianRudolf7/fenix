//! Independent disk-backed public corpus; never reads conversation or project memory.
use crate::{Config, SearchRequest, extract::Document, search::Selection};
use anyhow::{Context, Result, bail};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub(crate) struct Index {
    connection: Arc<Mutex<Connection>>,
    config: Config,
}

pub(crate) struct SearchPages {
    pub pages: Vec<Page>,
    pub broadened: bool,
    pub candidates_truncated: bool,
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
    pub priority_urls: usize,
    pub last_fetch_at: Option<i64>,
    pub requests_today: usize,
    pub max_pages: usize,
    pub page_limit_reached: bool,
}

impl Index {
    pub(crate) fn open(path: &Path, config: &Config) -> Result<Self> {
        let mut connection = Connection::open(path)?;
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
            CREATE TABLE IF NOT EXISTS frontier_priority (
              url TEXT PRIMARY KEY, queued_at INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS frontier_edges (
              source_origin TEXT NOT NULL, target TEXT NOT NULL,
              PRIMARY KEY(source_origin,target));
            CREATE TABLE IF NOT EXISTS hosts (
              origin TEXT PRIMARY KEY, robots TEXT, robots_expires INTEGER NOT NULL DEFAULT 0,
              next_request_ms INTEGER NOT NULL DEFAULT 0);
            CREATE TABLE IF NOT EXISTS traffic (
              id INTEGER PRIMARY KEY CHECK(id=1), day TEXT NOT NULL, requests INTEGER NOT NULL);
")?;
        // Additive migration: existing indexed text and FTS rows stay intact;
        // legacy pages have unknown publication dates until they are refreshed.
        let migration =
            connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let version: i64 = migration.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if version > 3 {
            bail!("Web index schema is newer than this executable supports");
        }
        let has_publication_date = migration
            .prepare("PRAGMA table_info(pages)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<rusqlite::Result<Vec<_>>>()?
            .iter()
            .any(|name| name == "published_at");
        if !has_publication_date {
            migration.execute("ALTER TABLE pages ADD COLUMN published_at INTEGER", [])?;
        }
        let has_label = migration
            .prepare("PRAGMA table_info(frontier)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<rusqlite::Result<Vec<_>>>()?
            .iter()
            .any(|name| name == "label");
        if !has_label {
            migration.execute(
                "ALTER TABLE frontier ADD COLUMN label TEXT NOT NULL DEFAULT ''",
                [],
            )?;
        }
        migration.execute_batch("CREATE INDEX IF NOT EXISTS pages_publication ON pages(published_at); PRAGMA user_version=3;")?;
        migration.commit()?;
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
            let mut origin_counts = HashMap::<String, usize>::new();
            let mut insert =
                tx.prepare("INSERT OR IGNORE INTO frontier(url,origin,depth) VALUES(?,?,?)")?;
            for value in urls.into_iter().take(2_000) {
                if count >= config.max_frontier {
                    break;
                }
                let url = reqwest::Url::parse(&value)?;
                let origin = url.origin().ascii_serialization();
                let origin_count = match origin_counts.entry(origin.clone()) {
                    std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(tx.query_row(
                            "SELECT count(*) FROM frontier WHERE origin=?",
                            [&origin],
                            |r| r.get(0),
                        )?)
                    }
                };
                if *origin_count >= config.max_urls_per_origin {
                    continue;
                }
                let added = insert.execute(params![value, origin, depth])?;
                count += added;
                *origin_count += added;
            }
            drop(insert);
            tx.commit()?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn label_links(&self, labels: Vec<(String, String)>) -> Result<()> {
        self.call(move |db| {
            let tx = db.transaction()?;
            let mut update = tx.prepare("UPDATE frontier SET label=CASE WHEN instr(label,?1)>0 THEN label ELSE substr(trim(label||' '||?1),1,1000) END WHERE url=?2")?;
            for (url, label) in labels.into_iter().take(2_000) {
                if !label.trim().is_empty() {
                    let normalized = label
                        .to_lowercase()
                        .split(|c: char| !c.is_alphanumeric())
                        .filter(|word| !word.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ");
                    update.execute(params![crate::extract::clipped(&normalized, 1_000), url])?;
                }
            }
            drop(update);
            tx.commit()?;
            Ok(())
        })
        .await
    }

    /// Keep bounded provenance so simultaneous queries cannot lend unrelated
    /// frontier URLs to each other's discovery traversals.
    pub(crate) async fn record_edges(&self, source: &str, targets: Vec<String>) -> Result<()> {
        let origin = reqwest::Url::parse(source)?.origin().ascii_serialization();
        let cap = self.config.max_frontier * 4;
        self.call(move |db| {
            let tx = db.transaction()?;
            let mut insert = tx.prepare("INSERT OR IGNORE INTO frontier_edges(source_origin,target) VALUES(?,?)")?;
            for target in targets.into_iter().take(2_000) {
                insert.execute(params![origin, target])?;
            }
            drop(insert);
            tx.execute("DELETE FROM frontier_edges WHERE rowid IN (SELECT rowid FROM frontier_edges ORDER BY rowid DESC LIMIT -1 OFFSET ?)", [cap])?;
            tx.commit()?;
            Ok(())
        }).await
    }

    /// Rank observed URLs on disk. Labels only discover leads; they never
    /// become article evidence in the full-text index.
    pub(crate) async fn candidates(
        &self,
        request: &SearchRequest,
        navigation_origins: Vec<String>,
    ) -> Result<Vec<String>> {
        let all_terms = crate::search::terms(&request.query);
        let topical: Vec<_> = all_terms
            .iter()
            .filter(|term| {
                !navigation_origins.iter().any(|origin| {
                    reqwest::Url::parse(origin)
                        .ok()
                        .and_then(|url| url.host_str().map(str::to_owned))
                        .is_some_and(|host| {
                            term.len() >= 3 && host.split('.').any(|label| label == term.as_str())
                        })
                })
            })
            .cloned()
            .collect();
        let terms = if topical.is_empty() {
            all_terms
        } else {
            topical
        };
        if terms.is_empty() {
            return Ok(vec![]);
        }
        let domains = request.domains.clone();
        self.call(move |db| {
            let mut parameters = Vec::<rusqlite::types::Value>::new();
            let mut scores = Vec::new();
            for term in terms {
                parameters.push(term.into());
                let position = parameters.len();
                // Short acronyms must match label words: 'HN' must not send
                // the crawler to an unrelated URL containing 'john'.
                let path_words = "' '||replace(replace(replace(replace(lower(substr(url,length(origin)+1)),'/',' '),'-',' '),'_',' '),'.',' ')||' '";
                let core = format!("(CASE WHEN (length(?{position})<3 AND instr(' '||label||' ',' '||?{position}||' ')>0) OR (length(?{position})>=3 AND instr(label,?{position})>0) THEN 3 ELSE 0 END + CASE WHEN length(?{position})>=3 AND instr(lower(substr(url,length(origin)+1)),?{position})>0 THEN 4 ELSE 0 END + CASE WHEN instr({path_words},' '||?{position}||' ')>0 THEN 4 ELSE 0 END + CASE WHEN instr(' '||label||' ',' '||?{position}||' ')>0 THEN 2 ELSE 0 END)");
                scores.push(core);
            }
            if !navigation_origins.is_empty() {
                let placeholders = navigation_origins.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                parameters.extend(navigation_origins.iter().cloned().map(Into::into));
                scores.push(format!("CASE WHEN origin IN ({placeholders}) THEN 0.25 ELSE 0 END"));
                parameters.extend(navigation_origins.iter().cloned().map(Into::into));
                scores.push(format!("CASE WHEN origin IN ({placeholders}) AND (label IN ('docs','documentation','learn','reference','api reference','library reference','library','module index','modules','manual','manuals','guides','table of contents','contents','documentation index','documentación','índice') OR substr(url,length(origin)+1) LIKE '/reference/%' OR substr(url,length(origin)+1) LIKE '/docs/%') THEN 0.5 ELSE 0 END"));
                let map_placeholders = navigation_origins.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                parameters.extend(navigation_origins.iter().cloned().map(Into::into));
                scores.push(format!("CASE WHEN origin IN ({map_placeholders}) AND (url LIKE '%sitemap%.xml' OR url LIKE '%/llms.txt') THEN 2 ELSE 0 END"));
            }
            // Query-directed exploration may use observed edges even when the
            // background frontier is full. A large sitemap must not prevent a
            // later relevant link from being read. Canonical URLs always have
            // a slash after the authority; extract that bounded public origin.
            let mut sql = format!("WITH links AS (
                SELECT url,origin,label,depth,due_at,last_error FROM frontier
                UNION
                SELECT target,substr(target,1,instr(substr(target,instr(target,'://')+3),'/')+instr(target,'://')+1),'',1,0,NULL
                FROM frontier_edges e WHERE NOT EXISTS (SELECT 1 FROM frontier f WHERE f.url=e.target)
                ) SELECT url, ({}) AS relevance FROM links WHERE relevance>0.25 AND (last_error IS NULL OR due_at<=?)", scores.join("+"));
            parameters.push(Utc::now().timestamp().into());
            if !navigation_origins.is_empty() {
                let scope = navigation_origins.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                sql.push_str(&format!(" AND (origin IN ({scope}) OR url IN (SELECT target FROM frontier_edges WHERE source_origin IN ({scope})))"));
                parameters.extend(navigation_origins.iter().cloned().map(Into::into));
                parameters.extend(navigation_origins.into_iter().map(Into::into));
            }
            if !domains.is_empty() {
                let mut allowed = Vec::new();
                for domain in domains {
                    allowed.push("(origin IN (?,?) OR origin LIKE ? OR origin LIKE ?)");
                    for value in [format!("https://{domain}"),format!("http://{domain}"),format!("https://%.{domain}"),format!("http://%.{domain}")] { parameters.push(value.into()); }
                }
                sql.push_str(&format!(" AND ({})", allowed.join(" OR ")));
            }
            sql.push_str(" ORDER BY relevance DESC, depth, length(url), url LIMIT 100");
            Ok(db.prepare(&sql)?.query_map(rusqlite::params_from_iter(parameters), |row| row.get(0))?
                .collect::<rusqlite::Result<Vec<String>>>()?)
        }).await
    }

    /// Prioritize observed, due links without overriding retry or refresh times.
    /// Only URLs already in the bounded frontier can enter this queue.
    pub(crate) async fn prioritize(&self, urls: Vec<String>) -> Result<usize> {
        self.call(move |db| {
            let tx = db.transaction()?;
            let now = Utc::now().timestamp();
            let mut queued = 0;
            for url in urls.into_iter().take(16) {
                queued += tx.execute("INSERT OR IGNORE INTO frontier_priority(url,queued_at) SELECT url,?2 FROM frontier WHERE url=?1 AND due_at<=?2", params![url, now])?;
            }
            tx.execute("DELETE FROM frontier_priority WHERE url IN (SELECT url FROM frontier_priority ORDER BY queued_at DESC,rowid DESC LIMIT -1 OFFSET 256)", [])?;
            tx.commit()?;
            Ok(queued)
        }).await
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
                    "SELECT f.url,f.depth FROM frontier f LEFT JOIN frontier_priority p ON p.url=f.url
                WHERE f.due_at<=? AND (?=0 OR f.url IN (SELECT url FROM pages))
                ORDER BY (p.url IS NULL),p.queued_at,f.due_at,f.depth,f.rowid LIMIT 1",
                    params![Utc::now().timestamp(), full],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?)),
                )
                .optional()?;
            if let Some((url, _)) = &next {
                tx.execute(
                    "UPDATE frontier SET due_at=? WHERE url=?",
                    params![Utc::now().timestamp() + 60, url],
                )?;
                tx.execute("DELETE FROM frontier_priority WHERE url=?", [url])?;
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
            db.execute("INSERT INTO pages(url,origin,title,text,hash,language,truncated,fetched_at,etag,modified,published_at)
                SELECT ?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?12
                WHERE (SELECT count(*) FROM pages) < ?11 OR EXISTS(SELECT 1 FROM pages WHERE url=?1)
                ON CONFLICT(url) DO UPDATE SET title=excluded.title,text=excluded.text,
                hash=excluded.hash,language=excluded.language,truncated=excluded.truncated,
                fetched_at=excluded.fetched_at,etag=excluded.etag,modified=excluded.modified,published_at=excluded.published_at",
                params![p.url, reqwest::Url::parse(&p.url)?.origin().ascii_serialization(),p.title,p.text,p.content_hash,p.language,p.truncated,page.fetched_at,page.etag,page.modified,max_pages,p.published_at])?;
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
                published_at: row.get(9)?,
                truncated: row.get(5)?,
                links: vec![],
                link_labels: vec![],
                indexable: true,
            },
            fetched_at: row.get(6)?,
            etag: row.get(7)?,
            modified: row.get(8)?,
        })
    }

    pub(crate) async fn get(&self, url: &str) -> Result<Option<Page>> {
        let url = url.to_owned();
        self.call(move |db| Ok(db.query_row("SELECT url,title,text,hash,language,truncated,fetched_at,etag,modified,published_at FROM pages WHERE url=?", [url], Self::page).optional()?)).await
    }

    #[cfg(test)]
    pub(crate) async fn search(&self, query: &str, limit: usize) -> Result<Vec<Page>> {
        self.search_with(&SearchRequest {
            query: query.into(),
            limit,
            ..SearchRequest::default()
        })
        .await
        .map(|result| result.pages)
    }

    pub(crate) async fn search_with(&self, request: &SearchRequest) -> Result<SearchPages> {
        let tokens: Vec<_> = request
            .query
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .take(40)
            .map(str::to_lowercase)
            .collect();
        if tokens.is_empty() {
            bail!("Search query must contain letters or numbers");
        }
        let useful: Vec<_> = tokens
            .iter()
            .filter(|term| !crate::search::stopword(term))
            .collect();
        let terms: Vec<_> = if useful.is_empty() {
            tokens.iter().collect()
        } else {
            useful
        }
        .into_iter()
        .take(20)
        .map(|s| format!("\"{s}\""))
        .collect();
        let exact = terms.join(" AND ");
        let broad = terms.join(" OR ");
        let request = request.clone();
        self.call(move |db| {
            let mut sql = String::from("SELECT p.url,p.title,p.text,p.hash,p.language,p.truncated,p.fetched_at,p.etag,p.modified,p.published_at
                FROM page_fts JOIN pages p ON p.id=page_fts.rowid WHERE page_fts MATCH ?");
            let mut filters = Vec::<rusqlite::types::Value>::new();
            if let Some(days) = request.days {
                let now = Utc::now().timestamp();
                sql.push_str(" AND p.published_at BETWEEN ? AND ?");
                filters.push((now - i64::from(days) * 86_400).into());
                filters.push(now.into());
            }
            if !request.domains.is_empty() {
                sql.push_str(" AND (");
                for (i, domain) in request.domains.iter().enumerate() {
                    if i > 0 { sql.push_str(" OR "); }
                    // No LIKE: literal suffix comparison preserves DNS boundaries
                    // and never interprets caller strings as SQL wildcards.
                    sql.push_str("(p.origin = ? OR p.origin = ? OR substr(p.origin, -length(?)) = ?)");
                    filters.push(format!("http://{domain}").into());
                    filters.push(format!("https://{domain}").into());
                    filters.push(format!(".{domain}").into());
                    filters.push(format!(".{domain}").into());
                }
                sql.push(')');
            }
            let mut title_scores = Vec::new();
            for term in crate::search::terms(&request.query) {
                title_scores.push("CASE WHEN instr(lower(p.title),?)>0 THEN 1 ELSE 0 END");
                filters.push(term.into());
            }
            let title_score = if title_scores.is_empty() { "0.0".to_owned() } else { title_scores.join("+") };
            sql.push_str(&format!(" ORDER BY ({title_score}) DESC,bm25(page_fts,5.0,1.0),p.url LIMIT 1001"));
            let mut statement = db.prepare(&sql)?;
            let mut pages = Vec::new();
            let mut selection = Selection::new();
            let mut broadened = false;
            let mut candidates_truncated = false;
            for (pass, expression) in [exact, broad].into_iter().enumerate() {
                let values = std::iter::once(rusqlite::types::Value::from(expression)).chain(filters.iter().cloned());
                let rows = statement.query_map(rusqlite::params_from_iter(values), Self::page)?;
                for (index, row) in rows.enumerate() {
                    if index >= 1000 {
                        candidates_truncated = true;
                        break;
                    }
                    let page = row?;
                    if crate::search::relevant(&page.document.title, &page.document.url, &page.document.text, &request.query) && selection.accept(&page, &request) {
                        broadened |= pass == 1;
                        pages.push(page);
                    }
                    if pages.len() >= request.limit { return Ok(SearchPages { pages, broadened, candidates_truncated }); }
                }
            }
            Ok(SearchPages { pages, broadened, candidates_truncated })
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
            let priority_urls =
                db.query_row("SELECT count(*) FROM frontier_priority", [], |r| r.get(0))?;
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
                priority_urls,
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
