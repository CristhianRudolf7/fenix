//! Aggregate recorded provider receipts; never infer billing from token counts.
use crate::Store;
use anyhow::{Result, ensure};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use std::collections::BTreeMap;

impl Store {
    pub fn usage_summary(&self, days: u32) -> Result<Value> {
        ensure!(matches!(days, 7 | 30 | 90), "Choose 7, 30 or 90 days");
        let today = (Utc::now() - Duration::hours(5)).date_naive();
        let start = today - Duration::days(i64::from(days) - 1);
        let mut daily = BTreeMap::new();
        for offset in 0..days {
            daily.insert(
                (start + Duration::days(i64::from(offset))).to_string(),
                [0u64; 5],
            );
        }
        let db = self.db()?;
        // Only authoritative usage receipts: tool results and copied summaries
        // can contain the same usage and must not be counted a second time.
        let mut query = db.prepare("WITH receipts AS (
            SELECT created_at AS at, json_extract(data,'$.usage') AS usage FROM events
            WHERE kind IN ('model_response','context_compacted','context_summary_response','skill_learning_usage','project_routing_response','decision_evaluation')
              AND COALESCE(json_extract(data,'$.cached'),0)=0
            UNION ALL SELECT created_at, json_extract(data,'$.result.usage') FROM events
            WHERE kind='memory_extraction' AND json_extract(data,'$.result.decision_provider') IS NULL
            UNION ALL SELECT updated_at, json_extract(data,'$.usage') FROM records WHERE kind='live_usage'
            UNION ALL SELECT updated_at, json_object('input_tokens',json_extract(data,'$.tokens'),'output_tokens',0,'total_tokens',json_extract(data,'$.tokens'))
            FROM records WHERE kind='library_usage'
        ) SELECT date(at,'-5 hours'),
          SUM(CASE WHEN json_type(usage,'$.input_tokens')='integer' THEN MAX(0,json_extract(usage,'$.input_tokens')) ELSE 0 END),
          SUM(CASE WHEN json_type(usage,'$.output_tokens')='integer' THEN MAX(0,json_extract(usage,'$.output_tokens')) ELSE 0 END),
          SUM(MAX(0,COALESCE(json_extract(usage,'$.total_tokens'),COALESCE(json_extract(usage,'$.input_tokens'),0)+COALESCE(json_extract(usage,'$.output_tokens'),0)))),
          COUNT(*), SUM(CASE WHEN json_type(usage,'$.input_tokens')='integer' AND json_type(usage,'$.output_tokens')='integer' THEN 0 ELSE 1 END)
        FROM receipts WHERE date(at,'-5 hours') BETWEEN ?1 AND ?2 GROUP BY date(at,'-5 hours')")?;
        let rows = query.query_map([start.to_string(), today.to_string()], |r| {
            Ok((
                r.get::<_, String>(0)?,
                [
                    r.get::<_, u64>(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                ],
            ))
        })?;
        let mut totals = [0u64; 5];
        for row in rows {
            let (date, values) = row?;
            for (sum, value) in totals.iter_mut().zip(values) {
                *sum = sum.saturating_add(value);
            }
            daily.insert(date, values);
        }
        let pack = |v: [u64; 5]| json!({"input_tokens":v[0],"output_tokens":v[1],"total_tokens":v[2],"requests":v[3],"missing_usage":v[4]});
        Ok(
            json!({"days":days,"timezone":"America/Lima","from":start.to_string(),"through":today.to_string(),"totals":pack(totals),
            "daily":daily.into_iter().map(|(date,values)|json!({"date":date,"usage":pack(values)})).collect::<Vec<_>>(),
            "coverage":"recorded_provider_usage","generated_at":crate::now()}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aggregates_receipts_without_recounting_results_or_cached_decisions() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = Store::open(&dir.path().join("store.sqlite"))?;
        let task = store.enqueue("test", "web", None)?;
        let usage = json!({"input_tokens":100,"output_tokens":20,"total_tokens":120});
        store.event(&task.id, "model_response", &json!({"usage":usage}))?;
        store.event(&task.id, "tool_result", &json!({"usage":usage}))?;
        store.event(
            &task.id,
            "decision_evaluation",
            &json!({"cached":true,"usage":usage}),
        )?;
        store.event(&task.id, "context_compacted", &json!({"usage":{}}))?;
        store.put(
            "live_usage",
            "voice",
            &json!({"usage":{"input_tokens":10,"output_tokens":5}}),
        )?;
        store.put("library_usage", "embedding", &json!({"tokens":7}))?;
        let report = store.usage_summary(7)?;
        assert_eq!(report["totals"]["total_tokens"], 142);
        assert_eq!(report["totals"]["requests"], 4);
        assert_eq!(report["totals"]["missing_usage"], 1);
        assert_eq!(report["daily"].as_array().unwrap().len(), 7);
        assert!(store.usage_summary(1000).is_err());
        let other = Store::open(&dir.path().join("other.sqlite"))?;
        assert_eq!(other.usage_summary(7)?["totals"]["total_tokens"], 0);
        Ok(())
    }
}
