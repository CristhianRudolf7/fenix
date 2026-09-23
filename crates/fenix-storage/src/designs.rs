//! User-local visual artifacts with transactional optimistic revisions.
use crate::{Store, id, now};
use anyhow::{Context, Result, ensure};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Design {
    pub id: String,
    pub project_id: Option<String>,
    pub task_id: Option<String>,
    pub title: String,
    pub kind: String,
    pub revision: u64,
    pub content: Value,
    pub template_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DesignInput {
    pub title: String,
    pub kind: String,
    pub content: Value,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub template_id: Option<String>,
    #[serde(default)]
    pub expected_revision: Option<u64>,
}
impl Store {
    pub fn design(&self, id: &str, revision: Option<u64>) -> Result<Design> {
        let db = self.db()?;
        let data: Option<String> = if let Some(revision) = revision {
            db.query_row(
                "SELECT data FROM design_revisions WHERE design_id=?1 AND revision=?2",
                params![id, revision],
                |r| r.get(0),
            )
            .optional()?
        } else {
            db.query_row("SELECT data FROM designs WHERE id=?1", [id], |r| r.get(0))
                .optional()?
        };
        Ok(serde_json::from_str(&data.context("Design not found")?)?)
    }
    /// Metadata only: listing never loads file bytes or content into model context.
    pub fn designs(
        &self,
        project: Option<&str>,
        scoped: bool,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Value>> {
        let db = self.db()?;
        let mut stmt = db.prepare("SELECT json_remove(data,'$.content') FROM designs WHERE (?1=0 OR project_id IS ?2) ORDER BY updated_at DESC,id LIMIT ?3 OFFSET ?4")?;
        stmt.query_map(params![scoped, project, limit.clamp(1, 100), offset], |r| {
            r.get::<_, String>(0)
        })?
        .map(|r| Ok(serde_json::from_str(&r?)?))
        .collect()
    }
    pub fn save_design(
        &self,
        existing: Option<&str>,
        task: Option<&str>,
        input: DesignInput,
    ) -> Result<Design> {
        self.save_design_with(existing, task, input, |_| Ok(()))
    }

    /// Persist external revision artifacts before committing the database revision.
    /// A callback failure rolls the transaction back and preserves the previous design.
    pub fn save_design_with(
        &self,
        existing: Option<&str>,
        task: Option<&str>,
        input: DesignInput,
        persist: impl FnOnce(&Design) -> Result<()>,
    ) -> Result<Design> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let previous: Option<Design> = if let Some(id) = existing {
            let data: String = tx
                .query_row("SELECT data FROM designs WHERE id=?1", [id], |r| r.get(0))
                .optional()?
                .context("Design not found")?;
            Some(serde_json::from_str(&data)?)
        } else {
            None
        };
        if let Some(old) = &previous {
            ensure!(
                input.expected_revision == Some(old.revision),
                "Design changed; reload before editing"
            );
            ensure!(
                input.project_id == old.project_id,
                "Cannot move a design between project scopes"
            );
        }
        let design = Design {
            id: previous.as_ref().map_or_else(id, |p| p.id.clone()),
            project_id: input.project_id,
            task_id: task
                .map(str::to_owned)
                .or_else(|| previous.as_ref().and_then(|p| p.task_id.clone())),
            title: input.title,
            kind: input.kind,
            revision: previous.as_ref().map_or(1, |p| p.revision + 1),
            content: input.content,
            template_id: input
                .template_id
                .or_else(|| previous.as_ref().and_then(|p| p.template_id.clone())),
            created_at: previous.as_ref().map_or_else(now, |p| p.created_at.clone()),
            updated_at: now(),
        };
        let data = serde_json::to_string(&design)?;
        tx.execute("INSERT INTO designs VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9) ON CONFLICT(id) DO UPDATE SET task_id=excluded.task_id,title=excluded.title,kind=excluded.kind,revision=excluded.revision,data=excluded.data,updated_at=excluded.updated_at", params![design.id,design.project_id,design.task_id,design.title,design.kind,design.revision,data,design.created_at,design.updated_at])?;
        tx.execute(
            "INSERT INTO design_revisions VALUES(?1,?2,?3)",
            params![design.id, design.revision, data],
        )?;
        persist(&design)?;
        tx.commit()?;
        Ok(design)
    }
}
