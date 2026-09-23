//! Compact model-facing receipts. The complete redacted result stays in the journal.
use serde_json::{Value, json};

pub(crate) fn bounded(output: Value, budget: usize, task: &str, call: &str) -> Value {
    let serialized = output.to_string();
    if serialized.len() <= budget {
        return output;
    }
    let mut receipt = json!({
        "truncated": true,
        "preview": head_tail(&serialized, budget),
        "task_id": task,
        "call_id": call,
        "retrieve": "Use task_inspect with task_id, call_id, offset and length to read the saved output in pages, or query to search its contents. Omit query when reading a search hit's character offsets. The preview omits the middle; it is not complete evidence."
    });
    // Keep execution state usable even when it falls in the omitted middle.
    // Repository guidance must remain intact before an agent retries a mutation.
    if let Some(rules) = output.get("repository_instructions") {
        receipt["repository_instructions"] = rules.clone();
    }
    // Search batches can exceed the per-call context allowance. Preserve the
    // distinction between no evidence and unavailable sources even when the
    // middle (including hits) is omitted. Full passages remain in task_inspect.
    if output["backend"] == "local_fts5" {
        let sources: Vec<_> = output["source_reads"]
            .as_array()
            .into_iter()
            .flatten()
            .chain(
                output["discovery"]["sources"]
                    .as_array()
                    .into_iter()
                    .flatten(),
            )
            .collect();
        let results = output["results"].as_array();
        receipt["backend"] = output["backend"].clone();
        receipt["query"] = json!(crate::redaction::truncate(
            output["query"].as_str().unwrap_or_default(),
            400
        ));
        receipt["discovery"] = json!({"enabled": output["discovery"]["enabled"]});
        receipt["search_summary"] = json!({
            "returned_results": results.map_or(0, Vec::len),
            "unread_results": results.into_iter().flatten().filter(|hit| hit["page"].is_null()).count(),
            "sources_attempted": sources.len(),
            "sources_failed": sources.iter().filter(|source| source["status"] == "failed").count(),
            "sources_blocked": sources.iter().filter(|source| source["failure_kind"] == "blocked").count(),
            "unvisited_domains": output["discovery"]["unvisited_domains"].as_array().map_or(0, Vec::len),
            "full_evidence": "Read the saved output with task_inspect before citing omitted passages or claiming source coverage."
        });
        receipt["preview"] = json!(head_tail(&serialized, budget.saturating_sub(800)));
    }
    // Preserve actionable download evidence even when diagnostic events occupy
    // the middle of an oversized browser inspection. Bound it independently.
    if output["downloads"].is_object() {
        let mut downloads = json!({});
        for key in ["files", "transfers"] {
            let items: Vec<_> = output["downloads"][key]
                .as_array()
                .into_iter()
                .flatten()
                .take(8)
                .cloned()
                .collect();
            downloads[key] = json!(items);
        }
        downloads["complete_list_in_saved_output"] = json!(true);
        let size = downloads.to_string().len();
        if size <= 4096 {
            receipt["downloads"] = downloads;
            receipt["profile"] = output["profile"].clone();
            receipt["tab"] = output["tab"].clone();
            receipt["preview"] = json!(head_tail(&serialized, budget.saturating_sub(size)));
        }
    }
    for key in [
        "executed",
        "denied",
        "status",
        "exit_code",
        "process_id",
        "execution_mode",
        "failure",
        "initial_wait_ms",
        "next_cursor",
        "log_complete",
        "all_invocations_reported_success",
        "coverage_gap_count",
        "next_offset",
        "index_complete",
        "indexed_through_event",
        "total",
        "output_error",
        "code",
        "error",
    ] {
        if let Some(value) = output.get(key) {
            receipt[key] = if value.to_string().len() <= 512 {
                value.clone()
            } else {
                json!({"truncated": true, "preview": head_tail(&value.to_string(), 512)})
            };
        }
    }
    receipt
}

fn head_tail(text: &str, budget: usize) -> String {
    if text.len() <= budget {
        return text.into();
    }
    const GAP: &str = "\n[... middle omitted ...]\n";
    if budget < GAP.len() {
        return GAP[..budget].into();
    }
    let available = budget - GAP.len();
    let mut head = available / 3;
    while !text.is_char_boundary(head) {
        head -= 1;
    }
    let mut tail = text.len() - (available - head);
    while !text.is_char_boundary(tail) {
        tail += 1;
    }
    format!("{}{GAP}{}", &text[..head], &text[tail..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_output_search_receipt_keeps_partial_coverage() {
        let receipt = bounded(
            json!({"results":"evidence".repeat(3000),"index_complete":false,"indexed_through_event":64,"next_offset":3}),
            1000,
            "task",
            "search",
        );
        assert_eq!(receipt["index_complete"], false);
        assert_eq!(receipt["indexed_through_event"], 64);
        assert_eq!(receipt["next_offset"], 3);
    }

    #[test]
    fn security_receipts_preserve_scan_gaps_and_pagination() {
        let result = bounded(
            json!({"candidates":"x".repeat(20000),"all_invocations_reported_success":false,"coverage_gap_count":2,"next_offset":10,"total":27}),
            1000,
            "worker",
            "scan",
        );
        assert_eq!(result["all_invocations_reported_success"], false);
        assert_eq!(result["coverage_gap_count"], 2);
        assert_eq!(result["next_offset"], 10);
        assert_eq!(result["total"], 27);
        assert_eq!(result["task_id"], "worker");
    }

    #[test]
    fn browser_receipts_keep_completed_links_and_transfer_progress() {
        let downloads = json!({"files":[{"id":"file","bytes":123,"url":"/api/browser/downloads/p/file"}],"transfers":[{"id":"pending","state":"inProgress","received_bytes":456}]});
        let output =
            json!({"profile":"p","tab":"t","downloads":downloads,"events":"noise".repeat(5000)});
        let receipt = bounded(output, 2000, "task", "call");
        assert_eq!(receipt["downloads"]["files"], downloads["files"]);
        assert_eq!(receipt["downloads"]["transfers"], downloads["transfers"]);
        assert_eq!(receipt["profile"], "p");
        assert!(receipt["truncated"].as_bool().unwrap());
    }

    #[test]
    fn retains_final_diagnostics_and_execution_state() {
        let output = json!({
            "aaa": "x".repeat(15_000),
            "executed": true, "exit_code": 2, "status": "completed",
            "process_id": "process-123", "next_cursor": 30000,
            "zzz": format!("{}\nFAILED: undefined reference to parser", "y".repeat(15_000))
        });
        let result = bounded(output, 9000, "task-123", "call-456");
        assert_eq!(result["exit_code"], 2);
        assert_eq!(result["executed"], true);
        assert_eq!(result["process_id"], "process-123");
        assert_eq!(result["next_cursor"], 30000);
        let preview = result["preview"].as_str().unwrap();
        assert!(preview.contains("FAILED: undefined reference to parser"));
        assert!(preview.contains("middle omitted"));
        assert!(preview.len() <= 9000);
        assert_eq!(result["call_id"], "call-456");
    }

    #[test]
    fn preserves_small_results_and_repository_rules() {
        let small = json!({"executed": false, "denied": true});
        assert_eq!(bounded(small.clone(), 256, "t", "c"), small);
        let rules = json!([{"path": "AGENTS.md", "content": "Preserve user changes."}]);
        let result = bounded(
            json!({"executed":false,"repository_instructions":rules,"output":"a".repeat(1000)}),
            256,
            "t",
            "c",
        );
        assert_eq!(result["repository_instructions"], rules);
        assert_eq!(result["executed"], false);
    }

    #[test]
    fn bounds_unicode_and_oversized_errors_without_panicking() {
        let text = "Inicio 🦀 中文 fin".repeat(100);
        for budget in 0..400 {
            assert!(head_tail(&text, budget).len() <= budget);
        }
        let result = bounded(json!({"error":text}), 256, "t", "c");
        assert_eq!(result["error"]["truncated"], true);
        assert!(result["error"]["preview"].as_str().unwrap().len() <= 512);
    }

    #[test]
    fn search_receipts_keep_coverage_failures_and_point_to_full_evidence() {
        let output = json!({"backend":"local_fts5", "query":"Rust", "results":[
            {"page":{"passage":"evidence ".repeat(1000)}}, {"page":null}
        ],"source_reads":[{"status":"failed","failure_kind":"blocked"}],
            "discovery":{"enabled":true,"sources":[{"status":"failed","failure_kind":"timeout"}],"unvisited_domains":["example.com"]}});
        let receipt = bounded(output, 1024, "task", "search");
        assert_eq!(receipt["truncated"], true);
        assert_eq!(receipt["backend"], "local_fts5");
        assert_eq!(receipt["discovery"]["enabled"], true);
        assert_eq!(receipt["search_summary"]["returned_results"], 2);
        assert_eq!(receipt["search_summary"]["unread_results"], 1);
        assert_eq!(receipt["search_summary"]["sources_failed"], 2);
        assert_eq!(receipt["search_summary"]["sources_blocked"], 1);
        assert_eq!(receipt["search_summary"]["unvisited_domains"], 1);
        assert_eq!(receipt["call_id"], "search");
        assert!(
            receipt["results"].is_null(),
            "A receipt must not impersonate complete evidence"
        );
    }
}
