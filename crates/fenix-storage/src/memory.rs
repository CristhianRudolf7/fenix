use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub title: String,
    pub content: String,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub task_id: Option<String>,
}

const MEMORY_COLUMNS: &str = "id,title,content,source,created_at,updated_at,project_id,task_id";
const JOINED_MEMORY_COLUMNS: &str =
    "m.id,m.title,m.content,m.source,m.created_at,m.updated_at,m.project_id,m.task_id";

impl Store {
    /// Owner administration can update a memory without changing its scope.
    pub fn save_memory(
        &self,
        memory_id: Option<&str>,
        title: &str,
        content: &str,
        source: &str,
    ) -> Result<Memory> {
        let previous = memory_id
            .map(|id| {
                self.db()?
                    .query_row(
                        &format!("SELECT {MEMORY_COLUMNS} FROM memories WHERE id=?1"),
                        [id],
                        memory_row,
                    )
                    .optional()?
                    .context("Memory not found")
            })
            .transpose()?;
        self.save_memory_scoped(
            memory_id,
            title,
            content,
            source,
            previous.as_ref().and_then(|m| m.project_id.as_deref()),
            previous.as_ref().and_then(|m| m.task_id.as_deref()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn save_memory_scoped(
        &self,
        memory_id: Option<&str>,
        title: &str,
        content: &str,
        source: &str,
        project_id: Option<&str>,
        task_id: Option<&str>,
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
        if let Some(project_id) = project_id {
            projects::query_project(&tx, project_id)?;
        }
        if let Some(task_id) = task_id {
            let binding = projects::read_binding(&tx, task_id)?;
            if !binding.bound
                || !matches!(binding.mode.as_str(), "general" | "project")
                || binding.project_id.as_deref() != project_id
            {
                bail!("Memory provenance task does not belong to this scope");
            }
        }
        let previous = if let Some(id) = memory_id {
            Some(
                tx.query_row(
                    &format!(
                        "SELECT {MEMORY_COLUMNS} FROM memories WHERE id=?1 AND project_id IS ?2"
                    ),
                    params![id, project_id],
                    memory_row,
                )
                .optional()?
                .context("Memory not found in this scope")?,
            )
        } else {
            None
        };
        if let Some(existing) = tx
            .query_row(
                &format!(
                    "SELECT {MEMORY_COLUMNS} FROM memories WHERE normalized=?1 AND project_id IS ?2"
                ),
                params![normalized, project_id],
                memory_row,
            )
            .optional()?
        {
            if memory_id.is_none() {
                return Ok(existing);
            }
            if memory_id != Some(existing.id.as_str()) {
                bail!("Another memory in this scope already contains this fact");
            }
        }
        let memory = Memory {
            id: memory_id.map(str::to_owned).unwrap_or_else(id),
            title: title.trim().into(),
            content: content.trim().into(),
            source: source.into(),
            created_at: previous
                .as_ref()
                .map(|p| p.created_at.clone())
                .unwrap_or_else(now),
            updated_at: now(),
            project_id: project_id.map(str::to_owned),
            task_id: task_id
                .map(str::to_owned)
                .or_else(|| previous.and_then(|memory| memory.task_id)),
        };
        tx.execute("INSERT INTO memories(id,title,content,source,normalized,created_at,updated_at,project_id,task_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9) ON CONFLICT(id) DO UPDATE SET title=excluded.title,content=excluded.content,source=excluded.source,normalized=excluded.normalized,updated_at=excluded.updated_at,task_id=excluded.task_id",params![memory.id,memory.title,memory.content,memory.source,normalized,memory.created_at,memory.updated_at,memory.project_id,memory.task_id])?;
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
            return Ok(db
                .prepare(&format!(
                    "SELECT {MEMORY_COLUMNS} FROM memories ORDER BY updated_at DESC LIMIT ?1"
                ))?
                .query_map([limit.min(200)], memory_row)?
                .collect::<rusqlite::Result<_>>()?);
        }
        if let Some(exact) = db
            .query_row(
                &format!("SELECT {MEMORY_COLUMNS} FROM memories WHERE id=?1"),
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
        Ok(db.prepare(&format!("SELECT {JOINED_MEMORY_COLUMNS} FROM memory_fts f JOIN memories m ON m.id=f.id WHERE memory_fts MATCH ?1 ORDER BY rank LIMIT ?2"))?.query_map(params![query,limit.min(200)],memory_row)?.collect::<rusqlite::Result<_>>()?)
    }

    pub fn memories_scoped(
        &self,
        query: &str,
        limit: usize,
        project_id: Option<&str>,
        include_global: bool,
    ) -> Result<Vec<Memory>> {
        let db = self.db()?;
        let scope = "(project_id IS ?1 OR (?2 AND project_id IS NULL))";
        if query.trim().is_empty() {
            return Ok(db.prepare(&format!("SELECT {MEMORY_COLUMNS} FROM memories WHERE {scope} ORDER BY updated_at DESC LIMIT ?3"))?.query_map(params![project_id,include_global,limit.min(200)],memory_row)?.collect::<rusqlite::Result<_>>()?);
        }
        if let Some(exact) = db
            .query_row(
                &format!("SELECT {MEMORY_COLUMNS} FROM memories WHERE {scope} AND id=?3"),
                params![project_id, include_global, query],
                memory_row,
            )
            .optional()?
        {
            return Ok(vec![exact]);
        }
        let Some(query) = lexical_query(query) else {
            return Ok(vec![]);
        };
        Ok(db.prepare(&format!("SELECT {JOINED_MEMORY_COLUMNS} FROM memory_fts f JOIN memories m ON m.id=f.id WHERE (m.project_id IS ?1 OR (?2 AND m.project_id IS NULL)) AND memory_fts MATCH ?3 ORDER BY rank LIMIT ?4"))?.query_map(params![project_id,include_global,query,limit.min(200)],memory_row)?.collect::<rusqlite::Result<_>>()?)
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

    pub fn delete_memory_scoped(&self, id: &str, project_id: Option<&str>) -> Result<()> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        if tx.execute(
            "DELETE FROM memories WHERE id=?1 AND project_id IS ?2",
            params![id, project_id],
        )? == 0
        {
            bail!("Memory not found in this scope");
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
        project_id: r.get(6)?,
        task_id: r.get(7)?,
    })
}
