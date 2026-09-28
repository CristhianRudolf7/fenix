//! Track actual comparison failures, without treating every diagnostic as a test.
use super::*;
use sha2::{Digest, Sha256};

fn comparison_command(command: &str) -> bool {
    Regex::new(r"(?m)(?:^|[;\n]|&&|\bdo)\s*(?:/usr/bin/)?(?:cmp|diff)(?:\s|$)")
        .unwrap()
        .is_match(command)
        && !["--help", "--version"]
            .iter()
            .any(|word| command.contains(word))
}

fn masks_failure(command: &str) -> bool {
    if command.split("||").skip(1).any(|tail| {
        !Regex::new(r"^\s*(?:exit|return)\s+[1-9][0-9]*(?:\s|;|$)")
            .unwrap()
            .is_match(tail)
    }) || Regex::new(r"(?:^|[^|])\|(?:[^|]|$)")
        .unwrap()
        .is_match(command)
    {
        return true;
    }
    Regex::new(r"\|\|\s*(?:true|:)(?:\s|;|$)|(?:;|\n)\s*(?:true|exit\s+0)\s*;?\s*$")
        .unwrap()
        .is_match(command)
}

impl Runtime {
    pub(super) fn record_comparison(
        &self,
        task: &str,
        call: &str,
        observation: &Value,
        output: &Value,
        error: bool,
        state: &mut Value,
    ) -> Result<()> {
        let command = observation["command"].as_str().unwrap_or("");
        if !comparison_command(command) || !output["exit_code"].is_number() {
            return Ok(());
        }
        // Replaying log pages or polling a finished process is not new evidence.
        if let Some(pid) = output["process_id"]
            .as_str()
            .or(observation["process_id"].as_str())
        {
            let key = format!("{task}:{pid}");
            if self
                .store
                .get::<bool>("comparison_seen_processes", &key)?
                .unwrap_or(false)
            {
                return Ok(());
            }
            self.store.put("comparison_seen_processes", &key, &true)?;
        }
        let sequence = state["comparison_sequence"].as_u64().unwrap_or(0) + 1;
        state["comparison_sequence"] = json!(sequence);
        let text = observation["output"].as_str().unwrap_or("");
        let discrepancy =
            Regex::new(r"(?im)differ: byte|^Files .+ differ$|^Binary files .+ differ$")
                .unwrap()
                .is_match(text);
        let passed = !error && output["exit_code"] == 0 && !discrepancy && !masks_failure(command);
        self.store.put(
            "comparison_receipts",
            &format!("{task}:{call}"),
            &json!({
                "call_id":call,"command":command,"cwd":observation["cwd"],
                "passed":passed,"revision":observation["revision"],"sequence":sequence,
            }),
        )?;
        let fingerprint = format!(
            "{:x}",
            Sha256::digest(json!([command, observation["cwd"]]).to_string().as_bytes())
        );
        let id = format!("comparison_{}", &fingerprint[..24]);
        let items = state["items"].as_array_mut().unwrap();
        if !passed && !items.iter().any(|item| item["id"] == id) {
            items.push(json!({"id":id,"kind":"comparison","command":command,"cwd":observation["cwd"],
                "source_quote":"Observed comparison did not pass; inspect the fixture before diagnosing the implementation.",
                "instruction":"Rerun this comparison successfully, or use task_contract resolve_check with a source-grounded explanation and a fresh successful corrected comparison receipt. Validate record formats, paths and identical starting data first."}));
        }
        if let Some(item) = items.iter_mut().find(|item| item["id"] == id) {
            item["status"] = json!(if passed { "passed" } else { "failed" });
            item["call_id"] = json!(call);
            item["checked_revision"] = observation["revision"].clone();
            item["last_exit_code"] = output["exit_code"].clone();
            item["last_output"] = json!(crate::redaction::truncate(text, 1500));
            if !passed {
                item["failure_sequence"] = json!(sequence);
                item.as_object_mut().unwrap().remove("resolution");
            }
        }
        Ok(())
    }

    pub(crate) fn resolve_completion_comparison(&self, task: &str, args: &Value) -> Result<()> {
        let mut state = self.completion_requirements(task)?;
        let reason = args["reason"].as_str().unwrap_or("").trim();
        let quote = args["source_quote"].as_str().unwrap_or("");
        ensure!(
            (20..=2000).contains(&reason.len()),
            "Explain the fixture correction and why it preserves the requested behavior"
        );
        ensure!(
            !quote.is_empty()
                && quote.len() <= 2000
                && self.store.task(task)?.request.contains(quote),
            "Cite an exact task quote grounding the corrected comparison; do not invent requirements"
        );
        let call = args["evidence_call_id"].as_str().unwrap_or("");
        let receipt = self.store.get::<Value>("comparison_receipts", &format!("{task}:{call}"))?
            .ok_or_else(|| anyhow::anyhow!("Provide a real comparison receipt from this task, not prose or another task's result"))?;
        ensure!(
            receipt["passed"] == true && receipt["revision"] == state["revision"],
            "The corrected comparison must have passed after the latest edit; unrelated successful commands cannot resolve it"
        );
        let item = state["items"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|item| item["id"] == args["requirement_id"])
            .ok_or_else(|| anyhow::anyhow!("Unknown verification requirement"))?;
        ensure!(
            item["kind"] == "comparison",
            "Requested suites and explicit checks cannot be waived or replaced"
        );
        ensure!(
            receipt["sequence"].as_u64() > item["failure_sequence"].as_u64(),
            "Use a corrected comparison run after the unresolved failure"
        );
        item["status"] = json!("passed");
        item["checked_revision"] = receipt["revision"].clone();
        item["resolution"] = json!({"reason":reason,"source_quote":quote,"evidence_call_id":call,
            "command":receipt["command"],"cwd":receipt["cwd"],"semantic_relevance":"agent_explanation_not_independent_proof"});
        // Retain the failed receipt alongside the replacement; never rewrite its outcome.
        self.store.put("completion_requirements", task, &state)?;
        self.store.event(task,"comparison_resolved",&json!({"requirement_id":args["requirement_id"],"evidence_call_id":call,"reason":reason,"source_quote":quote}))?;
        Ok(())
    }
}
