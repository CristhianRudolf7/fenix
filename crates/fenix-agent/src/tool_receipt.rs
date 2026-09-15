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
        "retrieve": "Use task_inspect with task_id, call_id, offset and length to read the saved output in pages. The preview omits the middle; it is not complete evidence."
    });
    // Keep execution state usable even when it falls in the omitted middle.
    // Repository guidance must remain intact before an agent retries a mutation.
    if let Some(rules) = output.get("repository_instructions") {
        receipt["repository_instructions"] = rules.clone();
    }
    for key in [
        "executed",
        "denied",
        "status",
        "exit_code",
        "process_id",
        "next_cursor",
        "log_complete",
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
}
