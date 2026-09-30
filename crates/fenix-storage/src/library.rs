//! Versioned document evidence. Search scope is applied before ranking.
use crate::{Store, id, now};
use anyhow::{Context, Result, ensure};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryDocument {
    pub id: String,
    pub project_id: Option<String>,
    pub title: String,
    pub collection: String,
    pub tags: Vec<String>,
    pub enabled: bool,
    pub revision: u64,
    pub filename: String,
    pub sha256: String,
    pub bytes: usize,
    pub format: String,
    pub status: String,
    pub embedding_status: String,
    pub error: Option<String>,
    pub warnings: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryChunk {
    pub id: String,
    pub document_id: String,
    pub revision: u64,
    pub ordinal: usize,
    pub text: String,
    pub location: Value,
    pub hash: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryHit {
    #[serde(flatten)]
    pub chunk: LibraryChunk,
    pub title: String,
    pub project_id: Option<String>,
    pub score: f64,
    pub branches: Vec<String>,
    pub citation: String,
}
fn chunk_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<LibraryChunk> {
    let location: String = row.get(5)?;
    Ok(LibraryChunk {
        id: row.get(0)?,
        document_id: row.get(1)?,
        revision: row.get(2)?,
        ordinal: row.get(3)?,
        text: row.get(4)?,
        location: serde_json::from_str(&location).unwrap_or(Value::Null),
        hash: row.get(6)?,
    })
}
impl Store {
    pub fn library_document(&self, id: &str, revision: Option<u64>) -> Result<LibraryDocument> {
        let db = self.db()?;
        let data: String = if let Some(rev) = revision {
            db.query_row(
                "SELECT data FROM library_revisions WHERE document_id=?1 AND revision=?2",
                params![id, rev],
                |r| r.get(0),
            )
            .optional()?
        } else {
            db.query_row(
                "SELECT data FROM library_documents WHERE id=?1",
                [id],
                |r| r.get(0),
            )
            .optional()?
        }
        .context("Document not found")?;
        Ok(serde_json::from_str(&data)?)
    }
    pub fn library_documents(
        &self,
        project: Option<&str>,
        all: bool,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<LibraryDocument>> {
        self.library_documents_filtered(project, all, offset, limit, "", "")
    }
    pub fn library_documents_filtered(
        &self,
        project: Option<&str>,
        all: bool,
        offset: usize,
        limit: usize,
        name: &str,
        format: &str,
    ) -> Result<Vec<LibraryDocument>> {
        let db = self.db()?;
        let mut q=db.prepare("SELECT data FROM library_documents WHERE (?2 OR project_id IS ?1)
            AND (?5 = '' OR instr(lower(json_extract(data,'$.title')), lower(?5)) > 0 OR instr(lower(json_extract(data,'$.filename')), lower(?5)) > 0)
            AND (?6 = '' OR json_extract(data,'$.format') = ?6)
            ORDER BY json_extract(data,'$.updated_at') DESC,id LIMIT ?3 OFFSET ?4")?;
        q.query_map(
            params![project, all, limit.min(101), offset, name, format],
            |r| r.get::<_, String>(0),
        )?
        .map(|r| Ok(serde_json::from_str(&r?)?))
        .collect()
    }
    /// Deduplication never crosses a project or account boundary.
    pub fn library_duplicate(
        &self,
        project: Option<&str>,
        hash: &str,
        format: &str,
    ) -> Result<Option<LibraryDocument>> {
        let raw: Option<String> = self.db()?.query_row(
            "SELECT data FROM library_documents WHERE project_id IS ?1 AND json_extract(data,'$.sha256')=?2 AND json_extract(data,'$.format')=?3 LIMIT 1",
            params![project,hash,format], |r| r.get(0)).optional()?;
        raw.map(|s| Ok(serde_json::from_str(&s)?)).transpose()
    }
    /// The caller writes the immutable original before publishing this row.
    pub fn library_save(&self, doc: &LibraryDocument, expected: Option<u64>) -> Result<()> {
        if let Some(project) = &doc.project_id {
            self.project(project)?;
        }
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let old: Option<(u64, Option<String>)> = tx
            .query_row(
                "SELECT revision,project_id FROM library_documents WHERE id=?1",
                [&doc.id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        match old {
            Some((rev, scope)) => ensure!(
                expected == Some(rev) && doc.revision == rev + 1 && scope == doc.project_id,
                "Document changed or belongs to another project"
            ),
            None => ensure!(
                expected.is_none() && doc.revision == 1,
                "Document not found or invalid initial revision"
            ),
        }
        let data = serde_json::to_string(doc)?;
        tx.execute("INSERT INTO library_documents VALUES (?1,?2,?3,?4,?5) ON CONFLICT(id) DO UPDATE SET revision=excluded.revision,data=excluded.data",params![doc.id,doc.project_id,doc.enabled,doc.revision,data])?;
        tx.execute(
            "INSERT INTO library_revisions VALUES (?1,?2,?3,?4,?5)",
            params![doc.id, doc.revision, doc.status, doc.embedding_status, data],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn library_metadata(
        &self,
        id: &str,
        expected: u64,
        title: String,
        collection: String,
        tags: Vec<String>,
        enabled: bool,
    ) -> Result<LibraryDocument> {
        ensure!(
            !title.trim().is_empty()
                && title.len() <= 240
                && collection.len() <= 120
                && tags.len() <= 20
                && tags.iter().all(|t| t.len() <= 80),
            "Invalid document metadata"
        );
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let data: String = tx
            .query_row(
                "SELECT data FROM library_documents WHERE id=?1 AND revision=?2",
                params![id, expected],
                |r| r.get(0),
            )
            .optional()?
            .context("Document changed; reload it")?;
        let mut doc: LibraryDocument = serde_json::from_str(&data)?;
        doc.title = title;
        doc.collection = collection;
        doc.tags = tags;
        doc.enabled = enabled;
        doc.updated_at = now();
        let data = serde_json::to_string(&doc)?;
        tx.execute(
            "UPDATE library_documents SET enabled=?2,data=?3 WHERE id=?1",
            params![id, enabled, data],
        )?;
        tx.execute(
            "UPDATE library_revisions SET data=?3 WHERE document_id=?1 AND revision=?2",
            params![id, expected, data],
        )?;
        tx.execute("UPDATE library_fts SET title=?2 WHERE id IN (SELECT id FROM library_chunks WHERE document_id=?1)",params![id,doc.title])?;
        tx.commit()?;
        Ok(doc)
    }
    pub fn library_revisions(&self, id: &str) -> Result<Vec<LibraryDocument>> {
        let db = self.db()?;
        let mut q = db.prepare(
            "SELECT data FROM library_revisions WHERE document_id=?1 ORDER BY revision DESC",
        )?;
        q.query_map([id], |r| r.get::<_, String>(0))?
            .map(|r| Ok(serde_json::from_str(&r?)?))
            .collect()
    }
    pub fn library_delete(&self, id: &str, expected: u64) -> Result<()> {
        ensure!(
            self.db()?.execute(
                "DELETE FROM library_documents WHERE id=?1 AND revision=?2",
                params![id, expected]
            )? == 1,
            "Document changed; reload it"
        );
        Ok(())
    }
    pub fn library_recover(&self) -> Result<()> {
        let docs = {
            let db = self.db()?;
            let mut q =
                db.prepare("SELECT data FROM library_revisions WHERE status='processing'")?;
            q.query_map([], |r| r.get::<_, String>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        for raw in docs {
            let mut doc: LibraryDocument = serde_json::from_str(&raw)?;
            doc.status = "queued".into();
            self.library_state(&doc)?;
        }
        Ok(())
    }
    pub fn library_job(&self, embeddings: bool) -> Result<Option<LibraryDocument>> {
        let db = self.db()?;
        let raw:Option<String>=db.query_row("SELECT r.data FROM library_revisions r JOIN library_documents d ON d.id=r.document_id AND d.revision=r.revision WHERE (NOT ?1 AND r.status='queued') OR (?1 AND d.enabled=1 AND r.status IN ('ready','partial') AND r.embedding_status='pending') ORDER BY json_extract(r.data,'$.created_at') LIMIT 1",[embeddings],|r|r.get(0)).optional()?;
        raw.map(|r| Ok(serde_json::from_str(&r)?)).transpose()
    }
    pub fn library_state(&self, doc: &LibraryDocument) -> Result<()> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        // Metadata may change while extraction is running: retain its current values.
        let raw: Option<String> = tx
            .query_row(
                "SELECT data FROM library_revisions WHERE document_id=?1 AND revision=?2",
                params![doc.id, doc.revision],
                |r| r.get(0),
            )
            .optional()?;
        let Some(raw) = raw else { return Ok(()) };
        let mut merged: LibraryDocument = serde_json::from_str(&raw)?;
        merged.status = doc.status.clone();
        merged.embedding_status = doc.embedding_status.clone();
        merged.error = doc.error.clone();
        merged.warnings = doc.warnings.clone();
        merged.updated_at = now();
        let data = serde_json::to_string(&merged)?;
        tx.execute("UPDATE library_revisions SET status=?3,embedding_status=?4,data=?5 WHERE document_id=?1 AND revision=?2",params![doc.id,doc.revision,doc.status,doc.embedding_status,data])?;
        tx.execute(
            "UPDATE library_documents SET data=?3 WHERE id=?1 AND revision=?2",
            params![doc.id, doc.revision, data],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn library_store_chunks(
        &self,
        doc: &LibraryDocument,
        chunks: &[LibraryChunk],
    ) -> Result<()> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        for (ordinal, c) in chunks.iter().enumerate() {
            ensure!(
                c.ordinal == ordinal
                    && c.document_id == doc.id
                    && c.revision == doc.revision
                    && !c.text.trim().is_empty()
                    && c.text.chars().count() <= 2400,
                "Invalid document fragment"
            );
            // Reuse exact text in this or an older version of the same private document.
            let cached:Option<(String,String)>=tx.query_row("SELECT embedding,model FROM library_chunks WHERE document_id=?1 AND hash=?2 AND text=?3 AND embedding IS NOT NULL LIMIT 1",params![doc.id,c.hash,c.text],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
            // Reprocessing a partial extraction must not invalidate citations to
            // unchanged evidence in the same revision.
            let existing_id: Option<String> = tx.query_row(
                "SELECT id FROM library_chunks WHERE document_id=?1 AND revision=?2 AND ordinal=?3 AND hash=?4 AND text=?5 AND location=?6",
                params![doc.id, doc.revision, c.ordinal, c.hash, c.text, c.location.to_string()],
                |r| r.get(0),
            ).optional()?;
            let chunk_id = existing_id.as_deref().unwrap_or(&c.id);
            tx.execute(
                "DELETE FROM library_chunks WHERE document_id=?1 AND revision=?2 AND ordinal=?3",
                params![doc.id, doc.revision, c.ordinal],
            )?;
            tx.execute(
                "INSERT INTO library_chunks VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![
                    chunk_id,
                    c.document_id,
                    c.revision,
                    c.ordinal,
                    c.text,
                    c.location.to_string(),
                    c.hash,
                    cached.as_ref().map(|x| &x.0),
                    cached.as_ref().map(|x| &x.1)
                ],
            )?;
            tx.execute(
                "INSERT INTO library_fts VALUES (?1,?2,?3)",
                params![chunk_id, doc.title, c.text],
            )?;
        }
        tx.execute(
            "DELETE FROM library_chunks WHERE document_id=?1 AND revision=?2 AND ordinal>=?3",
            params![doc.id, doc.revision, chunks.len()],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn library_chunks(
        &self,
        id: &str,
        revision: u64,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<LibraryChunk>> {
        let db = self.db()?;
        let mut q=db.prepare("SELECT id,document_id,revision,ordinal,text,location,hash FROM library_chunks WHERE document_id=?1 AND revision=?2 ORDER BY ordinal LIMIT ?3 OFFSET ?4")?;
        Ok(
            q.query_map(params![id, revision, limit.min(101), offset], chunk_row)?
                .collect::<std::result::Result<Vec<_>, _>>()?,
        )
    }
    pub fn library_chunk(&self, id: &str) -> Result<LibraryChunk> {
        self.db()?.query_row("SELECT id,document_id,revision,ordinal,text,location,hash FROM library_chunks WHERE id=?1",[id],chunk_row).optional()?.context("Source fragment not found")
    }
    pub fn library_pending_embeddings(
        &self,
        id: &str,
        revision: u64,
        model: &str,
    ) -> Result<Vec<LibraryChunk>> {
        let db = self.db()?;
        let mut q=db.prepare("SELECT id,document_id,revision,ordinal,text,location,hash FROM library_chunks WHERE document_id=?1 AND revision=?2 AND (embedding IS NULL OR model IS NOT ?3) ORDER BY ordinal LIMIT 16")?;
        Ok(q.query_map(params![id, revision, model], chunk_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?)
    }
    pub fn library_embeddings(
        &self,
        chunks: &[LibraryChunk],
        model: &str,
        vectors: &[Vec<f32>],
        tokens: u64,
    ) -> Result<()> {
        ensure!(chunks.len() == vectors.len(), "Embedding count mismatch");
        let mut db = self.db()?;
        let tx = db.transaction()?;
        for (c, v) in chunks.iter().zip(vectors) {
            ensure!(
                !v.is_empty() && v.iter().all(|v| v.is_finite()),
                "Invalid embedding"
            );
            tx.execute(
                "UPDATE library_chunks SET embedding=?3,model=?4 WHERE id=?1 AND hash=?2",
                params![c.id, c.hash, serde_json::to_string(v)?, model],
            )?;
        }
        // Keep the charge receipt atomic with the vectors; do not reacquire the DB mutex.
        let stamp = now();
        tx.execute(
            "INSERT INTO records(kind,id,data,updated_at) VALUES ('library_usage',?1,?2,?3)",
            params![
                id(),
                json!({"tokens":tokens,"model":model,"at":stamp,"operation":"index"}).to_string(),
                stamp
            ],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn library_search(
        &self,
        query: &str,
        project: Option<&str>,
        ids: &[String],
        collection: Option<&str>,
        vector: Option<(&str, &[f32])>,
        limit: usize,
    ) -> Result<Vec<LibraryHit>> {
        let db = self.db()?;
        let mut ranked = std::collections::BTreeMap::<String, (f64, Vec<String>)>::new();
        let ids = serde_json::to_string(ids)?;
        let scope = " FROM library_chunks c JOIN library_documents d ON d.id=c.document_id AND d.revision=c.revision JOIN library_revisions r ON r.document_id=c.document_id AND r.revision=c.revision WHERE d.project_id IS ?1 AND d.enabled=1 AND r.status IN ('ready','partial') AND (json_array_length(?2)=0 OR d.id IN (SELECT value FROM json_each(?2))) AND (?3 IS NULL OR json_extract(d.data,'$.collection')=?3) ";
        let terms = query
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .take(32)
            .map(|s| format!("\"{s}\""))
            .collect::<Vec<_>>()
            .join(" OR ");
        if !terms.is_empty() {
            let sql = format!(
                "SELECT c.id{scope} AND c.id IN (SELECT id FROM library_fts WHERE library_fts MATCH ?4) ORDER BY (SELECT bm25(library_fts,0.0,2.0,1.0) FROM library_fts WHERE id=c.id AND library_fts MATCH ?4) LIMIT 60"
            );
            let mut q = db.prepare(&sql)?;
            for (rank, row) in q
                .query_map(params![project, ids, collection, terms], |r| {
                    r.get::<_, String>(0)
                })?
                .enumerate()
            {
                ranked.insert(row?, (1.0 / (61 + rank) as f64, vec!["lexical".into()]));
            }
        }
        if let Some((model, query_vector)) = vector {
            let sql = format!(
                "SELECT c.id,c.embedding{scope} AND c.model=?4 AND c.embedding IS NOT NULL"
            );
            let mut q = db.prepare(&sql)?;
            let mut best: Vec<(String, f64)> = vec![];
            for row in q.query_map(params![project, ids, collection, model], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })? {
                let (id, raw) = row?;
                let v: Vec<f32> = serde_json::from_str(&raw)?;
                if v.len() != query_vector.len() {
                    continue;
                }
                let dot: f64 = v
                    .iter()
                    .zip(query_vector)
                    .map(|(a, b)| *a as f64 * *b as f64)
                    .sum();
                let norm = (v.iter().map(|a| (*a as f64).powi(2)).sum::<f64>()
                    * query_vector
                        .iter()
                        .map(|a| (*a as f64).powi(2))
                        .sum::<f64>())
                .sqrt();
                let score = dot / norm;
                if score.is_finite() && score >= 0.25 {
                    best.push((id, score));
                    best.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
                    best.truncate(60);
                }
            }
            for (rank, (id, _)) in best.into_iter().enumerate() {
                let e = ranked.entry(id).or_default();
                e.0 += 1.0 / (61 + rank) as f64;
                e.1.push("semantic".into());
            }
        }
        let mut ranked = ranked.into_iter().collect::<Vec<_>>();
        ranked.sort_by(|a, b| b.1.0.total_cmp(&a.1.0).then(a.0.cmp(&b.0)));
        let mut result = vec![];
        for (id, (score, branches)) in ranked.into_iter().take(limit.clamp(1, 30)) {
            let c=db.query_row("SELECT id,document_id,revision,ordinal,text,location,hash FROM library_chunks WHERE id=?1",[&id],chunk_row)?;
            let raw: String = db.query_row(
                "SELECT data FROM library_documents WHERE id=?1",
                [&c.document_id],
                |r| r.get(0),
            )?;
            let d: LibraryDocument = serde_json::from_str(&raw)?;
            result.push(LibraryHit {
                citation: format!(
                    "/documents?document={}&revision={}&chunk={}",
                    c.document_id, c.revision, c.id
                ),
                chunk: c,
                title: d.title,
                project_id: d.project_id,
                score,
                branches,
            });
        }
        Ok(result)
    }
}
