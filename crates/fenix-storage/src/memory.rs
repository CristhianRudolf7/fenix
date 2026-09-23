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
    #[serde(default)]
    pub metadata: MemoryMetadata,
}

impl Memory {
    pub(crate) fn safe_for_context(&self) -> bool {
        !fenix_core::content_trust::instruction_risk(&self.title)
            && !fenix_core::content_trust::instruction_risk(&self.content)
            && !self
                .metadata
                .evidence
                .iter()
                .any(|e| fenix_core::content_trust::instruction_risk(&e.quote))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MemoryMetadata {
    pub kind: String,
    pub status: String,
    pub assertion: String,
    pub entity_id: Option<String>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub supersedes: Option<String>,
    pub evidence: Vec<MemoryEvidence>,
    pub pinned: bool,
}
impl Default for MemoryMetadata {
    fn default() -> Self {
        Self {
            kind: "fact".into(),
            status: "confirmed".into(),
            assertion: "explicit".into(),
            entity_id: None,
            valid_from: None,
            valid_until: None,
            supersedes: None,
            evidence: vec![],
            pinned: false,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryEvidence {
    pub reference: String,
    pub quote: String,
}
impl MemoryMetadata {
    pub fn validate(&self) -> Result<()> {
        if ![
            "fact",
            "preference",
            "decision",
            "assumption",
            "commitment",
            "constraint",
        ]
        .contains(&self.kind.as_str())
            || ![
                "candidate",
                "confirmed",
                "superseded",
                "expired",
                "withdrawn",
                "conflict",
            ]
            .contains(&self.status.as_str())
            || !["explicit", "extracted", "inferred"].contains(&self.assertion.as_str())
            || self.evidence.len() > 32
        {
            bail!("Invalid memory metadata");
        }
        if self.assertion == "inferred" && self.status == "confirmed" {
            bail!("An inference cannot be confirmed as an explicit fact");
        }
        for value in [&self.valid_from, &self.valid_until].into_iter().flatten() {
            chrono::DateTime::parse_from_rfc3339(value).context("Memory dates must be RFC3339")?;
        }
        if let (Some(a), Some(b)) = (&self.valid_from, &self.valid_until)
            && chrono::DateTime::parse_from_rfc3339(a)? >= chrono::DateTime::parse_from_rfc3339(b)?
        {
            bail!("Memory validity interval is empty");
        }
        if self.evidence.iter().any(|e| {
            e.reference.is_empty()
                || e.reference.len() > 500
                || e.quote.is_empty()
                || e.quote.len() > 4000
        }) {
            bail!("Evidence requires a bounded reference and quote");
        }
        Ok(())
    }
    pub fn active(&self) -> bool {
        let instant = chrono::Utc::now();
        self.status == "confirmed"
            && self
                .valid_from
                .as_ref()
                .is_none_or(|v| chrono::DateTime::parse_from_rfc3339(v).is_ok_and(|d| d <= instant))
            && self
                .valid_until
                .as_ref()
                .is_none_or(|v| chrono::DateTime::parse_from_rfc3339(v).is_ok_and(|d| d > instant))
    }
}
pub(crate) fn normalized(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

const MEMORY_COLUMNS: &str =
    "id,title,content,source,created_at,updated_at,project_id,task_id,metadata";
const JOINED_MEMORY_COLUMNS: &str =
    "m.id,m.title,m.content,m.source,m.created_at,m.updated_at,m.project_id,m.task_id,m.metadata";

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
        self.save_memory_record(memory_id, title, content, source, project_id, task_id, None)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn save_memory_record(
        &self,
        memory_id: Option<&str>,
        title: &str,
        content: &str,
        source: &str,
        project_id: Option<&str>,
        task_id: Option<&str>,
        metadata: Option<MemoryMetadata>,
    ) -> Result<Memory> {
        static REDACTOR: std::sync::LazyLock<fenix_core::safety::Redactor> =
            std::sync::LazyLock::new(fenix_core::safety::Redactor::default);
        let safe_title = REDACTOR.text(title);
        let safe_content = REDACTOR.text(content);
        let safe_source = REDACTOR.text(source);
        let (title, content, source) = (
            safe_title.as_str(),
            safe_content.as_str(),
            safe_source.as_str(),
        );
        if title.trim().is_empty()
            || content.trim().is_empty()
            || title.len() > 200
            || content.len() > 16_000
        {
            bail!("Memory needs a title (1–200 bytes) and content (1–16000 bytes)");
        }
        let normalized = normalized(content);
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
        let mut metadata = metadata.unwrap_or_else(|| {
            previous
                .as_ref()
                .map(|m| m.metadata.clone())
                .unwrap_or_default()
        });
        for evidence in &mut metadata.evidence {
            evidence.quote = REDACTOR.text(&evidence.quote);
        }
        metadata.validate()?;
        // Suspicious directives remain reviewable data, never durable authority.
        let instruction_risk = fenix_core::content_trust::instruction_risk(title)
            || fenix_core::content_trust::instruction_risk(content)
            || metadata
                .evidence
                .iter()
                .any(|e| fenix_core::content_trust::instruction_risk(&e.quote));
        if instruction_risk
            && previous
                .as_ref()
                .is_some_and(|m| m.metadata.status == "confirmed" && m.safe_for_context())
        {
            bail!(
                "Risky instruction content cannot replace a confirmed memory; save it separately as a candidate for review"
            );
        }
        if instruction_risk && matches!(metadata.status.as_str(), "candidate" | "confirmed") {
            metadata.status = "candidate".into();
            metadata.pinned = false;
            metadata.supersedes = None;
        }

        if let Some(entity) = &metadata.entity_id {
            super::knowledge::entity_in_scope(&tx, entity, project_id)?;
        }
        let withdrawn: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM memory_withdrawals WHERE scope=?1 AND normalized=?2)",
            params![project_id.unwrap_or(""), normalized],
            |r| r.get(0),
        )?;
        if withdrawn {
            bail!("This claim was withdrawn; save a corrected claim instead");
        }
        if let Some(old) = &metadata.supersedes {
            if Some(old.as_str()) == memory_id {
                bail!("A memory cannot supersede itself");
            }
            let exists: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM memories WHERE id=?1 AND project_id IS ?2)",
                params![old, project_id],
                |r| r.get(0),
            )?;
            if !exists || metadata.status != "confirmed" || metadata.assertion != "explicit" {
                bail!("Supersession requires an explicit confirmed correction in the same scope");
            }
        }
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
                if instruction_risk
                    && existing.metadata.status == "confirmed"
                    && existing.safe_for_context()
                {
                    bail!(
                        "Risky evidence cannot alter a confirmed memory; review the source separately"
                    );
                }
                let mut merged = existing.clone();
                if instruction_risk
                    && matches!(merged.metadata.status.as_str(), "candidate" | "confirmed")
                {
                    merged.metadata.status = "candidate".into();
                    merged.metadata.pinned = false;
                }
                for evidence in &metadata.evidence {
                    tx.execute(
                        "INSERT OR IGNORE INTO memory_observations VALUES (?1,?2)",
                        params![existing.id, evidence.reference],
                    )?;
                    if !merged
                        .metadata
                        .evidence
                        .iter()
                        .any(|e| e.reference == evidence.reference)
                        && merged.metadata.evidence.len() < 32
                    {
                        merged.metadata.evidence.push(evidence.clone());
                    }
                }
                let observations: i64 = tx.query_row(
                    "SELECT COUNT(*) FROM memory_observations WHERE memory_id=?1",
                    [&existing.id],
                    |r| r.get(0),
                )?;
                if !instruction_risk
                    && merged.metadata.status == "candidate"
                    && merged.metadata.assertion != "inferred"
                    && (metadata.status == "confirmed"
                        || (merged.metadata.kind == "preference" && observations >= 3))
                {
                    merged.metadata.status = "confirmed".into();
                }
                if !instruction_risk
                    && merged.metadata.status == "expired"
                    && metadata.status == "confirmed"
                    && metadata.assertion == "explicit"
                    && metadata.active()
                {
                    merged.metadata = metadata.clone();
                }
                if serde_json::to_value(&merged.metadata)?
                    != serde_json::to_value(&existing.metadata)?
                {
                    merged.updated_at = now();
                    tx.execute(
                        "UPDATE memories SET metadata=?2,updated_at=?3 WHERE id=?1",
                        params![
                            existing.id,
                            serde_json::to_string(&merged.metadata)?,
                            merged.updated_at
                        ],
                    )?;
                }
                tx.commit()?;
                return Ok(merged);
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
            metadata,
            project_id: project_id.map(str::to_owned),
            task_id: task_id
                .map(str::to_owned)
                .or_else(|| previous.and_then(|memory| memory.task_id)),
        };
        tx.execute("INSERT INTO memories(id,title,content,source,normalized,created_at,updated_at,project_id,task_id,metadata) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10) ON CONFLICT(id) DO UPDATE SET title=excluded.title,content=excluded.content,source=excluded.source,normalized=excluded.normalized,updated_at=excluded.updated_at,task_id=excluded.task_id,metadata=excluded.metadata",params![memory.id,memory.title,memory.content,memory.source,normalized,memory.created_at,memory.updated_at,memory.project_id,memory.task_id,serde_json::to_string(&memory.metadata)?])?;
        tx.execute("DELETE FROM memory_fts WHERE id=?1", [&memory.id])?;
        tx.execute(
            "INSERT INTO memory_fts(id,title,content) VALUES (?1,?2,?3)",
            params![memory.id, memory.title, memory.content],
        )?;
        for evidence in &memory.metadata.evidence {
            tx.execute(
                "INSERT OR IGNORE INTO memory_observations VALUES (?1,?2)",
                params![memory.id, evidence.reference],
            )?;
        }
        if let Some(old) = &memory.metadata.supersedes {
            tx.execute("UPDATE memories SET metadata=json_set(metadata,'$.status','superseded'),updated_at=?2 WHERE id=?1",params![old,now()])?;
        }
        tx.commit()?;
        Ok(memory)
    }

    pub fn memories(&self, query: &str, limit: usize) -> Result<Vec<Memory>> {
        let db = self.db()?;
        if query.trim().is_empty() {
            return Ok(db
                .prepare(&format!(
                    "SELECT {MEMORY_COLUMNS} FROM memories WHERE coalesce(json_extract(metadata,'$.status'),'confirmed') != 'withdrawn' ORDER BY updated_at DESC LIMIT ?1"
                ))?
                .query_map([limit.min(200)], memory_row)?
                .collect::<rusqlite::Result<_>>()?);
        }
        if let Some(exact) = db
            .query_row(
                &format!("SELECT {MEMORY_COLUMNS} FROM memories WHERE id=?1 AND coalesce(json_extract(metadata,'$.status'),'confirmed') != 'withdrawn'"),
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
        let scope = "(project_id IS ?1 OR (?2 AND project_id IS NULL)) AND coalesce(json_extract(metadata,'$.status'),'confirmed') != 'withdrawn'";
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
        let memory = self
            .memories(id, 1)?
            .into_iter()
            .find(|m| m.id == id)
            .context("Memory not found")?;
        self.delete_memory_scoped(id, memory.project_id.as_deref())
    }
    pub fn delete_memory_scoped(&self, id: &str, project_id: Option<&str>) -> Result<()> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let content: String = tx
            .query_row(
                "SELECT normalized FROM memories WHERE id=?1 AND project_id IS ?2",
                params![id, project_id],
                |r| r.get(0),
            )
            .optional()?
            .context("Memory not found in this scope")?;
        tx.execute(
            "INSERT OR IGNORE INTO memory_withdrawals VALUES (?1,?2)",
            params![project_id.unwrap_or(""), content],
        )?;
        tx.execute("UPDATE memories SET metadata=json_set(metadata,'$.status','withdrawn'),updated_at=?2 WHERE id=?1",params![id,now()])?;
        tx.execute("DELETE FROM memory_fts WHERE id=?1", [id])?;
        tx.commit()?;
        Ok(())
    }
}

pub(crate) fn memory_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Memory> {
    Ok(Memory {
        id: r.get(0)?,
        title: r.get(1)?,
        content: r.get(2)?,
        source: r.get(3)?,
        created_at: r.get(4)?,
        updated_at: r.get(5)?,
        project_id: r.get(6)?,
        task_id: r.get(7)?,
        metadata: serde_json::from_str(&r.get::<_, String>(8)?).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(8, rusqlite::types::Type::Text, Box::new(e))
        })?,
    })
}
