//! Security specialization uses the existing worker pool, never a second orchestrator.
use crate::{Runtime, skills::Skill};
use anyhow::{Result, bail};
use serde_json::{Value, json};

// Built-in identities remain specialized even when an older installed copy has
// no category metadata. Categories also support newly installed private skills.
const SECURITY_SKILLS: &[&str] = &[
    "security-code-review",
    "api-security-testing",
    "threat-modeling",
    "dependency-and-secret-audit",
    "agent-security-evaluation",
    "detection-engineering",
    "identity-access-review",
    "incident-response",
    "privacy-redaction",
    "privacy-and-compliance",
    "security-artifact-triage",
    "skill-source-review",
    "vulnerability-management",
    "phishing-triage",
    "security-change-review",
    "security-finding-validation",
    "security-static-analysis",
    "security-rust-boundaries",
    "security-property-testing",
    "security-fix-validation",
];
const SUPPORT_SKILLS: &[&str] = &[
    "repository-navigation",
    "local-file-work",
    "test-driven-repair",
    "test-strategy",
    "environment-and-dependencies",
    "code-review",
    "browser-navigation",
    "web-research",
    "git-recovery-and-change-management",
    "completion-verification",
];
pub(crate) fn skill_allowed(profile: &str, skill: &Skill) -> bool {
    let security = skill.category == "security" || SECURITY_SKILLS.contains(&skill.name.as_str());
    match profile {
        "security" => security || SUPPORT_SKILLS.contains(&skill.name.as_str()),
        "general" => !security,
        _ => false,
    }
}
impl Runtime {
    pub(crate) fn profile_for_skills(&self, names: &[String]) -> Result<String> {
        let skills = self.skills()?;
        Ok(if skills
            .iter()
            .any(|s| names.contains(&s.name) && !skill_allowed("general", s))
        {
            "security"
        } else {
            "general"
        }
        .into())
    }
    pub(crate) fn task_profile(&self, task: &str) -> Result<String> {
        if self.store.task(task)?.origin == "subagent" {
            Ok(self.store.subagent(task)?.spec.profile)
        } else {
            Ok("general".into())
        }
    }
    pub(crate) fn authorize_skill(&self, task: &str, name: &str) -> Result<()> {
        let profile = self.task_profile(task)?;
        let skill = self.skills()?.into_iter().find(|s| s.name == name);
        if !skill.is_some_and(|s| s.enabled && skill_allowed(&profile, &s)) {
            bail!(
                "Skill is unavailable to this task profile; ask the master to delegate a security audit when appropriate"
            );
        }
        Ok(())
    }
    pub(crate) fn security_tool_allowed(&self, task: &str, name: &str) -> Result<bool> {
        if self.task_profile(task)? != "security" {
            return Ok(name != "security_scan_read");
        }
        Ok(matches!(
            name,
            "security_scan_read"
                | "tool_search"
                | "skill_list"
                | "skill_search"
                | "skill_load"
                | "file_read"
                | "files_list"
                | "files_glob"
                | "files_search"
                | "file_write"
                | "file_edit"
                | "file_patch"
                | "code_query"
                | "command_run"
                | "command_status"
                | "command_list"
                | "command_output"
                | "command_input"
                | "command_cancel"
                | "task_inspect"
                | "execution_status"
                | "workspace_checkpoint"
                | "worktree_manage"
                | "project_spec"
                | "web_search"
                | "web_read"
                | "ask_user"
                | "request_approval"
                | "subagent_finish"
                | "plan_review_read"
                | "plan_evidence"
                | "browser_act"
                | "browser_check"
                | "browser_close"
                | "browser_inspect"
                | "browser_observe"
                | "browser_open"
                | "browser_resize"
                | "browser_tabs"
                | "browser_wait"
        ))
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
    async fn profile_controls_discovery_loading_resources_and_tool_access() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills"))).await?;
        let parent =
            rt.store
                .enqueue_bound("Audit the fixture", "web", None, &TaskBinding::default())?;
        rt.store.claim_next()?;
        let general = rt.store.create_subagent(
            &parent.id,
            &SubagentSpec {
                profile: "general".into(),
                objective: "Read fixture".into(),
                expected_output: "A summary".into(),
                context: String::new(),
                constraints: String::new(),
            },
        )?;
        let delegated = rt.spawn_subagent(&parent.id, &json!({"profile":"security","objective":"Audit source only","expected_output":"Evidence with coverage","constraints":"Only the fixture repository; no network testing or code changes"})).await?;
        let child = delegated["subagent_id"].as_str().unwrap();
        assert_eq!(rt.task_profile(child)?, "security");
        for task in [&parent.id, &general.id] {
            assert!(rt.authorize_skill(task, "security-code-review").is_err());
            assert!(
                rt.authorize_project_tool(
                    task,
                    "skill_load",
                    &json!({"name":"security-rust-boundaries","resource":"references/rust.md"})
                )
                .is_err()
            );
            assert!(
                rt.authorize_project_tool(
                    task,
                    "security_scan_read",
                    &json!({"path":"result.json","format":"sarif"})
                )
                .is_err()
            );
        }
        let settings = crate::config::Settings::initial(dir.path().to_owned());
        let cancel = tokio_util::sync::CancellationToken::new();
        let artifact = dir.path().join("scan.sarif");
        std::fs::write(
            &artifact,
            serde_json::to_vec(
                &json!({"version":"2.1.0","runs":[{"invocations":[{"executionSuccessful":true}],"results":[]}]}),
            )?,
        )?;
        let parsed = rt
            .execute_tool(
                child,
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
                &parent.id,
                "web",
                &settings,
                "security_scan_read",
                &json!({"path":"scan.sarif","format":"sarif"}),
                &cancel
            )
            .await
            .is_err()
        );
        assert!(
            rt.execute_tool(
                child,
                "web",
                &settings,
                "security_scan_read",
                &json!({"path":"../scan.sarif","format":"sarif"}),
                &cancel
            )
            .await
            .is_err()
        );
        rt.authorize_skill(child, "security-code-review")?;
        rt.authorize_skill(child, "security-rust-boundaries")?;
        rt.authorize_skill(child, "repository-navigation")?;
        assert!(rt.authorize_skill(child, "meal-planning").is_err());
        rt.set_skill_enabled("security-rust-boundaries", false)?;
        assert!(
            rt.authorize_skill(child, "security-rust-boundaries")
                .is_err()
        );
        let custom = rt.skills_dir.join("custom-audit");
        std::fs::create_dir(&custom)?;
        std::fs::write(
            custom.join("SKILL.md"),
            "---\nname: custom-audit\ndescription: zebraflange audit\nmetadata:\n  category: security\n---\nPrivate security procedure\n",
        )?;
        assert!(rt.authorize_skill(&parent.id, "custom-audit").is_err());
        rt.authorize_skill(child, "custom-audit")?;
        let general_matches = rt.search_skills_for_profile(Some("general"), "zebraflange", 8)?;
        assert_eq!(general_matches["skills"], json!([]));
        assert!(
            !rt.skill_guidance_for_profile(Some("general"), "zebraflange")?
                .contains("custom-audit")
        );
        assert_eq!(
            rt.search_skills_for_profile(Some("security"), "zebraflange", 8)?["skills"][0]["skill"]
                ["name"],
            "custom-audit"
        );
        let mut offset = 0;
        loop {
            let page = rt.list_skills_for_profile(Some("general"), offset, 20)?;
            for entry in page["skills"].as_array().unwrap() {
                let skill: Skill = serde_json::from_value(entry["skill"].clone())?;
                assert!(skill_allowed("general", &skill));
            }
            match page["next_offset"].as_u64() {
                Some(next) => offset = next as usize,
                None => break,
            }
        }
        let catalog = rt.task_tool_catalog(child)?;
        assert!(catalog.iter().any(|t| t.name == "security_scan_read"));
        for forbidden in [
            "subagent_spawn",
            "subagent_send",
            "memory_manage",
            "schedule_manage",
            "integration_call",
        ] {
            assert!(!catalog.iter().any(|t| t.name == forbidden));
            assert!(
                rt.authorize_project_tool(child, forbidden, &json!({}))
                    .is_err()
            );
        }
        assert!(rt.spawn_subagent(child,&json!({"profile":"security","objective":"Nested","expected_output":"No","constraints":"Fixture only"})).await.is_err());
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
