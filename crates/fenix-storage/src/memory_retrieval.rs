//! Evidence-bearing retrieval. Ranking is advisory, never a truth probability.
use super::*;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MemorySearchOptions {
    pub limit: usize,
    pub graph: bool,
    pub depth: usize,
    /// auto, on or off. Recency only nudges relevant candidates.
    pub recency: String,
    pub since: Option<String>,
    pub as_of: Option<String>,
}
impl Default for MemorySearchOptions {
    fn default() -> Self {
        Self {
            limit: 10,
            graph: true,
            depth: 2,
            recency: "auto".into(),
            since: None,
            as_of: None,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchHit {
    pub memory: Memory,
    pub score: f64,
    pub branches: Vec<String>,
    pub evidence: Vec<Value>,
    pub create_safety: String,
    pub freshness: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchReport {
    pub results: Vec<MemorySearchHit>,
    pub diagnostics: Value,
    pub warnings: Vec<String>,
    pub reviews: Vec<Value>,
}

pub(crate) fn words(text: &str) -> String {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}
fn recent_query(query: &str) -> bool {
    let q = format!(" {} ", words(query));
    [
        "now",
        "current",
        "currently",
        "latest",
        "recent",
        "today",
        "ahora",
        "actual",
        "actualmente",
        "reciente",
        "último",
        "hoy",
    ]
    .iter()
    .any(|w| q.contains(&format!(" {w} ")))
}
pub(crate) fn freshness(memory: &Memory) -> Value {
    let days = match memory.metadata.kind.as_str() {
        "commitment" => 14.0,
        "assumption" => 30.0,
        "fact" => 90.0,
        _ => 0.0, // Stable preferences, constraints and decisions do not decay.
    };
    let date = memory
        .metadata
        .valid_from
        .as_deref()
        .unwrap_or(&memory.created_at);
    let age = chrono::DateTime::parse_from_rfc3339(date).ok().map(|d| {
        (chrono::Utc::now() - d.with_timezone(&chrono::Utc))
            .num_days()
            .max(0) as f64
    });
    json!({"age_days":age,"reference_date":date,"half_life_days":days,
        "needs_revalidation":days>0.0 && age.is_some_and(|a|a>days*2.0),
        "meaning":"age is a review hint, not evidence that the claim is false"})
}
impl Store {
    /// Explicit aliases only; bounded matching does not invent an entity from a paraphrase.
    pub(crate) fn memory_query_entities(
        &self,
        query: &str,
        project: Option<&str>,
    ) -> Result<(Vec<Entity>, bool)> {
        let query = format!(" {} ", words(query));
        let db = self.db()?;
        let mut entities = Vec::new();
        let mut statement =
            db.prepare("SELECT body FROM knowledge_entities WHERE scope=?1 ORDER BY id")?;
        let rows = statement.query_map([project.unwrap_or("")], |r| r.get::<_, String>(0))?;
        for row in rows {
            let entity: Entity = serde_json::from_str(&row?)?;
            if std::iter::once(&entity.input.name)
                .chain(entity.input.aliases.iter())
                .any(|a| query.contains(&format!(" {} ", words(a))))
            {
                if entities.len() == 4 {
                    return Ok((entities, true));
                }
                entities.push(entity);
            }
        }
        Ok((entities, false))
    }
    pub fn search_memory_evidence(
        &self,
        query: &str,
        project: Option<&str>,
        opts: &MemorySearchOptions,
        semantic: Vec<(Memory, f64)>,
    ) -> Result<MemorySearchReport> {
        if query.len() > 8000 || !["auto", "on", "off"].contains(&opts.recency.as_str()) {
            bail!("Invalid memory query or recency mode");
        }
        let limit = opts.limit.clamp(1, 100);
        let pool = (limit * 4).clamp(40, 200);
        let historical = opts.as_of.is_some() || opts.since.is_some();
        if historical && !semantic.is_empty() {
            bail!("Temporal recall cannot use current vectors");
        }
        let lexical = self.memory_recall(
            query,
            project,
            false,
            pool,
            opts.since.as_deref(),
            opts.as_of.as_deref(),
        )?;
        let lexical_count = lexical.len();
        let mut hits = BTreeMap::<String, MemorySearchHit>::new();
        let mut add = |m: Memory, rank: usize, branch: &str, evidence: Value| {
            if m.project_id.as_deref() != project || !m.safe_for_context() {
                return;
            }
            let entry = hits.entry(m.id.clone()).or_insert_with(|| MemorySearchHit {
                freshness: freshness(&m),
                memory: m,
                score: 0.0,
                branches: vec![],
                evidence: vec![],
                create_safety: "unknown".into(),
            });
            if !entry.branches.iter().any(|b| b == branch) {
                entry.score += 1.0 / (61 + rank) as f64;
                entry.branches.push(branch.into());
            }
            if entry.evidence.len() < 8 {
                entry.evidence.push(evidence);
            }
        };
        for (rank, m) in lexical.into_iter().enumerate() {
            let signal = if m.id == query {
                "exact_id"
            } else if memory::normalized(&m.content) == memory::normalized(query) {
                "exact_content"
            } else if words(&m.title) == words(query) {
                "exact_title"
            } else {
                "keyword"
            };
            add(m, rank, "lexical", json!({"signal":signal}));
        }
        let semantic_count = semantic.len();
        for (rank, (m, cosine)) in semantic.into_iter().enumerate() {
            if m.metadata.active() && cosine.is_finite() {
                add(
                    m,
                    rank,
                    "semantic",
                    json!({"signal":"vector","cosine":cosine}),
                );
            }
        }
        let mut warnings = vec![];
        let mut graph_count = 0;
        let mut anchors_count = 0;
        if opts.graph && !historical && !query.trim().is_empty() {
            let (anchors, truncated) = self.memory_query_entities(query, project)?;
            anchors_count = anchors.len();
            if truncated {
                warnings.push("anchor_limit_reached".into());
            }
            for anchor in anchors {
                // A failed arm must not conceal successful lexical retrieval.
                let neighbors =
                    match self.memory_neighbors(&anchor.id, project, opts.depth.clamp(1, 3), 40) {
                        Ok(v) => v,
                        Err(_) => {
                            warnings.push("graph_unavailable".into());
                            continue;
                        }
                    };
                if neighbors["truncated"] == true {
                    warnings.push("graph_limit_reached".into());
                }
                for row in neighbors["neighbors"].as_array().into_iter().flatten() {
                    let Some(id) = row["edge"]["evidence_memory_id"].as_str() else {
                        continue;
                    };
                    for m in self
                        .memory_recall(id, project, false, 1, None, None)?
                        .into_iter()
                        .filter(|m| m.id == id)
                    {
                        add(
                            m,
                            graph_count,
                            "graph",
                            json!({"signal":"relation_path","anchor_id":anchor.id,
                            "entity":row["entity"],"edge":row["edge"],"path":row["path"]}),
                        );
                        graph_count += 1;
                    }
                }
            }
        } else if historical && opts.graph {
            warnings.push("historical_graph_disabled".into());
        }
        let recency =
            !historical && (opts.recency == "on" || opts.recency == "auto" && recent_query(query));
        let mut results = hits.into_values().collect::<Vec<_>>();
        for hit in &mut results {
            if hit
                .evidence
                .iter()
                .any(|e| matches!(e["signal"].as_str(), Some("exact_id" | "exact_content")))
            {
                hit.create_safety = "exists".into();
            } else if hit.evidence.iter().any(|e| e["signal"] == "exact_title") {
                hit.create_safety = "possible_match".into();
            }
            if recency {
                let half = hit.freshness["half_life_days"].as_f64().unwrap_or(0.0);
                let age = hit.freshness["age_days"].as_f64().unwrap_or(0.0);
                let boost = if half > 0.0 {
                    1.0 + 0.15 * half / (half + age)
                } else {
                    1.0
                };
                hit.score *= boost;
                hit.freshness["ranking_multiplier"] = json!(boost);
            }
        }
        // Exact identity is stronger than any blended score; scores are not probabilities.
        results.sort_by(|a, b| {
            (b.create_safety == "exists")
                .cmp(&(a.create_safety == "exists"))
                .then(b.score.total_cmp(&a.score))
                .then(a.memory.id.cmp(&b.memory.id))
        });
        // Preserve one graph answer when fusion would otherwise hide the entire relational arm.
        if limit > 1
            && results
                .iter()
                .take(limit)
                .all(|h| !h.branches.iter().any(|b| b == "graph"))
            && let Some(pos) = results
                .iter()
                .position(|h| h.branches.iter().any(|b| b == "graph"))
            && pos >= limit
        {
            let hit = results.remove(pos);
            results.insert(limit - 1, hit);
        }
        let candidates = results.len();
        results.truncate(limit);
        let ids = results
            .iter()
            .map(|h| h.memory.id.clone())
            .collect::<Vec<_>>();
        let reviews = if historical || ids.is_empty() {
            vec![]
        } else {
            self.memory_review_details_for(project, Some(&ids))?.into_iter().take(5).map(|p|json!({
                "first_id":p["first_id"],"second_id":p["second_id"],"first_revision":p["first_revision"],"second_revision":p["second_revision"],
                "classification":p["assessment"]["classification"],"method":p["assessment"]["method"],
                "reason":p["assessment"]["reason"],"question":p["assessment"]["question"],
                "details":"memory_context reviews; a proposal never overrides original evidence"
            })).collect()
        };
        warnings.sort();
        warnings.dedup();
        Ok(MemorySearchReport {
            results,
            warnings,
            reviews,
            diagnostics: json!({"lexical_candidates":lexical_count,
            "semantic_candidates":semantic_count,"graph_candidates":graph_count,"anchors":anchors_count,
            "candidate_limit":pool,"candidate_limit_reached":lexical_count==pool,"unique_candidates":candidates,
            "recency_applied":recency,"historical":historical,
            "score_meaning":"reciprocal rank fusion, not probability or truth confidence"}),
        })
    }
}
