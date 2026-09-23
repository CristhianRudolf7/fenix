//! Deterministic provider fixture for existing planner tests that now audit results.
use fenix_storage::Store;
use serde_json::{Value, json};

pub fn response(body: &Value, store: &Store) -> Option<Value> {
    if !body["tools"]
        .as_array()?
        .iter()
        .any(|t| t["name"] == "completion_audit_read")
    {
        return None;
    }
    let system = body["input"]
        .as_array()?
        .iter()
        .find(|m| m["role"] == "system")?
        .to_string();
    let plan = store
        .plans()
        .ok()?
        .into_iter()
        .find(|p| system.contains(&p.id) || body["input"].to_string().contains(&p.id))?;
    let audit = store
        .completion_audits(&plan.id)
        .ok()?
        .into_iter()
        .find(|a| a.worker_id.as_ref().is_some_and(|w| system.contains(w)))?;
    let read = body["input"]
        .as_array()?
        .iter()
        .any(|m| m["type"] == "function_call_output" && m["call_id"] == "audit-read");
    let (tool, args) = if !read {
        ("file_read", json!({"path":"audit-fixture.txt"}))
    } else {
        let checks=store.plan_evidence(&plan.id,plan.version).ok()?.into_iter().filter(|e|audit.node_id.is_empty() || e.input.node_id.as_deref()==Some(audit.node_id.as_str())).map(|e|json!({"entry_id":e.input.id,"state":"verified","detail":"Compared fixture deliverable","call_ids":["audit-read"]})).collect::<Vec<_>>();
        (
            "subagent_finish",
            json!({"status":"completed","summary":"Fixture audited","completion_audit":{"verdict":"verified","summary":"Independent fixture observation","checks":checks}}),
        )
    };
    Some(
        json!({"id":"audit-fixture","status":"completed","output":[{"type":"function_call","call_id":if read {"audit-finish"} else {"audit-read"},"name":tool,"arguments":args.to_string()}],"usage":{"input_tokens":12,"output_tokens":8,"total_tokens":20}}),
    )
}
