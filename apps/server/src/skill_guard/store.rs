//! Host-owned publication queue and immutable snapshots; tenant agents cannot write it.
use super::scanner::POLICY;
use anyhow::{Result, ensure};
use fenix_agent::skills::bundle::Bundle;
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;
use serde_json::Value;
use std::{
    path::Path,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
pub struct Store(Mutex<Connection>);
#[derive(Clone, Debug, Serialize)]
pub struct Publication {
    pub id: String,
    pub name: String,
    pub category: String,
    pub status: String,
    pub public: bool,
    pub source_hash: String,
    pub reviewed_hash: Option<String>,
    pub updated_at: i64,
    pub repaired: bool,
}
pub struct Job {
    pub id: String,
    pub attempt: i64,
    pub bundle: Bundle,
}
impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        let db = Connection::open(path)?;
        db.busy_timeout(std::time::Duration::from_secs(2))?;
        db.execute_batch("PRAGMA journal_mode=WAL;
            CREATE TABLE IF NOT EXISTS publications (
                id TEXT PRIMARY KEY, owner TEXT NOT NULL, name TEXT NOT NULL, category TEXT NOT NULL,
                source_hash TEXT NOT NULL, policy TEXT NOT NULL, original TEXT NOT NULL, candidate TEXT,
                reviewed_hash TEXT, status TEXT NOT NULL, public INTEGER NOT NULL DEFAULT 0,
                report TEXT, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, attempts INTEGER NOT NULL DEFAULT 0,
                UNIQUE(owner,name,source_hash,policy));
            CREATE INDEX IF NOT EXISTS publication_queue ON publications(status,created_at);
            CREATE INDEX IF NOT EXISTS publication_owner ON publications(owner,created_at);
            CREATE INDEX IF NOT EXISTS public_catalog ON publications(public,category,updated_at);")?;
        // A restart never publishes a half-completed review. At most two worker attempts.
        db.execute("UPDATE publications SET status=CASE WHEN attempts<2 THEN 'queued' ELSE 'failed' END WHERE status='reviewing'", [])?;
        let entries = {
            let mut statement = db.prepare("SELECT id,original FROM publications")?;
            statement
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        for (id, source) in entries {
            let bundle: Bundle = serde_json::from_str(&source)?;
            let category = fenix_agent::skills::category_from_content(&bundle.files["SKILL.md"])?;
            db.execute(
                "UPDATE publications SET category=?1 WHERE id=?2",
                params![category, id],
            )?;
        }
        Ok(Self(Mutex::new(db)))
    }
    fn db(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.0
            .lock()
            .map_err(|_| anyhow::anyhow!("Publication store unavailable"))
    }
    pub fn submit(&self, owner: &str, bundle: &Bundle, _category: &str) -> Result<Publication> {
        bundle.validate()?;
        let category = fenix_agent::skills::category_from_content(&bundle.files["SKILL.md"])?;
        let mut db = self.db()?;
        let tx = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let hash = bundle.hash();
        let previous: Option<String> = tx.query_row("SELECT id FROM publications WHERE owner=?1 AND name=?2 AND source_hash=?3 AND policy=?4", params![owner,bundle.name,hash,POLICY], |r| r.get(0)).optional()?;
        if let Some(id) = previous {
            tx.commit()?;
            drop(db);
            return self.get(owner, &id);
        }
        let recent: i64 = tx.query_row(
            "SELECT COUNT(*) FROM publications WHERE owner=?1 AND created_at>?2",
            params![owner, now() - 3600],
            |r| r.get(0),
        )?;
        let daily: i64 = tx.query_row(
            "SELECT COUNT(*) FROM publications WHERE owner=?1 AND created_at>?2",
            params![owner, now() - 86400],
            |r| r.get(0),
        )?;
        let active: i64 = tx.query_row(
            "SELECT COUNT(*) FROM publications WHERE owner=?1 AND status IN ('queued','reviewing')",
            [owner],
            |r| r.get(0),
        )?;
        let global: i64 = tx.query_row(
            "SELECT COUNT(*) FROM publications WHERE status IN ('queued','reviewing')",
            [],
            |r| r.get(0),
        )?;
        ensure!(
            recent < 5 && daily < 20 && active < 2 && global < 64,
            "Publication quota reached; try later"
        );
        let id = uuid::Uuid::new_v4().to_string();
        tx.execute("INSERT INTO publications(id,owner,name,category,source_hash,policy,original,status,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,'queued',?8,?8)", params![id,owner,bundle.name,category,hash,POLICY,serde_json::to_string(bundle)?,now()])?;
        tx.execute("UPDATE publications SET status='cancelled',updated_at=?4 WHERE owner=?1 AND name=?2 AND id!=?3 AND status IN ('queued','reviewing')",params![owner,bundle.name,id,now()])?;
        tx.commit()?;
        drop(db);
        self.get(owner, &id)
    }
    fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Publication> {
        Ok(Publication {
            id: r.get(0)?,
            name: r.get(1)?,
            category: r.get(2)?,
            status: r.get(3)?,
            public: r.get(4)?,
            source_hash: r.get(5)?,
            reviewed_hash: r.get(6)?,
            updated_at: r.get(7)?,
            repaired: r.get(8)?,
        })
    }
    pub fn get(&self, owner: &str, id: &str) -> Result<Publication> {
        Ok(self.db()?.query_row("SELECT id,name,category,status,public,source_hash,reviewed_hash,updated_at,COALESCE(candidate IS NOT NULL AND reviewed_hash!=source_hash,0) FROM publications WHERE owner=?1 AND id=?2",params![owner,id],Self::row)?)
    }
    pub fn mine(&self, owner: &str, offset: usize) -> Result<Vec<Publication>> {
        let db = self.db()?;
        let mut query = db.prepare("SELECT id,name,category,status,public,source_hash,reviewed_hash,updated_at,COALESCE(candidate IS NOT NULL AND reviewed_hash!=source_hash,0) FROM publications WHERE owner=?1 ORDER BY created_at DESC,id LIMIT 50 OFFSET ?2")?;
        Ok(query
            .query_map(params![owner, offset.min(100_000)], Self::row)?
            .collect::<rusqlite::Result<_>>()?)
    }
    pub fn catalog(&self, query: &str, category: &str, offset: usize) -> Result<Vec<Publication>> {
        let db = self.db()?;
        let mut stmt = db.prepare("SELECT id,name,category,status,public,source_hash,reviewed_hash,updated_at,reviewed_hash!=source_hash FROM publications WHERE public=1 AND status='approved' AND (?1='' OR instr(lower(name),lower(?1))>0) AND (?2='' OR category=?2) ORDER BY updated_at DESC,id LIMIT 24 OFFSET ?3")?;
        Ok(stmt
            .query_map(params![query, category, offset.min(100_000)], Self::row)?
            .collect::<rusqlite::Result<_>>()?)
    }
    pub fn report(&self, owner: &str, id: &str) -> Result<Value> {
        let text: Option<String> = self.db()?.query_row(
            "SELECT report FROM publications WHERE owner=?1 AND id=?2",
            params![owner, id],
            |r| r.get(0),
        )?;
        Ok(text
            .map(|s| serde_json::from_str(&s))
            .transpose()?
            .unwrap_or(Value::Null))
    }
    pub fn bundle(&self, owner: &str, id: &str, original: bool) -> Result<Bundle> {
        let text: String = self.db()?.query_row("SELECT CASE WHEN ?3 THEN original ELSE COALESCE(candidate,original) END FROM publications WHERE id=?1 AND (owner=?2 OR (public=1 AND status='approved' AND ?3=0))",params![id,owner,original], |r| r.get(0))?;
        Ok(serde_json::from_str(&text)?)
    }
    pub fn visibility(&self, owner: &str, id: &str, public: bool) -> Result<Publication> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let (name, status, policy): (String, String, String) = tx.query_row(
            "SELECT name,status,policy FROM publications WHERE id=?1 AND owner=?2",
            params![id, owner],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        ensure!(
            !public || (status == "approved" && policy == POLICY),
            "Only a reviewed version can be public"
        );
        if public {
            tx.execute(
                "UPDATE publications SET public=0 WHERE owner=?1 AND name=?2",
                params![owner, name],
            )?;
            tx.execute(
                "UPDATE publications SET public=1,updated_at=?2 WHERE id=?1",
                params![id, now()],
            )?;
        } else {
            // Cancel pending versions too: a late worker must not undo the user's choice.
            tx.execute("UPDATE publications SET public=0,status=CASE WHEN status IN ('queued','reviewing') THEN 'cancelled' ELSE status END,updated_at=?3 WHERE owner=?1 AND name=?2",params![owner,name,now()])?;
        }
        tx.commit()?;
        drop(db);
        self.get(owner, id)
    }
    pub fn retry(&self, owner: &str, id: &str) -> Result<()> {
        let mut db = self.db()?;
        let tx = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let active: i64 = tx.query_row(
            "SELECT COUNT(*) FROM publications WHERE owner=?1 AND status IN ('queued','reviewing')",
            [owner],
            |r| r.get(0),
        )?;
        let global: i64 = tx.query_row(
            "SELECT COUNT(*) FROM publications WHERE status IN ('queued','reviewing')",
            [],
            |r| r.get(0),
        )?;
        ensure!(
            active < 2 && global < 64,
            "Publication quota reached; try later"
        );
        let changed=tx.execute("UPDATE publications SET status='queued',updated_at=?3 WHERE id=?1 AND owner=?2 AND status IN ('failed','cancelled') AND attempts<2 AND updated_at<=?4",params![id,owner,now(),now()-60])?;
        ensure!(
            changed == 1,
            "Retry is unavailable; wait one minute (maximum two attempts per version)"
        );
        tx.execute("UPDATE publications SET status='cancelled',updated_at=?3 WHERE owner=?1 AND id!=?2 AND name=(SELECT name FROM publications WHERE id=?2) AND status IN ('queued','reviewing')",params![owner,id,now()])?;
        tx.commit()?;
        Ok(())
    }
    pub fn claim(&self) -> Result<Option<Job>> {
        let mut db = self.db()?;
        let tx = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let row: Option<(String,String,i64)> = tx.query_row("SELECT id,original,attempts+1 FROM publications WHERE status='queued' ORDER BY created_at,id LIMIT 1",[], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?;
        let Some((id, text, attempt)) = row else {
            return Ok(None);
        };
        tx.execute("UPDATE publications SET status='reviewing',attempts=attempts+1,updated_at=?2 WHERE id=?1",params![id,now()])?;
        tx.commit()?;
        Ok(Some(Job {
            id,
            attempt,
            bundle: serde_json::from_str(&text)?,
        }))
    }
    pub fn is_current(&self, id: &str, attempt: i64) -> Result<bool> {
        Ok(self.db()?.query_row("SELECT EXISTS(SELECT 1 FROM publications WHERE id=?1 AND status='reviewing' AND attempts=?2)",params![id,attempt],|r|r.get(0))?)
    }
    pub fn finish(
        &self,
        id: &str,
        attempt: i64,
        status: &str,
        candidate: Option<&Bundle>,
        report: &Value,
    ) -> Result<()> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        if status == "approved" {
            tx.execute("UPDATE publications SET public=0 WHERE (owner,name) IN (SELECT owner,name FROM publications WHERE id=?1 AND status='reviewing' AND attempts=?2)",params![id,attempt])?;
        }
        tx.execute("UPDATE publications SET status=?2,public=(?2='approved'),candidate=?3,reviewed_hash=?4,report=?5,updated_at=?6 WHERE id=?1 AND status='reviewing' AND attempts=?7",params![id,status,candidate.map(serde_json::to_string).transpose()?,candidate.map(Bundle::hash),report.to_string(),now(),attempt])?;
        tx.commit()?;
        Ok(())
    }
}
