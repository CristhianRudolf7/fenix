use super::*;
impl Store {
    pub fn pending_memory_embeddings(&self, model: &str, limit: usize) -> Result<Vec<Memory>> {
        let db = self.db()?;
        let rows=db.prepare("SELECT m.id,m.title,m.content,m.source,m.created_at,m.updated_at,m.project_id,m.task_id,m.metadata FROM memories m LEFT JOIN memory_embeddings e ON e.memory_id=m.id WHERE (e.memory_id IS NULL OR e.model!=?1 OR e.content!=m.content) AND coalesce(json_extract(m.metadata,'$.status'),'confirmed')='confirmed' AND (json_extract(m.metadata,'$.valid_from') IS NULL OR julianday(json_extract(m.metadata,'$.valid_from'))<=julianday('now')) AND (json_extract(m.metadata,'$.valid_until') IS NULL OR julianday(json_extract(m.metadata,'$.valid_until'))>julianday('now')) ORDER BY m.updated_at,m.id LIMIT ?2")?.query_map(params![model,limit.min(32)],memory::memory_row)?.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows
            .into_iter()
            .filter(|m| m.metadata.active() && m.safe_for_context())
            .collect())
    }
    pub fn save_memory_embedding(
        &self,
        id: &str,
        model: &str,
        content: &str,
        vector: &[f64],
    ) -> Result<()> {
        validate_vector(vector)?;
        self.db()?.execute("INSERT INTO memory_embeddings SELECT id,?2,content,?4 FROM memories WHERE id=?1 AND content=?3 AND coalesce(json_extract(metadata,'$.status'),'confirmed')='confirmed' ON CONFLICT(memory_id) DO UPDATE SET model=excluded.model,content=excluded.content,vector=excluded.vector",params![id,model,content,serde_json::to_string(vector)?])?;
        Ok(())
    }
    pub fn semantic_memories(
        &self,
        project: Option<&str>,
        model: &str,
        query: &[f64],
        limit: usize,
    ) -> Result<Vec<(Memory, f64)>> {
        validate_vector(query)?;
        let db = self.db()?;
        let mut statement=db.prepare("SELECT m.id,m.title,m.content,m.source,m.created_at,m.updated_at,m.project_id,m.task_id,m.metadata,e.vector FROM memories m JOIN memory_embeddings e ON e.memory_id=m.id WHERE m.project_id IS ?1 AND e.model=?2 AND e.content=m.content AND coalesce(json_extract(m.metadata,'$.status'),'confirmed')='confirmed'")?;
        let rows = statement.query_map(params![project, model], |r| {
            Ok((memory::memory_row(r)?, r.get::<_, String>(9)?))
        })?;
        let mut scored = vec![];
        let qnorm = query.iter().map(|x| x * x).sum::<f64>().sqrt();
        for row in rows {
            let (m, raw) = row?;
            if !m.metadata.active() || !m.safe_for_context() {
                continue;
            }
            let vector: Vec<f64> = serde_json::from_str(&raw)?;
            if vector.len() != query.len() {
                continue;
            }
            let denominator = qnorm * vector.iter().map(|x| x * x).sum::<f64>().sqrt();
            let score = vector.iter().zip(query).map(|(a, b)| a * b).sum::<f64>() / denominator;
            if score.is_finite() && score > 0.0 {
                scored.push((m, score));
                scored.sort_by(|(a, x), (b, y)| y.total_cmp(x).then(a.id.cmp(&b.id)));
                scored.truncate(limit.min(100));
            }
        }
        scored.sort_by(|(a, x), (b, y)| y.total_cmp(x).then(a.id.cmp(&b.id)));
        scored.truncate(limit.min(100));
        Ok(scored)
    }
}
fn validate_vector(v: &[f64]) -> Result<()> {
    if !(2..=8192).contains(&v.len())
        || v.iter().any(|n| !n.is_finite())
        || !v.iter().map(|n| n * n).sum::<f64>().is_finite()
        || v.iter().map(|n| n * n).sum::<f64>() <= 0.0
    {
        bail!("Embedding must have 2–8192 finite dimensions and a positive norm");
    }
    Ok(())
}
