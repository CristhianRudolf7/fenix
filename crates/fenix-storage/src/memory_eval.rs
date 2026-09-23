//! Offline retrieval evaluation: expected memory IDs, no model judge or network.
use super::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvalCase {
    pub name: String,
    pub query: String,
    pub project_id: Option<String>,
    pub relevant: Vec<String>,
    #[serde(default)]
    pub forbidden: Vec<String>,
}
/// Binary relevance, unique IDs, and a fixed k denominator (short lists cannot inflate precision).
pub fn retrieval_metrics(hits: &[String], relevant: &[String], k: usize) -> Value {
    let relevant = relevant.iter().collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let unique = hits
        .iter()
        .filter(|id| seen.insert(*id))
        .take(k)
        .collect::<Vec<_>>();
    let count = unique.iter().filter(|id| relevant.contains(**id)).count();
    let reciprocal = unique
        .iter()
        .position(|id| relevant.contains(*id))
        .map(|i| 1.0 / (i + 1) as f64)
        .unwrap_or(0.0);
    let dcg = unique
        .iter()
        .enumerate()
        .filter(|(_, id)| relevant.contains(**id))
        .map(|(i, _)| 1.0 / ((i + 2) as f64).log2())
        .sum::<f64>();
    let ideal = (0..k.min(relevant.len()))
        .map(|i| 1.0 / ((i + 2) as f64).log2())
        .sum::<f64>();
    json!({"precision_at_k":if k>0 {count as f64/k as f64}else{0.0},
        "recall_at_k":if relevant.is_empty(){if unique.is_empty(){1.0}else{0.0}}else{count as f64/relevant.len() as f64},
        "mrr":reciprocal,"ndcg_at_k":if ideal>0.0{dcg/ideal}else{0.0},
        "correct_abstention":relevant.is_empty() && unique.is_empty()})
}
impl Store {
    /// Compare lexical baseline to the production deterministic path on exactly the same corpus.
    pub fn evaluate_memory_retrieval(&self, cases: &[MemoryEvalCase], k: usize) -> Result<Value> {
        if cases.is_empty() || !(1..=100).contains(&k) {
            bail!("Evaluation requires cases and k=1..100");
        }
        let mut runs = vec![];
        for graph in [false, true] {
            let mut rows = vec![];
            for case in cases {
                let start = std::time::Instant::now();
                let (ids, context) = if graph {
                    let report = self.search_memory_evidence(
                        &case.query,
                        case.project_id.as_deref(),
                        &MemorySearchOptions {
                            limit: k,
                            ..Default::default()
                        },
                        vec![],
                    )?;
                    let ids = report
                        .results
                        .iter()
                        .map(|h| h.memory.id.clone())
                        .collect::<Vec<_>>();
                    (ids, serde_json::to_string(&report.results)?)
                } else {
                    let hits = self.memory_recall(
                        &case.query,
                        case.project_id.as_deref(),
                        false,
                        k,
                        None,
                        None,
                    )?;
                    let ids = hits.iter().map(|h| h.id.clone()).collect::<Vec<_>>();
                    (ids, serde_json::to_string(&hits)?)
                };
                let forbidden = ids
                    .iter()
                    .filter(|id| case.forbidden.contains(id))
                    .cloned()
                    .collect::<Vec<_>>();
                rows.push(json!({"name":case.name,"metrics":retrieval_metrics(&ids,&case.relevant,k),"hits":ids,
                    "forbidden_hits":forbidden,"latency_ms":start.elapsed().as_secs_f64()*1000.0,
                    "estimated_result_tokens":context.len().div_ceil(3)}));
            }
            let mean = |field: &str| {
                rows.iter()
                    .map(|r| r["metrics"][field].as_f64().unwrap_or(0.0))
                    .sum::<f64>()
                    / rows.len() as f64
            };
            let mut latency = rows
                .iter()
                .map(|r| r["latency_ms"].as_f64().unwrap_or(0.0))
                .collect::<Vec<_>>();
            latency.sort_by(f64::total_cmp);
            runs.push(json!({"configuration":if graph{"relational_evidence"}else{"lexical_baseline"},
                "mean_precision":mean("precision_at_k"),"mean_recall":mean("recall_at_k"),"mean_mrr":mean("mrr"),"mean_ndcg":mean("ndcg_at_k"),
                "p95_latency_ms":latency[((latency.len() as f64*0.95).ceil() as usize-1).min(latency.len()-1)],
                "mean_estimated_result_tokens":rows.iter().map(|r|r["estimated_result_tokens"].as_u64().unwrap_or(0)).sum::<u64>() as f64/rows.len() as f64,
                "forbidden_hits":rows.iter().map(|r|r["forbidden_hits"].as_array().map_or(0,Vec::len)).sum::<usize>(),"cases":rows}));
        }
        Ok(
            json!({"k":k,"query_count":cases.len(),"runs":runs,"model_calls":0,"embedding_calls":0,
            "token_measurement":"UTF-8 bytes / 3 estimate of serialized retrieval results, not billed model tokens",
            "scope":"offline development retrieval evaluation, not evidence of end-to-end user task accuracy"}),
        )
    }
}
