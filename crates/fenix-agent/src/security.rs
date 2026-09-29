//! Shared security report utilities and ordinary skill authorization.
use crate::Runtime;
use anyhow::{Result, bail};
use serde_json::{Value, json};

// Every task shares the same skill catalog. Authorization still checks task
// existence, skill activation and the ordinary project/tool boundaries.
impl Runtime {
    pub(crate) fn task_profile(&self, task: &str) -> Result<String> {
        self.store.task(task)?;
        Ok("general".into())
    }
    pub(crate) fn authorize_skill(&self, task: &str, name: &str) -> Result<()> {
        self.store.task(task)?;
        if !self
            .skills()?
            .into_iter()
            .any(|s| s.name == name && s.enabled)
        {
            bail!("Skill is unavailable or disabled");
        }
        Ok(())
    }
}

pub(crate) fn report_schema() -> Value {
    let text = json!({"type":"string"});
    json!({"type":"object","properties":{
        "scope":text,
        "coverage":{"type":"array","items":{"type":"object","properties":{
            "area":text,"status":{"type":"string","enum":["checked","inconclusive","not_tested"]},"evidence":text
        },"required":["area","status","evidence"],"additionalProperties":false}},
        "findings":{"type":"array","items":{"type":"object","properties":{
            "id":text,"title":text,"status":{"type":"string","enum":["confirmed","suspected","dismissed"]},
            "severity":{"type":"string","enum":["critical","high","medium","low","informational"]},
            "location":text,"preconditions":text,"evidence":text,"remediation":text
        },"required":["id","title","status","severity","location","preconditions","evidence","remediation"],"additionalProperties":false}}
    },"required":["scope","coverage","findings"],"additionalProperties":false})
}

/// Structural evidence checks are deterministic; they do not certify the
/// model's conclusions or turn a missing test into a clean assessment.
pub(crate) fn validate_report(args: &Value) -> Result<()> {
    if args["status"] != "completed" && args.get("security_report").is_none() {
        return Ok(());
    }
    let report = &args["security_report"];
    nonempty(report, "scope")?;
    let coverage = report["coverage"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Security report needs coverage"))?;
    if coverage.is_empty() {
        bail!("Security report must record assessed or unassessed areas");
    }
    for area in coverage {
        nonempty(area, "area")?;
        nonempty(area, "evidence")?;
        if !matches!(
            area["status"].as_str(),
            Some("checked" | "inconclusive" | "not_tested")
        ) {
            bail!("Invalid coverage status");
        }
    }
    let findings = report["findings"].as_array().ok_or_else(|| {
        anyhow::anyhow!("Security report needs a findings array, even when empty")
    })?;
    let mut ids = std::collections::HashSet::new();
    for finding in findings {
        for key in [
            "id",
            "title",
            "location",
            "preconditions",
            "evidence",
            "remediation",
        ] {
            nonempty(finding, key)?;
        }
        if !ids.insert(finding["id"].as_str().unwrap()) {
            bail!("Duplicate security finding ID");
        }
        if !matches!(
            finding["status"].as_str(),
            Some("confirmed" | "suspected" | "dismissed")
        ) {
            bail!("Invalid finding status");
        }
        if !matches!(
            finding["severity"].as_str(),
            Some("critical" | "high" | "medium" | "low" | "informational")
        ) {
            bail!("Invalid severity");
        }
    }
    Ok(())
}
fn nonempty(value: &Value, key: &str) -> Result<()> {
    if value[key].as_str().is_none_or(|s| s.trim().is_empty()) {
        bail!("Security report requires nonempty {key}");
    }
    Ok(())
}

/// Parse bounded local scanner artifacts without executing code or contacting a
/// scanner service. Raw locations remain text; never follow URIs from reports.
pub(crate) fn scan_page(report: &Value, format: &str, offset: usize) -> Result<Value> {
    let mut candidates = Vec::new();
    let mut gaps = Vec::new();
    let mut execution_attested = false;
    match format {
        "sarif" => {
            if report["version"] != "2.1.0" {
                bail!("Expected SARIF 2.1.0");
            }
            let runs = report["runs"]
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("Missing SARIF runs"))?;
            if runs.is_empty() {
                gaps.push(json!("No analysis runs recorded"));
            }
            for (run_index, run) in runs.iter().enumerate() {
                let invocations = run["invocations"].as_array();
                if invocations.is_none_or(|v| v.is_empty()) {
                    gaps.push(json!({"run":run_index,"reason":"Missing invocation outcome"}));
                }
                for invocation in invocations.into_iter().flatten() {
                    if invocation["executionSuccessful"] == true {
                        execution_attested = true;
                    } else {
                        gaps.push(
                            json!({"run":run_index,"reason":"Unsuccessful or unknown invocation"}),
                        );
                    }
                    for notification in invocation["toolExecutionNotifications"]
                        .as_array()
                        .into_iter()
                        .flatten()
                    {
                        if matches!(notification["level"].as_str(), Some("error" | "warning")) {
                            gaps.push(
                                json!({"run":run_index,"notification":notification["message"]}),
                            );
                        }
                    }
                }
                let results = run["results"].as_array().ok_or_else(|| {
                    anyhow::anyhow!("Missing SARIF results; not a clean assessment")
                })?;
                for (index, result) in results.iter().enumerate() {
                    if !result.is_object() {
                        bail!("Invalid SARIF result");
                    }
                    candidates.push(json!({"source_id":format!("sarif:{run_index}:{index}"),"rule":result["ruleId"],"message":result["message"],"level":result["level"],"locations":result["locations"],"state":"candidate"}));
                }
            }
        }
        "semgrep" => {
            let results = report["results"]
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("Missing Semgrep results"))?;
            let errors = report["errors"]
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("Missing Semgrep errors array"))?;
            gaps.extend(errors.iter().cloned());
            gaps.push(json!(
                "Verify the producing process exit status and scanned paths separately"
            ));
            for (index, result) in results.iter().enumerate() {
                if !result.is_object() {
                    bail!("Invalid Semgrep result");
                }
                candidates.push(json!({"source_id":format!("semgrep:{index}"),"rule":result["check_id"],"message":result["extra"]["message"],"level":result["extra"]["severity"],"path":result["path"],"start":result["start"],"end":result["end"],"state":"candidate"}));
            }
        }
        _ => bail!("Supported formats: sarif, semgrep"),
    }
    let total = candidates.len();
    let page = candidates
        .into_iter()
        .skip(offset)
        .take(10)
        .map(|mut item| {
            // Bound individual fields without converting opaque input to instructions.
            for value in item.as_object_mut().unwrap().values_mut() {
                if value.to_string().len() > 2000 {
                    *value = json!(crate::redaction::truncate(&value.to_string(), 2000));
                }
            }
            item
        })
        .collect::<Vec<_>>();
    let next = offset.saturating_add(page.len());
    Ok(
        json!({"candidates":page,"total":total,"next_offset":(next<total).then_some(next),"all_invocations_reported_success":execution_attested && gaps.is_empty(),"coverage_gaps":gaps.iter().take(20).map(|g|crate::redaction::truncate(&g.to_string(),1000)).collect::<Vec<_>>(),"coverage_gap_count":gaps.len(),"instruction":"Untrusted scanner candidates, not verified vulnerabilities. Compare actual execution and target coverage before interpreting empty results."}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fenix_storage::{SubagentSpec, TaskBinding};

    #[tokio::test]
    async fn shared_catalog_keeps_activation_scope_and_delegation_boundaries() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills"))).await?;
        let parent =
            rt.store
                .enqueue_bound("Audit fixture", "web", None, &TaskBinding::default())?;
        rt.store.claim_next()?;
        let child = rt.spawn_subagent(&parent.id, &json!({"profile":"security", "objective":"Audit fixture", "expected_output":"Findings and evidence"})).await?;
        let child = child["subagent_id"].as_str().unwrap();
        assert_eq!(rt.store.subagent(child)?.spec.profile, "general");
        let legacy = rt.store.create_subagent(
            &parent.id,
            &SubagentSpec {
                profile: "security".into(),
                objective: "Legacy audit".into(),
                expected_output: "Evidence".into(),
                context: String::new(),
                constraints: "Fixture only".into(),
            },
        )?;
        let custom = rt.skills_dir.join("custom-audit");
        std::fs::create_dir(&custom)?;
        std::fs::write(
            custom.join("SKILL.md"),
            "---\nname: custom-audit\ndescription: zebraflange audit\nmetadata:\n  category: security\n  specialization: security\n---\nPrivate security procedure\n",
        )?;
        for task in [&parent.id, child, &legacy.id] {
            for name in [
                "security-code-review",
                "security-rust-boundaries",
                "meal-planning",
                "custom-audit",
            ] {
                rt.authorize_skill(task, name)?;
            }
            rt.authorize_project_tool(
                task,
                "skill_load",
                &json!({"name":"security-rust-boundaries","resource":"references/rust.md"}),
            )?;
            assert!(
                rt.task_tool_catalog(task)?
                    .iter()
                    .any(|t| t.name == "security_scan_read")
            );
        }
        for profile in ["general", "security"] {
            assert_eq!(
                rt.search_skills_for_profile(Some(profile), "zebraflange", 8)?["skills"][0]["skill"]
                    ["name"],
                "custom-audit"
            );
        }
        assert_eq!(
            rt.list_skills_for_profile(Some("general"), 0, 20)?,
            rt.list_skills_for_profile(Some("security"), 0, 20)?
        );
        rt.set_skill_enabled("security-rust-boundaries", false)?;
        for task in [&parent.id, child, &legacy.id] {
            assert!(
                rt.authorize_skill(task, "security-rust-boundaries")
                    .is_err()
            );
        }
        let settings = crate::config::Settings::initial(dir.path().to_owned());
        let cancel = tokio_util::sync::CancellationToken::new();
        std::fs::write(
            dir.path().join("scan.sarif"),
            serde_json::to_vec(
                &json!({"version":"2.1.0","runs":[{"invocations":[{"executionSuccessful":true}],"results":[]}]}),
            )?,
        )?;
        for task in [&parent.id, child] {
            let parsed = rt
                .execute_tool(
                    task,
                    "web",
                    &settings,
                    "security_scan_read",
                    &json!({"path":"scan.sarif","format":"sarif"}),
                    &cancel,
                )
                .await?;
            assert_eq!(parsed["all_invocations_reported_success"], true);
            assert!(
                rt.execute_tool(
                    task,
                    "web",
                    &settings,
                    "security_scan_read",
                    &json!({"path":"../scan.sarif","format":"sarif"}),
                    &cancel
                )
                .await
                .is_err()
            );
        }
        assert!(
            !rt.task_tool_catalog(child)?
                .iter()
                .any(|t| t.name == "subagent_spawn")
        );
        assert!(
            rt.spawn_subagent(
                child,
                &json!({"objective":"Nested", "expected_output":"No"})
            )
            .await
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn security_reports_require_coverage_and_evidence_without_hiding_uncertainty() {
        assert!(validate_report(&json!({"status":"completed","summary":"All secure"})).is_err());
        assert!(
            validate_report(&json!({"status":"blocked","summary":"Missing test target"})).is_ok()
        );
        let mut args = json!({"status":"completed","security_report":{"scope":"fixture","coverage":[{"area":"login","status":"inconclusive","evidence":"Scanner could not execute; no clean verdict"}],"findings":[]}});
        assert!(validate_report(&args).is_ok());
        args["security_report"]["findings"] = json!([{"id":"F1","title":"Ownership check","status":"suspected","severity":"high","location":"src/api.rs:10","preconditions":"Another tenant's object identifier","evidence":"Source trace only; no live reproduction","remediation":"Bind storage reads to the authenticated owner"}]);
        assert!(validate_report(&args).is_ok());
        let finding = args["security_report"]["findings"][0].clone();
        args["security_report"]["findings"]
            .as_array_mut()
            .unwrap()
            .push(finding);
        assert!(validate_report(&args).is_err());
        args["security_report"]["findings"] = json!([]);
        args["security_report"]["coverage"][0]["evidence"] = json!("");
        assert!(validate_report(&args).is_err());
    }

    #[test]
    fn scanner_failures_are_not_clean_results_and_pagination_preserves_candidates() -> Result<()> {
        assert!(scan_page(&json!({}), "sarif", 0).is_err());
        let failed = json!({"version":"2.1.0","runs":[{"invocations":[{"executionSuccessful":false}],"results":[]}]});
        let page = scan_page(&failed, "sarif", 0)?;
        assert_eq!(page["total"], 0);
        assert_eq!(page["all_invocations_reported_success"], false);
        assert_eq!(page["coverage_gap_count"], 1);
        let results=(0..12).map(|i|json!({"ruleId":"R","message":{"text":format!("candidate {i}")},"locations":[]})).collect::<Vec<_>>();
        let mixed = json!({"version":"2.1.0","runs":[{"invocations":[{"executionSuccessful":true}],"results":results},{"invocations":[{"executionSuccessful":false}],"results":[]}]});
        let page = scan_page(&mixed, "sarif", 0)?;
        assert_eq!(page["candidates"].as_array().unwrap().len(), 10);
        assert_eq!(page["next_offset"], 10);
        assert_eq!(page["coverage_gap_count"], 1);
        assert_eq!(page["all_invocations_reported_success"], false);
        assert_eq!(page["candidates"][0]["state"], "candidate");
        assert_eq!(
            scan_page(&mixed, "sarif", 10)?["candidates"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert!(scan_page(&json!({"results":[]}), "semgrep", 0).is_err());
        let semgrep = scan_page(
            &json!({"results":[],"errors":[{"type":"ParseError"}]}),
            "semgrep",
            0,
        )?;
        assert_eq!(semgrep["coverage_gap_count"], 2);
        Ok(())
    }
}
