//! Scoped, versioned knowledge. SQLite is canonical; documents remain evidence.
use super::*;
use std::collections::{BTreeSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityInput {
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub description: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub project_id: Option<String>,
    #[serde(flatten)]
    pub input: EntityInput,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationSchema {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub source_kinds: Vec<String>,
    #[serde(default)]
    pub target_kinds: Vec<String>,
    pub status: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationInput {
    pub source_id: String,
    pub target_id: String,
    pub predicate: String,
    pub evidence_memory_id: String,
    #[serde(default = "candidate")]
    pub status: String,
}
fn candidate() -> String {
    "candidate".into()
}
fn symbol(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}
fn scope(value: Option<&str>) -> &str {
    value.unwrap_or("")
}
pub(crate) fn entity_in_scope(db: &Connection, id: &str, project: Option<&str>) -> Result<Entity> {
    let body: String = db
        .query_row(
            "SELECT body FROM knowledge_entities WHERE id=?1 AND scope=?2",
            params![id, scope(project)],
            |r| r.get(0),
        )
        .optional()?
        .context("Entity not found in this scope")?;
    Ok(serde_json::from_str(&body)?)
}
fn validate_scope(db: &Connection, project: Option<&str>) -> Result<()> {
    if let Some(p) = project {
        projects::query_project(db, p)?;
    }
    Ok(())
}

impl Store {
    pub fn save_entity(
        &self,
        entity_id: Option<&str>,
        project: Option<&str>,
        input: EntityInput,
    ) -> Result<Entity> {
        if input.name.trim().is_empty()
            || input.name.len() > 200
            || !symbol(&input.kind)
            || input.description.len() > 4000
            || input.aliases.len() > 32
            || input
                .aliases
                .iter()
                .any(|a| a.trim().is_empty() || a.len() > 200)
        {
            bail!("Invalid entity name, type or aliases");
        }
        let mut db = self.db()?;
        let tx = db.transaction()?;
        validate_scope(&tx, project)?;
        if let Some(id) = entity_id {
            let previous = entity_in_scope(&tx, id, project)?;
            if previous.input.kind != input.kind {
                let linked:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM knowledge_relations WHERE source_id=?1 OR target_id=?1)",[id],|r|r.get(0))?;
                if linked {
                    bail!(
                        "An entity referenced by relations cannot change kind; create a new entity and revalidate the relations"
                    );
                }
            }
        }
        let entity = Entity {
            id: entity_id.map(str::to_owned).unwrap_or_else(id),
            project_id: project.map(str::to_owned),
            input,
        };
        tx.execute("INSERT INTO knowledge_entities VALUES (?1,?2,?3) ON CONFLICT(id) DO UPDATE SET body=excluded.body",params![entity.id,scope(project),serde_json::to_string(&entity)?])?;
        tx.execute(
            "DELETE FROM knowledge_aliases WHERE entity_id=?1",
            [&entity.id],
        )?;
        let aliases = std::iter::once(&entity.input.name)
            .chain(entity.input.aliases.iter())
            .map(|s| memory::normalized(s))
            .collect::<BTreeSet<_>>();
        for alias in aliases {
            tx.execute(
                "INSERT INTO knowledge_aliases VALUES (?1,?2,?3)",
                params![scope(project), alias, entity.id],
            )
            .context("An entity already owns this name or alias in this scope")?;
        }
        tx.commit()?;
        Ok(entity)
    }
    pub fn entities(&self, project: Option<&str>) -> Result<Vec<Entity>> {
        let db = self.db()?;
        let raw = db
            .prepare("SELECT body FROM knowledge_entities WHERE scope=?1 ORDER BY id LIMIT 500")?
            .query_map([scope(project)], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        raw.iter().map(|s| Ok(serde_json::from_str(s)?)).collect()
    }
    pub fn resolve_entity(&self, name: &str, project: Option<&str>) -> Result<Option<Entity>> {
        let db = self.db()?;
        let entity: Option<String> = db
            .query_row(
                "SELECT entity_id FROM knowledge_aliases WHERE scope=?1 AND alias=?2",
                params![scope(project), memory::normalized(name)],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(id) = entity {
            return Ok(Some(entity_in_scope(&db, &id, project)?));
        }
        let exists: bool = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM knowledge_entities WHERE id=?1 AND scope=?2)",
            params![name, scope(project)],
            |r| r.get(0),
        )?;
        if exists {
            Ok(Some(entity_in_scope(&db, name, project)?))
        } else {
            Ok(None)
        }
    }
    pub fn save_relation_schema(
        &self,
        project: Option<&str>,
        schema: RelationSchema,
    ) -> Result<Value> {
        if !symbol(&schema.name)
            || schema.description.is_empty()
            || schema.description.len() > 2000
            || !["candidate", "active", "disabled"].contains(&schema.status.as_str())
            || schema.source_kinds.len() > 32
            || schema.target_kinds.len() > 32
            || schema
                .source_kinds
                .iter()
                .chain(&schema.target_kinds)
                .any(|s| !symbol(s))
        {
            bail!("Invalid relation schema");
        }
        let mut db = self.db()?;
        let tx = db.transaction()?;
        validate_scope(&tx, project)?;
        let version: i64 = tx.query_row(
            "SELECT coalesce(max(version),0)+1 FROM knowledge_schemas WHERE scope=?1 AND name=?2",
            params![scope(project), schema.name],
            |r| r.get(0),
        )?;
        tx.execute(
            "INSERT INTO knowledge_schemas VALUES (?1,?2,?3,?4)",
            params![
                scope(project),
                schema.name,
                version,
                serde_json::to_string(&schema)?
            ],
        )?;
        tx.commit()?;
        Ok(json!({"version":version,"schema":schema}))
    }
    pub fn relation_schemas(&self, project: Option<&str>) -> Result<Vec<Value>> {
        let db = self.db()?;
        let raw=db.prepare("SELECT version,body FROM knowledge_schemas s WHERE scope=?1 AND version=(SELECT max(version) FROM knowledge_schemas v WHERE v.scope=s.scope AND v.name=s.name) ORDER BY name")?.query_map([scope(project)],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
        raw.into_iter()
            .map(|(v, s)| Ok(json!({"version":v,"schema":serde_json::from_str::<Value>(&s)?})))
            .collect()
    }
    pub fn save_relation(&self, project: Option<&str>, input: RelationInput) -> Result<Value> {
        if !["candidate", "confirmed", "withdrawn"].contains(&input.status.as_str()) {
            bail!("Invalid relation status");
        }
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let from = entity_in_scope(&tx, &input.source_id, project)?;
        let to = entity_in_scope(&tx, &input.target_id, project)?;
        let (version,raw):(i64,String)=tx.query_row("SELECT version,body FROM knowledge_schemas WHERE scope=?1 AND name=?2 ORDER BY version DESC LIMIT 1",params![scope(project),input.predicate],|r|Ok((r.get(0)?,r.get(1)?))).optional()?.context("Register a relation schema before using it")?;
        let schema: RelationSchema = serde_json::from_str(&raw)?;
        if schema.status == "disabled"
            || (input.status == "confirmed" && schema.status != "active")
            || (!schema.source_kinds.is_empty() && !schema.source_kinds.contains(&from.input.kind))
            || (!schema.target_kinds.is_empty() && !schema.target_kinds.contains(&to.input.kind))
        {
            bail!("Relation violates its current schema");
        }
        let raw: String = tx
            .query_row(
                "SELECT metadata FROM memories WHERE id=?1 AND project_id IS ?2",
                params![input.evidence_memory_id, project],
                |r| r.get(0),
            )
            .optional()?
            .context("Relation evidence must belong to the same scope")?;
        let evidence: MemoryMetadata = serde_json::from_str(&raw)?;
        if input.status == "confirmed" && (!evidence.active() || evidence.evidence.is_empty()) {
            bail!("Confirmed relations require active memory with source evidence");
        }
        let existing:Option<String>=tx.query_row("SELECT id FROM knowledge_relations WHERE scope=?1 AND source_id=?2 AND target_id=?3 AND predicate=?4 AND json_extract(body,'$.evidence_memory_id')=?5",params![scope(project),input.source_id,input.target_id,input.predicate,input.evidence_memory_id],|r|r.get(0)).optional()?;
        let id = existing.unwrap_or_else(id);
        let evidence_revision: i64 = tx.query_row(
            "SELECT max(seq) FROM memory_revisions WHERE memory_id=?1",
            [&input.evidence_memory_id],
            |r| r.get(0),
        )?;
        let body = json!({"id":id,"evidence_revision":evidence_revision,"schema_version":version,"source_id":input.source_id,"target_id":input.target_id,"predicate":input.predicate,"evidence_memory_id":input.evidence_memory_id,"status":input.status});
        tx.execute("INSERT INTO knowledge_relations VALUES (?1,?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET body=excluded.body",params![id,scope(project),input.source_id,input.target_id,input.predicate,body.to_string()])?;
        tx.commit()?;
        Ok(body)
    }
    /// Breadth-first paths, bounded independently by depth and result count. Every hop stays scoped.
    pub fn memory_neighbors(
        &self,
        entity: &str,
        project: Option<&str>,
        depth: usize,
        limit: usize,
    ) -> Result<Value> {
        let anchor = self
            .resolve_entity(entity, project)?
            .context("Entity not found")?;
        let db = self.db()?;
        let mut queue = VecDeque::from([(anchor.id.clone(), Vec::<String>::new())]);
        let mut visited = BTreeSet::from([anchor.id.clone()]);
        let mut visited_edges = BTreeSet::new();
        let mut rows = vec![];
        while let Some((node, path)) = queue.pop_front() {
            if path.len() >= depth.min(3) {
                continue;
            }
            let raw=db.prepare("SELECT r.body,m.metadata FROM knowledge_relations r JOIN memories m ON m.id=json_extract(r.body,'$.evidence_memory_id') WHERE r.scope=?1 AND (r.source_id=?2 OR r.target_id=?2) AND json_extract(r.body,'$.status')='confirmed' AND json_extract(r.body,'$.evidence_revision')=(SELECT max(seq) FROM memory_revisions WHERE memory_id=m.id) AND EXISTS(SELECT 1 FROM knowledge_schemas s WHERE s.scope=r.scope AND s.name=r.predicate AND s.version=(SELECT max(version) FROM knowledge_schemas v WHERE v.scope=s.scope AND v.name=s.name) AND json_extract(s.body,'$.status')='active' AND s.version=json_extract(r.body,'$.schema_version')) ORDER BY r.id LIMIT 100")?.query_map(params![scope(project),node],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
            for (raw, meta) in raw {
                if !serde_json::from_str::<MemoryMetadata>(&meta)?.active() {
                    continue;
                }
                let edge: Value = serde_json::from_str(&raw)?;
                let next = if edge["source_id"] == node {
                    edge["target_id"].as_str()
                } else {
                    edge["source_id"].as_str()
                }
                .context("Invalid graph endpoint")?;
                if visited_edges.insert(
                    edge["id"]
                        .as_str()
                        .context("Relation ID missing")?
                        .to_owned(),
                ) {
                    let mut route = path.clone();
                    route.push(edge["id"].as_str().unwrap_or_default().into());
                    rows.push(json!({"entity":entity_in_scope(&db,next,project)?,"edge":edge,"path":route}));
                    if visited.insert(next.to_owned()) {
                        queue.push_back((next.to_owned(), route));
                    }
                }
                if rows.len() >= limit.clamp(1, 100) {
                    return Ok(json!({"anchor":anchor,"neighbors":rows,"truncated":true}));
                }
            }
        }
        Ok(json!({"anchor":anchor,"neighbors":rows,"truncated":false}))
    }
    pub fn memory_card(&self, name: &str, project: Option<&str>) -> Result<Value> {
        let Some(entity) = self.resolve_entity(name, project)? else {
            return Ok(json!({"found":false}));
        };
        let facts = {
            let db = self.db()?;
            let rows=db.prepare("SELECT id,title,content,source,created_at,updated_at,project_id,task_id,metadata FROM memories WHERE project_id IS ?1 AND json_extract(metadata,'$.entity_id')=?2 AND coalesce(json_extract(metadata,'$.status'),'confirmed')='confirmed' AND (json_extract(metadata,'$.valid_from') IS NULL OR julianday(json_extract(metadata,'$.valid_from'))<=julianday('now')) AND (json_extract(metadata,'$.valid_until') IS NULL OR julianday(json_extract(metadata,'$.valid_until'))>julianday('now')) ORDER BY updated_at DESC LIMIT 30")?.query_map(params![project,entity.id],super::memory::memory_row)?.collect::<rusqlite::Result<Vec<_>>>()?;
            rows.into_iter()
                .filter(|m| m.metadata.active())
                .take(30)
                .collect::<Vec<_>>()
        };
        Ok(
            json!({"found":true,"entity":entity,"facts":facts,"relations":self.memory_neighbors(&entity.id,project,1,20)?}),
        )
    }
    /// Scope and lifecycle filters run before top-K, so inactive memories cannot crowd out evidence.
    pub fn memory_recall(
        &self,
        query: &str,
        project: Option<&str>,
        global: bool,
        limit: usize,
        since: Option<&str>,
        as_of: Option<&str>,
    ) -> Result<Vec<Memory>> {
        for date in [since, as_of].into_iter().flatten() {
            chrono::DateTime::parse_from_rfc3339(date)?;
        }
        if let Some(date) = as_of {
            return self.memory_recall_at(query, project, global, limit, since, date);
        }
        let db = self.db()?;
        let columns = "m.id,m.title,m.content,m.source,m.created_at,m.updated_at,m.project_id,m.task_id,m.metadata";
        let filter = "(m.project_id IS ?1 OR (?2 AND m.project_id IS NULL)) AND coalesce(json_extract(m.metadata,'$.status'),'confirmed')='confirmed' AND (json_extract(m.metadata,'$.valid_from') IS NULL OR julianday(json_extract(m.metadata,'$.valid_from'))<=julianday(?3)) AND (json_extract(m.metadata,'$.valid_until') IS NULL OR julianday(json_extract(m.metadata,'$.valid_until'))>julianday(?3)) AND (?4 IS NULL OR julianday(m.updated_at)>=julianday(?4))";
        let instant = as_of.map(str::to_owned).unwrap_or_else(now);
        let lexical = lexical_query(query);
        let raw = if query.is_empty() {
            db.prepare(&format!("SELECT {columns} FROM memories m WHERE {filter} ORDER BY coalesce(json_extract(m.metadata,'$.pinned'),0) DESC,CASE json_extract(m.metadata,'$.kind') WHEN 'constraint' THEN 0 WHEN 'preference' THEN 1 WHEN 'decision' THEN 2 ELSE 3 END,m.updated_at DESC,m.id LIMIT ?5"))?.query_map(params![project,global,instant,since,limit.min(200)],super::memory::memory_row)?.collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            db.prepare(&format!("SELECT {columns} FROM memories m WHERE {filter} AND (m.id=?6 OR m.id IN (SELECT id FROM memory_fts WHERE memory_fts MATCH ?7)) ORDER BY CASE WHEN m.id=?6 THEN 0 WHEN lower(m.title)=lower(?6) THEN 1 ELSE 2 END,coalesce((SELECT rank FROM memory_fts WHERE id=m.id AND memory_fts MATCH ?7),0),m.id LIMIT ?5"))?.query_map(params![project,global,instant,since,limit.min(200),query,lexical.unwrap_or_else(||"\"__no_match__\"".into())],super::memory::memory_row)?.collect::<rusqlite::Result<Vec<_>>>()?
        };
        Ok(raw)
    }
    fn memory_recall_at(
        &self,
        query: &str,
        project: Option<&str>,
        global: bool,
        limit: usize,
        since: Option<&str>,
        date: &str,
    ) -> Result<Vec<Memory>> {
        let instant = chrono::DateTime::parse_from_rfc3339(date)?;
        let db = self.db()?;
        let rows=db.prepare("SELECT r.memory_id,r.snapshot,r.changed_at,m.created_at,m.project_id,m.task_id FROM memory_revisions r JOIN memories m ON m.id=r.memory_id WHERE (r.project_id IS ?1 OR (?2 AND r.project_id IS NULL)) AND julianday(r.changed_at)<=julianday(?3) AND r.seq=(SELECT max(v.seq) FROM memory_revisions v WHERE v.memory_id=r.memory_id AND julianday(v.changed_at)<=julianday(?3)) ORDER BY r.seq DESC")?.query_map(params![project,global,date],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,Option<String>>(4)?,r.get::<_,Option<String>>(5)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
        let terms = query
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(str::to_lowercase)
            .collect::<Vec<_>>();
        let mut found = vec![];
        for (id, raw, updated_at, created_at, project_id, task_id) in rows {
            let snapshot: Value = serde_json::from_str(&raw)?;
            let metadata: MemoryMetadata = serde_json::from_value(snapshot["metadata"].clone())?;
            if metadata.status != "confirmed"
                || metadata.valid_from.as_ref().is_some_and(|d| {
                    chrono::DateTime::parse_from_rfc3339(d).is_ok_and(|d| d > instant)
                })
                || metadata.valid_until.as_ref().is_some_and(|d| {
                    chrono::DateTime::parse_from_rfc3339(d).is_ok_and(|d| d <= instant)
                })
            {
                continue;
            }
            if let Some(since) = since
                && chrono::DateTime::parse_from_rfc3339(&updated_at)?
                    < chrono::DateTime::parse_from_rfc3339(since)?
            {
                continue;
            }
            let text = format!("{} {}", snapshot["title"], snapshot["content"]).to_lowercase();
            if !query.is_empty() && id != query && !terms.iter().any(|t| text.contains(t)) {
                continue;
            }
            found.push(Memory {
                id,
                title: snapshot["title"].as_str().unwrap_or_default().into(),
                content: snapshot["content"].as_str().unwrap_or_default().into(),
                source: snapshot["source"].as_str().unwrap_or_default().into(),
                created_at,
                updated_at,
                project_id,
                task_id,
                metadata,
            });
            if found.len() >= limit.min(200) {
                break;
            }
        }
        Ok(found)
    }
    pub fn memory_context_pack(
        &self,
        query: &str,
        project: Option<&str>,
        budget: usize,
    ) -> Result<Value> {
        let budget = budget.clamp(256, 16_000);
        let mut items = vec![];
        let mut seen = BTreeSet::new();
        for m in self
            .memory_recall("", project, true, 40, None, None)?
            .into_iter()
            .filter(|m| {
                m.metadata.pinned
                    || ["constraint", "preference", "decision"].contains(&m.metadata.kind.as_str())
            })
        {
            if m.safe_for_context() && seen.insert(m.id.clone()) {
                items.push(json!({"type":"memory","value":m}));
            }
        }
        let priority_count = items.len();
        let mut retrieval = self.search_memory_evidence(
            query,
            project,
            &MemorySearchOptions {
                limit: 40,
                ..Default::default()
            },
            vec![],
        )?;
        // Preserve explicit general context without traversing relationships across projects.
        if project.is_some() {
            let general = self.search_memory_evidence(
                query,
                None,
                &MemorySearchOptions {
                    limit: 20,
                    graph: false,
                    ..Default::default()
                },
                vec![],
            )?;
            retrieval.results.extend(general.results);
            retrieval.reviews.extend(general.reviews);
        }
        for hit in retrieval.results {
            if hit.memory.safe_for_context() && seen.insert(hit.memory.id.clone()) {
                items.push(json!({"type":"memory","value":hit.memory,"retrieval":{"evidence":hit.evidence,"branches":hit.branches,"freshness":hit.freshness}}));
            }
        }
        for notice in retrieval.reviews.into_iter().take(3) {
            items.insert(
                priority_count,
                json!({"type":"memory_review","value":notice}),
            );
        }
        let (entities, _) = self.memory_query_entities(query, project)?;
        for entity in entities {
            items.push(json!({"type":"entity","value":self.memory_card(&entity.id,project)?}));
        }
        let mut included = vec![];
        let mut dropped = 0;
        for item in items {
            if fenix_core::content_trust::instruction_risk(&serde_json::to_string(&item)?) {
                dropped += 1;
                continue;
            }
            included.push(item);
            if serde_json::to_string(&included)?.len().div_ceil(3) > budget {
                included.pop();
                dropped += 1;
            }
        }
        let text = serde_json::to_string(&included)?;
        Ok(
            json!({"items":included,"text":text,"estimated_tokens":text.len().div_ceil(3),"budget_tokens":budget,"dropped_count":dropped,"retrieval_warnings":retrieval.warnings}),
        )
    }
    pub fn memory_history(&self, id: &str, project: Option<&str>) -> Result<Vec<Value>> {
        let db = self.db()?;
        let rows=db.prepare("SELECT seq,snapshot,changed_at FROM memory_revisions WHERE memory_id=?1 AND project_id IS ?2 ORDER BY seq DESC LIMIT 100")?.query_map(params![id,project],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
        rows.into_iter().map(|(seq,s,date)|Ok(json!({"seq":seq,"snapshot":serde_json::from_str::<Value>(&s)?,"changed_at":date}))).collect()
    }
    /// Stateless sequence cursor: acknowledge only after consuming the returned batch.
    pub fn memory_delta(&self, project: Option<&str>, after: i64, limit: usize) -> Result<Value> {
        let db = self.db()?;
        let limit = limit.clamp(1, 100);
        let rows=db.prepare("SELECT seq,record_id,body,changed_at,category FROM memory_changes WHERE project_id IS ?1 AND seq>?2 ORDER BY seq LIMIT ?3")?.query_map(params![project,after,limit+1],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
        let mut more = rows.len() > limit;
        let mut cursor = after;
        let mut items = vec![];
        let mut bytes = 0;
        for (seq, id, raw, date, category) in rows.into_iter().take(limit) {
            let snapshot: Value = serde_json::from_str(&raw)?;
            let snapshot = if raw.len() > 4000 {
                json!({"id":id,"status":snapshot["metadata"]["status"],"reference_only":true,"instruction":"Retrieve the record or its history by ID"})
            } else {
                snapshot
            };
            let item = json!({"seq":seq,"record_id":id,"category":category,"snapshot":snapshot,"changed_at":date});
            let size = item.to_string().len();
            if bytes + size > 16000 && !items.is_empty() {
                more = true;
                break;
            }
            bytes += size;
            cursor = seq;
            items.push(item);
        }
        Ok(json!({"items":items,"next_cursor":cursor,"has_more":more}))
    }
    pub fn enqueue_memory_job(&self, id: &str, task: &str, body: &Value) -> Result<()> {
        if body.to_string().len() > 200_000 {
            bail!("Memory extraction input exceeds 200KB");
        }
        self.db()?.execute(
            "INSERT OR IGNORE INTO memory_jobs(id,task_id,body,created_at) VALUES (?1,?2,?3,?4)",
            params![id, task, body.to_string(), now()],
        )?;
        Ok(())
    }
    pub fn claim_memory_job(&self) -> Result<Option<Value>> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let job=tx.query_row("SELECT id,task_id,body,attempts FROM memory_jobs WHERE status='queued' AND attempts<3 ORDER BY created_at,id LIMIT 1",[],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,i64>(3)?))).optional()?;
        let Some((id, task, raw, attempts)) = job else {
            return Ok(None);
        };
        tx.execute(
            "UPDATE memory_jobs SET status='running',attempts=attempts+1 WHERE id=?1",
            [&id],
        )?;
        tx.commit()?;
        Ok(Some(
            json!({"id":id,"task_id":task,"body":serde_json::from_str::<Value>(&raw)?,"attempts":attempts+1}),
        ))
    }
    pub fn finish_memory_job(&self, id: &str, error: Option<&str>) -> Result<()> {
        self.db()?.execute("UPDATE memory_jobs SET status=CASE WHEN ?2 IS NULL THEN 'done' WHEN attempts>=3 THEN 'failed' ELSE 'queued' END,error=?2 WHERE id=?1",params![id,error])?;
        Ok(())
    }
    pub fn memory_maintenance(&self) -> Result<Value> {
        let repaired = self.repair_memory_lexical_index()?;
        let db = self.db()?;
        let expired=db.execute("UPDATE memories SET metadata=json_set(metadata,'$.status','expired'),updated_at=?1 WHERE coalesce(json_extract(metadata,'$.status'),'confirmed') IN ('confirmed','candidate') AND julianday(json_extract(metadata,'$.valid_until'))<=julianday(?1)",[now()])?;
        let candidates: i64 = db.query_row(
            "SELECT count(*) FROM memories WHERE json_extract(metadata,'$.status')='candidate'",
            [],
            |r| r.get(0),
        )?;
        let failed: i64 = db.query_row(
            "SELECT count(*) FROM memory_jobs WHERE status='failed'",
            [],
            |r| r.get(0),
        )?;
        Ok(
            json!({"expired":expired,"candidates":candidates,"failed_jobs":failed,"lexical_entries_repaired":repaired}),
        )
    }
    /// Incremental overlap discovery is a review hint, never an automatic contradiction verdict.
    pub fn review_memory_overlaps(&self) -> Result<usize> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let cursor: i64 = tx.query_row(
            "SELECT seq FROM memory_maintenance_cursor WHERE id=1",
            [],
            |r| r.get(0),
        )?;
        let revisions = tx
            .prepare(
                "SELECT seq,memory_id FROM memory_revisions WHERE seq>?1 ORDER BY seq LIMIT 20",
            )?
            .query_map([cursor], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let mut found = 0;
        for (seq, id) in revisions {
            let current=tx.query_row("SELECT id,title,content,source,created_at,updated_at,project_id,task_id,metadata FROM memories WHERE id=?1",[&id],memory::memory_row)?;
            if matches!(current.metadata.status.as_str(), "confirmed" | "candidate") {
                let words = |s: &str| {
                    s.split(|c: char| !c.is_alphanumeric())
                        .filter(|s| s.len() > 2)
                        .map(str::to_lowercase)
                        .collect::<BTreeSet<_>>()
                };
                let a = words(&current.content);
                if a.len() >= 3 {
                    let candidates=tx.prepare("SELECT id,title,content,source,created_at,updated_at,project_id,task_id,metadata FROM memories WHERE project_id IS ?1 AND id!=?2 AND coalesce(json_extract(metadata,'$.kind'),'fact')=?3 AND json_extract(metadata,'$.entity_id') IS ?4 AND coalesce(json_extract(metadata,'$.status'),'confirmed') IN ('confirmed','candidate') AND id IN (SELECT id FROM memory_fts WHERE memory_fts MATCH ?5) LIMIT 100")?.query_map(params![current.project_id,current.id,current.metadata.kind,current.metadata.entity_id,lexical_query(&current.content).unwrap_or_default()],memory::memory_row)?.collect::<rusqlite::Result<Vec<_>>>()?;
                    for other in candidates {
                        let b = words(&other.content);
                        let union = a.union(&b).count();
                        if union > 0
                            && (a.intersection(&b).count() as f64 / union as f64 >= 0.35
                                || memory_retrieval::words(&current.title)
                                    == memory_retrieval::words(&other.title))
                        {
                            let (first, second) = if current.id < other.id {
                                (&current.id, &other.id)
                            } else {
                                (&other.id, &current.id)
                            };
                            let revision = |id: &str| -> Result<i64> {
                                Ok(tx.query_row(
                                    "SELECT max(seq) FROM memory_revisions WHERE memory_id=?1",
                                    [id],
                                    |r| r.get(0),
                                )?)
                            };
                            let assessment =
                                super::memory_health::local_assessment(&current, &other);
                            found+=tx.execute("INSERT INTO memory_review_pairs(first_id,second_id,reason,first_revision,second_revision,assessment) VALUES (?1,?2,'text_overlap: verify sources and validity intervals',?3,?4,?5) ON CONFLICT(first_id,second_id) DO UPDATE SET first_revision=excluded.first_revision,second_revision=excluded.second_revision,assessment=excluded.assessment,attempts=0,review_after=NULL,status='pending' WHERE memory_review_pairs.first_revision!=excluded.first_revision OR memory_review_pairs.second_revision!=excluded.second_revision",params![first,second,revision(first)?,revision(second)?,assessment.to_string()])?;
                        }
                    }
                }
            }
            tx.execute(
                "UPDATE memory_maintenance_cursor SET seq=?1 WHERE id=1",
                [seq],
            )?;
        }
        tx.commit()?;
        Ok(found)
    }
    pub fn memory_review_pairs(&self, project: Option<&str>) -> Result<Vec<Value>> {
        self.memory_review_details(project)
    }
    pub fn resolve_memory_pair(
        &self,
        project: Option<&str>,
        first: &str,
        second: &str,
        decision: &str,
    ) -> Result<()> {
        let (a,b) = self.db()?.query_row("SELECT first_revision,second_revision FROM memory_review_pairs WHERE first_id=?1 AND second_id=?2", params![first,second], |r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?)))?;
        self.resolve_memory_pair_at(project, first, second, decision, a, b)
    }
    pub fn resolve_memory_pair_at(
        &self,
        project: Option<&str>,
        first: &str,
        second: &str,
        decision: &str,
        first_revision: i64,
        second_revision: i64,
    ) -> Result<()> {
        if ![
            "complementary",
            "duplicate",
            "conflicting",
            "temporal_change",
            "ambiguous",
            "dismissed",
        ]
        .contains(&decision)
        {
            bail!("Invalid memory review decision");
        }
        let changed=self.db()?.execute("UPDATE memory_review_pairs SET status=?4 WHERE first_id=?2 AND second_id=?3 AND first_revision=?5 AND second_revision=?6 AND first_revision=(SELECT max(seq) FROM memory_revisions WHERE memory_id=?2) AND second_revision=(SELECT max(seq) FROM memory_revisions WHERE memory_id=?3) AND EXISTS(SELECT 1 FROM memories a WHERE a.id=?2 AND a.project_id IS ?1) AND EXISTS(SELECT 1 FROM memories b WHERE b.id=?3 AND b.project_id IS ?1)",params![project,first,second,decision,first_revision,second_revision])?;
        if changed == 0 {
            bail!("Memory review pair not found in scope or revisions changed; read reviews again");
        }
        Ok(())
    }
    pub fn recover_memory_jobs(&self) -> Result<()> {
        self.db()?.execute("UPDATE memory_jobs SET status=CASE WHEN attempts>=3 THEN 'failed' ELSE 'queued' END WHERE status='running'",[])?;
        Ok(())
    }
}
