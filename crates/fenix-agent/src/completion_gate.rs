//! Durable verification requirements. Only actual tool results can satisfy a check.
mod comparisons;
mod invocations;

use crate::Runtime;
use anyhow::{Result, bail, ensure};
use regex::Regex;
use serde_json::{Value, json};

fn requested_suite(request: &str) -> Option<String> {
    request
        .lines()
        .find(|line| {
            let text = line.to_lowercase();
            !["do not run", "don't run", "no ejecutes", "no ejecutar"]
                .iter()
                .any(|s| request.to_lowercase().contains(s))
                && [
                    "they should still pass",
                    "run the tests",
                    "run tests",
                    "run the test suite",
                    "tests must pass",
                    "tests should pass",
                    "ejecuta las pruebas",
                    "ejecutar las pruebas",
                ]
                .iter()
                .any(|s| text.contains(s))
        })
        .map(str::to_owned)
}
fn suite_command(command: &str) -> bool {
    Regex::new(r"(?m)(?:^|&&|;|\n)\s*(?:(?:python[0-9.]*)\s+-m\s+)?(?:pytest|unittest|cargo\s+test|go\s+test|(?:npm|pnpm)\s+(?:run\s+)?test)(?:\s|$)").unwrap().is_match(command)
        && !["--version","--help","--collect-only","||", "echo "].iter().any(|s|command.contains(s))
}
fn broad_suite(command: &str, request: &str) -> bool {
    let words: Vec<_> = command.split_whitespace().collect();
    let mut skip = false;
    for (index, word) in words.iter().copied().enumerate() {
        if skip {
            skip = false;
            if !request.contains(word.trim_matches(['\'', '"'])) {
                return false;
            }
            continue;
        }
        if word == "--ignore" || word == "--ignore-glob" {
            skip = true;
            continue;
        }
        if word.starts_with("--ignore=") || word.starts_with("--ignore-glob=") {
            if !request.contains(word.split_once('=').unwrap().1.trim_matches(['\'', '"'])) {
                return false;
            }
            continue;
        }
        if word.contains("::")
            || word == "-k"
            || word == "-m"
                && !words
                    .get(index.wrapping_sub(1))
                    .is_some_and(|w| w.starts_with("python"))
            || word.trim_matches(['\'', '"']).ends_with(".py")
        {
            return false;
        }
    }
    true
}
fn suite_passed(output: &str) -> bool {
    let nonzero = Regex::new(r"\b[1-9][0-9]* (?:passed|tests? passed)\b")
        .unwrap()
        .is_match(output)
        || Regex::new(r"Ran [1-9][0-9]* tests?")
            .unwrap()
            .is_match(output)
            && output.lines().any(|l| l.trim() == "OK")
        || output
            .lines()
            .any(|l| l.starts_with("ok\t") || l.starts_with("ok  "));
    nonzero
        && !Regex::new(r"(?i)\b[1-9][0-9]* (?:failed|errors?)\b|No module named|ERROR collecting")
            .unwrap()
            .is_match(output)
}
fn changed_environment(command: &str) -> bool {
    [
        "pip install",
        "pip uninstall",
        "build_ext",
        "cargo build",
        "npm install",
        "pnpm install",
        "make install",
    ]
    .iter()
    .any(|s| command.contains(s))
}
impl Runtime {
    /// Reuse real successful receipts without another model round just to restate
    /// them. This does not satisfy or waive any acceptance/completion check.
    pub(crate) fn completion_receipt_evidence(&self, task: &str) -> Result<String> {
        let contract = self.task_contract(task)?;
        let receipts = contract["recent_tool_evidence"].as_array().into_iter().flatten().rev()
            .filter(|r| r["is_error"] == false && r["executed"] != false)
            .filter(|r| match r["tool"].as_str() {
                Some("command_run" | "command_status" | "command_input") => r["exit_code"] == 0,
                Some("file_write" | "file_edit" | "file_patch") => r["error"].is_null(),
                _ => false,
            })
            .take(6).map(|r|json!({"call_id":r["call_id"],"tool":r["tool"],"artifact":r["artifact"],"exit_code":r["exit_code"]})).collect::<Vec<_>>();
        ensure!(
            !receipts.is_empty(),
            "Provide observed evidence; no successful execution receipts are available"
        );
        Ok(format!(
            "Engine-recorded receipts (not a waiver of completion checks): {}",
            json!(receipts)
        ))
    }

    pub(crate) fn completion_requirements(&self, task: &str) -> Result<Value> {
        let mut state = self
            .store
            .get::<Value>("completion_requirements", task)?
            .unwrap_or_else(|| json!({"revision":0,"items":[]}));
        let owner = self.store.task(task)?;
        invocations::register(&owner.request, &mut state);
        if let Some(source) = requested_suite(&owner.request)
            && !state["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["id"] == "requested_test_suite")
        {
            state["items"].as_array_mut().unwrap().push(json!({"id":"requested_test_suite","kind":"test_suite","source_quote":source,"status":"pending","instruction":"Run the requested repository suite with the owner's exclusions. A smoke example, imports, installing the test runner or a selected single test cannot replace this check."}));
            self.store.put("completion_requirements", task, &state)?;
        }
        Ok(state)
    }
    pub(crate) fn require_completion_check(&self, task: &str, args: &Value) -> Result<()> {
        let owner = self.store.task(task)?;
        let quote = args["source_quote"].as_str().unwrap_or("");
        let command = args["command"].as_str().unwrap_or("");
        let id = args["requirement_id"].as_str().unwrap_or("");
        ensure!(
            !quote.is_empty() && quote.len() <= 2000 && owner.request.contains(quote),
            "A requirement must cite an exact quote from the task assignment"
        );
        ensure!(
            Regex::new(r"^[a-z][a-z0-9_-]{0,63}$").unwrap().is_match(id)
                && !command.trim().is_empty()
                && command.len() <= 8000,
            "Provide a stable requirement_id and bounded verification command"
        );
        let mut state = self.completion_requirements(task)?;
        let items = state["items"].as_array_mut().unwrap();
        if let Some(existing) = items.iter().find(|r| r["id"] == id) {
            ensure!(
                existing["command"] == command
                    && existing["source_quote"] == quote
                    && existing["cwd"] == args["cwd"],
                "Registered checks cannot be weakened or replaced by the agent"
            );
            return Ok(());
        }
        ensure!(items.len() < 24, "Keep completion checks bounded");
        items.push(json!({"id":id,"kind":"command","source_quote":quote,"command":command,"cwd":args["cwd"],"status":"pending"}));
        self.store.put("completion_requirements", task, &state)?;
        Ok(())
    }
    pub(crate) fn record_completion_check(
        &self,
        task: &str,
        call: &str,
        tool: &str,
        args: &Value,
        output: &Value,
        error: bool,
    ) -> Result<()> {
        let mut state = self.completion_requirements(task)?;
        if output["executed"] == false
            || output["denied"] == true
            || !error && crate::execution_state::unchanged_file_edit(tool, output)
        {
            return Ok(());
        }
        if !error && matches!(tool, "file_write" | "file_edit" | "file_patch") {
            state["revision"] = json!(state["revision"].as_u64().unwrap_or(0) + 1);
            self.store.put("completion_requirements", task, &state)?;
            return Ok(());
        }
        if !matches!(tool, "command_run" | "command_status" | "command_output") {
            return Ok(());
        }
        let pid = output["process_id"]
            .as_str()
            .or(args["process_id"].as_str());
        let process_key = pid.map(|p| format!("{task}:{p}"));
        let mut observation = if tool == "command_run" {
            json!({"command":args["command"],"cwd":args["cwd"],"process_id":pid,"output":"","revision":state["revision"]})
        } else {
            let Some(key) = &process_key else {
                return Ok(());
            };
            let Some(prior) = self.store.get::<Value>("completion_processes", key)? else {
                return Ok(());
            };
            prior
        };
        let command = observation["command"].as_str().unwrap_or("").to_owned();
        if tool == "command_run" && changed_environment(&command) {
            state["revision"] = json!(state["revision"].as_u64().unwrap_or(0) + 1);
            observation["revision"] = state["revision"].clone();
        }
        let text = format!(
            "{}{}",
            observation["output"].as_str().unwrap_or(""),
            output["output"].as_str().unwrap_or("")
        );
        observation["output"] = json!(
            text.chars()
                .rev()
                .take(32000)
                .collect::<String>()
                .chars()
                .rev()
                .collect::<String>()
        );
        if let Some(key) = &process_key {
            self.store.put("completion_processes", key, &observation)?;
        }
        self.record_comparison(task, call, &observation, output, error, &mut state)?;
        let is_suite = suite_command(&command);
        let items = state["items"].as_array_mut().unwrap();
        if is_suite && !items.iter().any(|r| r["kind"] == "test_suite") {
            items.push(json!({"id":"attempted_test_suite","kind":"test_suite","source_quote":"Observed verification attempt; success cannot be inferred from unrelated commands","status":"pending"}));
        }
        for requirement in items {
            if requirement["kind"] == "comparison" {
                continue;
            }
            let matched = if requirement["kind"] == "invocation" {
                invocations::matches(requirement, &command, observation["cwd"].as_str())
            } else if requirement["kind"] == "test_suite" {
                is_suite && broad_suite(&command, &self.store.task(task)?.request)
            } else {
                requirement["command"] == command
                    && (requirement["cwd"].is_null() || requirement["cwd"] == observation["cwd"])
            };
            if !matched {
                continue;
            }
            let passed = !error
                && output["exit_code"] == 0
                && (requirement["kind"] != "invocation" || invocations::unmasked(&command))
                && (requirement["kind"] != "test_suite"
                    || suite_passed(observation["output"].as_str().unwrap_or("")));
            requirement["status"] = json!(if passed {
                "passed"
            } else if output["exit_code"].is_number() || error {
                "failed"
            } else {
                "pending"
            });
            requirement["call_id"] = json!(call);
            requirement["checked_revision"] = observation["revision"].clone();
            requirement["last_exit_code"] = output["exit_code"].clone();
            requirement["last_output"] = json!(crate::tool_receipt::head_tail(
                observation["output"].as_str().unwrap_or(""),
                1500
            ));
        }
        self.store.put("completion_requirements", task, &state)?;
        Ok(())
    }
    /// Catch explicit contradictory success claims without a reviewer model.
    /// The optional caveat explanation preserves honest out-of-scope limitations;
    /// existing executable requirements still run afterwards and cannot be waived.
    pub(crate) fn validate_completion_claim(&self, task: &str, args: &Value) -> Result<()> {
        ensure!(
            args["unverified_requirements"]
                .as_array()
                .is_none_or(|items| items.is_empty()),
            "Completion rejected: requested requirements are explicitly unverified. Continue the relevant investigation/check; a written artifact or disclaimer does not satisfy the original objective."
        );
        let text = format!(
            "{} {}",
            args["summary"].as_str().unwrap_or(""),
            args["evidence"].as_str().unwrap_or("")
        )
        .to_lowercase();
        let admission = [
            "unverified",
            "not verified",
            "did not verify",
            "didn't verify",
            "have not verified",
            "not implemented",
            "not completed",
            "still pending",
            "no verifiqué",
            "no verifique",
            "no pude validar",
            "no pude comparar",
            "no implementé",
            "no completé",
            "sigue pendiente",
        ]
        .iter()
        .any(|phrase| text.contains(phrase));
        if admission {
            let note = args["non_required_limitations"]
                .as_str()
                .unwrap_or("")
                .trim();
            ensure!(
                (20..=2000).contains(&note.len()),
                "Completion conflicts with an admitted verification gap. Check the central requested outcome, not just file existence, and continue if it is unresolved. If the caveat concerns ONLY work outside the request, explain that exact boundary in non_required_limitations. Do not reword or hide a real gap; registered checks remain mandatory."
            );
            self.store.event(
                task,
                "completion_limitation",
                &json!({"explanation":note,"classification":"agent_scope_assertion_not_a_waiver"}),
            )?;
        }
        self.validate_task_completion(task)
    }

    pub(crate) fn validate_task_completion(&self, task: &str) -> Result<()> {
        let state = self.completion_requirements(task)?;
        let pending:Vec<_>=state["items"].as_array().unwrap().iter().filter(|r|r["status"]!="passed" || r["checked_revision"]!=state["revision"]).map(|r|json!({"id":r["id"],"status":r["status"],"source_quote":r["source_quote"],"call_id":r["call_id"],"last_exit_code":r["last_exit_code"],"kind":r["kind"],"stale":r["checked_revision"]!=state["revision"]})).collect();
        if !pending.is_empty() {
            self.store.event(
                task,
                "completion_rejected",
                &json!({"pending":pending,"published":false}),
            )?;
            bail!(
                "Completion rejected: mandatory checks are missing, failed or stale: {}. Use task_contract get. Install missing test dependencies within existing authorization, repair failures and rerun the full requested suite after the last edit/install. A smoke example or a statement of success cannot satisfy a missing check. For a failed comparison, first validate the input format, paths and identical initial data. A malformed self-authored fixture is not proof of an implementation defect. Rerun the valid comparison; if its command changed, resolve the old comparison with task_contract resolve_check, its requirement_id, source_quote, reason and a fresh passing evidence_call_id. Do not change status to completed merely to bypass a rejected failed outcome. If genuinely blocked, report blocked with the concrete prerequisite instead of completed.",
                json!(pending)
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
