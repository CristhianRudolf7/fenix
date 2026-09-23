//! Non-destructive maintenance and revision-bound review proposals.
use super::*;

pub(crate) fn local_assessment(a: &Memory, b: &Memory) -> Value {
    let parse = |s: &Option<String>| {
        s.as_ref()
            .and_then(|v| chrono::DateTime::parse_from_rfc3339(v).ok())
    };
    let separated = parse(&a.metadata.valid_until)
        .zip(parse(&b.metadata.valid_from))
        .is_some_and(|(a, b)| a <= b)
        || parse(&b.metadata.valid_until)
            .zip(parse(&a.metadata.valid_from))
            .is_some_and(|(b, a)| b <= a);
    let classification = if memory::normalized(&a.content) == memory::normalized(&b.content) {
        "duplicate"
    } else if separated {
        "temporal_change"
    } else {
        "ambiguous"
    };
    json!({"classification":classification,"method":"deterministic","requires_model":classification=="ambiguous",
        "reason":if separated {"Explicit validity intervals do not overlap; this is not evidence of simultaneous contradiction."}
            else {"Text overlap is a review hint, not proof of agreement or contradiction."},
        "question":null,"mutates_facts":false})
}
impl Store {
    /// Inspect both original claims, not just an overlap score. Assessments expire on ANY revision.
    pub fn memory_review_details(&self, project: Option<&str>) -> Result<Vec<Value>> {
        self.memory_review_details_for(project, None)
    }
    pub(crate) fn memory_review_details_for(
        &self,
        project: Option<&str>,
        ids: Option<&[String]>,
    ) -> Result<Vec<Value>> {
        let db = self.db()?;
        let ids = ids.map(serde_json::to_string).transpose()?;
        let mut statement=db.prepare("SELECT p.first_id,p.second_id,p.reason,p.first_revision,p.second_revision,p.assessment,p.attempts FROM memory_review_pairs p JOIN memories a ON a.id=p.first_id JOIN memories b ON b.id=p.second_id WHERE a.project_id IS ?1 AND b.project_id IS ?1 AND (?2 IS NULL OR p.first_id IN (SELECT value FROM json_each(?2)) OR p.second_id IN (SELECT value FROM json_each(?2))) AND p.status='pending' AND coalesce(json_extract(a.metadata,'$.status'),'confirmed') IN ('confirmed','candidate') AND coalesce(json_extract(b.metadata,'$.status'),'confirmed') IN ('confirmed','candidate') AND p.first_revision=(SELECT max(seq) FROM memory_revisions WHERE memory_id=a.id) AND p.second_revision=(SELECT max(seq) FROM memory_revisions WHERE memory_id=b.id) ORDER BY CASE WHEN p.attempts<2 AND (p.assessment IS NULL OR json_extract(p.assessment,'$.requires_model')=1) THEN 0 ELSE 1 END,p.first_id,p.second_id LIMIT 100")?;
        let rows = statement
            .query_map(params![project, ids], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, i64>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, i64>(6)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let mut results = vec![];
        for (first, second, reason, first_revision, second_revision, assessment, attempts) in rows {
            let read = |id: &str| -> Result<Memory> {
                Ok(db.query_row("SELECT id,title,content,source,created_at,updated_at,project_id,task_id,metadata FROM memories WHERE id=?1 AND project_id IS ?2",params![id,project],memory::memory_row)?)
            };
            let a = read(&first)?;
            let b = read(&second)?;
            if !a.safe_for_context() || !b.safe_for_context() {
                continue;
            }
            results.push(json!({"first_id":first,"second_id":second,"reason":reason,"first_revision":first_revision,
                "second_revision":second_revision,"first":a,"second":b,"attempts":attempts,
                "assessment":assessment.map(|s|serde_json::from_str::<Value>(&s)).transpose()?.unwrap_or_else(||local_assessment(&a,&b))}));
        }
        Ok(results)
    }
    /// Reserve an attempt transactionally, across workers/restarts. Never run more than twice per revision pair.
    pub fn claim_memory_review(&self) -> Result<Option<Value>> {
        let scopes = {
            let db = self.db()?;
            db.prepare("SELECT DISTINCT a.project_id FROM memory_review_pairs p JOIN memories a ON a.id=p.first_id WHERE p.status='pending' AND p.attempts<2 AND (p.review_after IS NULL OR julianday(p.review_after)<=julianday('now')) AND (p.assessment IS NULL OR json_extract(p.assessment,'$.requires_model')=1) ORDER BY a.project_id")?
                .query_map([],|r|r.get::<_,Option<String>>(0))?.collect::<rusqlite::Result<Vec<_>>>()?
        };
        for scope in scopes {
            for pair in self.memory_review_details(scope.as_deref())? {
                if pair["assessment"]["requires_model"] != true
                    || pair["attempts"].as_i64().unwrap_or(2) >= 2
                {
                    continue;
                }
                let changed=self.db()?.execute("UPDATE memory_review_pairs SET attempts=attempts+1,review_after=strftime('%Y-%m-%dT%H:%M:%fZ','now','+60 seconds') WHERE first_id=?1 AND second_id=?2 AND first_revision=?3 AND second_revision=?4 AND attempts=?5 AND status='pending' AND (review_after IS NULL OR julianday(review_after)<=julianday('now'))",params![pair["first_id"].as_str(),pair["second_id"].as_str(),pair["first_revision"].as_i64(),pair["second_revision"].as_i64(),pair["attempts"].as_i64()])?;
                if changed == 1 {
                    return Ok(Some(pair));
                }
            }
        }
        Ok(None)
    }
    pub fn save_memory_review_assessment(&self, pair: &Value, assessment: &Value) -> Result<bool> {
        let classification = assessment["classification"]
            .as_str()
            .context("Missing review classification")?;
        if ![
            "duplicate",
            "complementary",
            "conflicting",
            "temporal_change",
            "ambiguous",
        ]
        .contains(&classification)
        {
            bail!("Invalid memory assessment");
        }
        let reason = assessment["reason"]
            .as_str()
            .context("Missing review explanation")?;
        let question = assessment["question"].as_str();
        if reason.is_empty()
            || reason.len() > 1200
            || question.is_some_and(|q| q.len() > 500)
            || classification == "ambiguous" && question.is_none_or(|q| q.trim().is_empty())
        {
            bail!("Invalid explanation or missing clarification question");
        }
        // Quotes must be exact substrings of BOTH authorized input claims. Labels alone are insufficient.
        for (key, side) in [("first_quote", "first"), ("second_quote", "second")] {
            let quote = assessment[key]
                .as_str()
                .context("Missing review evidence quote")?;
            let content = pair[side]["content"]
                .as_str()
                .context("Missing reviewed claim")?;
            if quote.trim().is_empty() || quote.len() > 4000 || !content.contains(quote) {
                bail!("Review evidence is not in the claim");
            }
        }
        if fenix_core::content_trust::instruction_risk(&assessment.to_string()) {
            bail!("Unsafe memory review output");
        }
        let mut stored = json!({"classification":classification,"reason":reason,"question":question,
            "first_quote":assessment["first_quote"],"second_quote":assessment["second_quote"]});
        stored["method"] = json!("model_proposal");
        stored["requires_model"] = json!(false);
        stored["mutates_facts"] = json!(false);
        let project = pair["first"]["project_id"].as_str();
        let changed=self.db()?.execute("UPDATE memory_review_pairs SET assessment=?5 WHERE first_id=?1 AND second_id=?2 AND first_revision=?3 AND second_revision=?4 AND status='pending' AND first_revision=(SELECT max(seq) FROM memory_revisions WHERE memory_id=?1) AND second_revision=(SELECT max(seq) FROM memory_revisions WHERE memory_id=?2) AND EXISTS(SELECT 1 FROM memories WHERE id=?1 AND project_id IS ?6) AND EXISTS(SELECT 1 FROM memories WHERE id=?2 AND project_id IS ?6) AND attempts=?7",params![pair["first_id"].as_str(),pair["second_id"].as_str(),pair["first_revision"].as_i64(),pair["second_revision"].as_i64(),stored.to_string(),project,pair["attempts"].as_i64().unwrap_or(0)+1])?;
        Ok(changed == 1)
    }
    /// Bounded repair of derived lexical entries only. Canonical memories are never rewritten.
    pub fn repair_memory_lexical_index(&self) -> Result<usize> {
        let cursor = self
            .get::<String>("memory_maintenance", "lexical_cursor")?
            .unwrap_or_default();
        let mut db = self.db()?;
        let tx = db.transaction()?;
        // Bound scanned rows as well as repairs, including the healthy-index case.
        let rows = tx.prepare("SELECT id,title,content,coalesce(json_extract(metadata,'$.status'),'confirmed') FROM memories WHERE id>?1 ORDER BY id LIMIT 50")?
            .query_map([&cursor], |r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let mut repaired = 0;
        for (id, title, content, status) in &rows {
            let indexed = tx
                .prepare("SELECT title,content FROM memory_fts WHERE id=?1 LIMIT 2")?
                .query_map([id], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            let correct = if status == "withdrawn" {
                indexed.is_empty()
            } else {
                indexed.len() == 1 && indexed[0].0 == *title && indexed[0].1 == *content
            };
            if !correct {
                tx.execute("DELETE FROM memory_fts WHERE id=?1", [id])?;
                if status != "withdrawn" {
                    tx.execute(
                        "INSERT INTO memory_fts(id,title,content) VALUES (?1,?2,?3)",
                        params![id, title, content],
                    )?;
                }
                repaired += 1;
            }
        }
        let next = rows.last().map(|row| row.0.as_str()).unwrap_or("");
        tx.execute("INSERT INTO records(kind,id,data,updated_at) VALUES ('memory_maintenance','lexical_cursor',?1,?2) ON CONFLICT(kind,id) DO UPDATE SET data=excluded.data,updated_at=excluded.updated_at",params![serde_json::to_string(next)?,now()])?;
        tx.commit()?;
        Ok(repaired)
    }
    /// Round-robin one scope per worker cycle. Persisted cursor avoids repeatedly scanning only global memory.
    pub fn memory_health_tick(&self, model: Option<&str>) -> Result<()> {
        let previous = self.get::<String>("memory_health", "cursor")?;
        let scopes = {
            let db = self.db()?;
            db.prepare("SELECT coalesce(project_id,'') AS scope FROM memories UNION SELECT scope FROM knowledge_entities ORDER BY scope")?
                .query_map([], |r|r.get::<_,String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?
        };
        let next = scopes
            .iter()
            .find(|scope| previous.as_ref().is_none_or(|p| *scope > p))
            .or(scopes.first());
        if let Some(scope) = next {
            let project = if scope.is_empty() {
                None
            } else {
                Some(scope.as_str())
            };
            let mut report = self.memory_diagnostics(project, model)?;
            report["checked_at"] = json!(now());
            self.put("memory_health_reports", scope, &report)?;
            self.put("memory_health", "cursor", scope)?;
        }
        Ok(())
    }
    /// Scoped, read-only health counts. An orphan is a review hint, never permission to delete it.
    pub fn memory_diagnostics(&self, project: Option<&str>, model: Option<&str>) -> Result<Value> {
        let db = self.db()?;
        let count = |sql: &str| -> Result<i64> { Ok(db.query_row(sql, [project], |r| r.get(0))?) };
        let total = count("SELECT count(*) FROM memories WHERE project_id IS ?1")?;
        let unindexed = count(
            "SELECT count(*) FROM memories m LEFT JOIN (SELECT id,count(*) AS n,max(title) AS title,max(content) AS content FROM memory_fts GROUP BY id) f ON f.id=m.id WHERE m.project_id IS ?1 AND coalesce(json_extract(m.metadata,'$.status'),'confirmed')!='withdrawn' AND (f.id IS NULL OR f.n!=1 OR f.title!=m.title OR f.content!=m.content)",
        )?;
        let orphans = count(
            "SELECT count(*) FROM knowledge_entities e WHERE e.scope=coalesce(?1,'') AND NOT EXISTS(SELECT 1 FROM memories m WHERE json_extract(m.metadata,'$.entity_id')=e.id AND m.project_id IS ?1) AND NOT EXISTS(SELECT 1 FROM knowledge_relations r WHERE r.scope=e.scope AND (r.source_id=e.id OR r.target_id=e.id))",
        )?;
        let ineffective = count(
            "SELECT count(*) FROM knowledge_relations r LEFT JOIN memories m ON m.id=json_extract(r.body,'$.evidence_memory_id') WHERE r.scope=coalesce(?1,'') AND json_extract(r.body,'$.status')='confirmed' AND (m.id IS NULL OR json_extract(r.body,'$.evidence_revision') IS NOT (SELECT max(seq) FROM memory_revisions WHERE memory_id=m.id) OR m.project_id IS NOT ?1 OR coalesce(json_extract(m.metadata,'$.status'),'confirmed')!='confirmed' OR julianday(json_extract(m.metadata,'$.valid_from'))>julianday('now') OR julianday(json_extract(m.metadata,'$.valid_until'))<=julianday('now') OR NOT EXISTS(SELECT 1 FROM knowledge_schemas s WHERE s.scope=r.scope AND s.name=r.predicate AND s.version=(SELECT max(version) FROM knowledge_schemas v WHERE v.scope=s.scope AND v.name=s.name) AND json_extract(s.body,'$.status')='active' AND s.version=json_extract(r.body,'$.schema_version')))",
        )?;
        let pending_vectors:Option<i64>=model.map(|model|db.query_row("SELECT count(*) FROM memories m LEFT JOIN memory_embeddings e ON e.memory_id=m.id WHERE m.project_id IS ?1 AND coalesce(json_extract(m.metadata,'$.status'),'confirmed')='confirmed' AND (json_extract(m.metadata,'$.valid_from') IS NULL OR julianday(json_extract(m.metadata,'$.valid_from'))<=julianday('now')) AND (json_extract(m.metadata,'$.valid_until') IS NULL OR julianday(json_extract(m.metadata,'$.valid_until'))>julianday('now')) AND (e.memory_id IS NULL OR e.model!=?2 OR e.content!=m.content)",params![project,model],|r|r.get(0))).transpose()?;
        let rows=db.prepare("SELECT id,title,content,source,created_at,updated_at,project_id,task_id,metadata FROM memories WHERE project_id IS ?1 AND coalesce(json_extract(metadata,'$.status'),'confirmed')='confirmed' ORDER BY id LIMIT 500")?.query_map([project],memory::memory_row)?.collect::<rusqlite::Result<Vec<_>>>()?;
        let sampled = rows.len();
        let stale = rows
            .iter()
            .filter(|m| {
                m.metadata.active() && memory_retrieval::freshness(m)["needs_revalidation"] == true
            })
            .map(|m| m.id.clone())
            .take(50)
            .collect::<Vec<_>>();
        let failed_reviews = count(
            "SELECT count(*) FROM memory_review_pairs p JOIN memories m ON m.id=p.first_id WHERE m.project_id IS ?1 AND p.status='pending' AND p.attempts>=2 AND (p.assessment IS NULL OR json_extract(p.assessment,'$.requires_model')=1)",
        )?;
        Ok(
            json!({"project_id":project,"total_memories":total,"lexical_missing_or_outdated":unindexed,
            "entities_without_facts_or_relations":orphans,"inactive_evidence_or_schema_edges":ineffective,
            "pending_embeddings":pending_vectors,"review_exhausted":failed_reviews,"revalidation_candidates":stale,
            "freshness_sample_size":sampled,"freshness_sample_limit":500,"candidate_ids_limit":50,
            "destructive_changes":false}),
        )
    }
}
