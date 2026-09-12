use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub title: String,
    pub content: String,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

impl Store {
    pub fn save_memory(
        &self,
        memory_id: Option<&str>,
        title: &str,
        content: &str,
        source: &str,
    ) -> Result<Memory> {
        if title.trim().is_empty()
            || content.trim().is_empty()
            || title.len() > 200
            || content.len() > 16_000
        {
            bail!("Memory needs a title (1–200 bytes) and content (1–16000 bytes)");
        }
        let normalized = content
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let previous = if let Some(id) = memory_id {
            Some(tx.query_row("SELECT id,title,content,source,created_at,updated_at FROM memories WHERE id=?1",[id],memory_row).optional()?.context("Memory not found")?)
        } else {
            None
        };
        if let Some(existing)=tx.query_row("SELECT id,title,content,source,created_at,updated_at FROM memories WHERE normalized=?1",[&normalized],memory_row).optional()? {
            if memory_id.is_none() { return Ok(existing); }
            if memory_id!=Some(existing.id.as_str()) { bail!("Another memory already contains this fact"); }
        }
        let memory = Memory {
            id: memory_id.map(str::to_owned).unwrap_or_else(id),
            title: title.trim().into(),
            content: content.trim().into(),
            source: source.into(),
            created_at: previous.map(|p| p.created_at).unwrap_or_else(now),
            updated_at: now(),
        };
        tx.execute("INSERT INTO memories(id,title,content,source,normalized,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(id) DO UPDATE SET title=excluded.title,content=excluded.content,source=excluded.source,normalized=excluded.normalized,updated_at=excluded.updated_at",params![memory.id,memory.title,memory.content,memory.source,normalized,memory.created_at,memory.updated_at])?;
        tx.execute("DELETE FROM memory_fts WHERE id=?1", [&memory.id])?;
        tx.execute(
            "INSERT INTO memory_fts(id,title,content) VALUES (?1,?2,?3)",
            params![memory.id, memory.title, memory.content],
        )?;
        tx.commit()?;
        Ok(memory)
    }

    pub fn memories(&self, query: &str, limit: usize) -> Result<Vec<Memory>> {
        let db = self.db()?;
        if query.trim().is_empty() {
            return Ok(db.prepare("SELECT id,title,content,source,created_at,updated_at FROM memories ORDER BY updated_at DESC LIMIT ?1")?.query_map([limit.min(200)],memory_row)?.collect::<rusqlite::Result<_>>()?);
        }
        if let Some(exact) = db
            .query_row(
                "SELECT id,title,content,source,created_at,updated_at FROM memories WHERE id=?1",
                [query],
                memory_row,
            )
            .optional()?
        {
            return Ok(vec![exact]);
        }
        let Some(query) = lexical_query(query) else {
            return Ok(vec![]);
        };
        Ok(db.prepare("SELECT m.id,m.title,m.content,m.source,m.created_at,m.updated_at FROM memory_fts f JOIN memories m ON m.id=f.id WHERE memory_fts MATCH ?1 ORDER BY rank LIMIT ?2")?.query_map(params![query,limit.min(200)],memory_row)?.collect::<rusqlite::Result<_>>()?)
    }
    pub fn delete_memory(&self, id: &str) -> Result<()> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        if tx.execute("DELETE FROM memories WHERE id=?1", [id])? == 0 {
            bail!("Memory not found");
        }
        tx.execute("DELETE FROM memory_fts WHERE id=?1", [id])?;
        tx.commit()?;
        Ok(())
    }
}
fn memory_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Memory> {
    Ok(Memory {
        id: r.get(0)?,
        title: r.get(1)?,
        content: r.get(2)?,
        source: r.get(3)?,
        created_at: r.get(4)?,
        updated_at: r.get(5)?,
    })
}
