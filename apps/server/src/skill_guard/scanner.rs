//! Fénix Sentinel: original conservative rules, independent of third-party scanners.
use anyhow::{Result, ensure};
use fenix_agent::skills::bundle::Bundle;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};

pub const POLICY: &str = "sentinel-1";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    pub rule: String,
    pub severity: String,
    pub file: String,
    pub line: usize,
    pub explanation: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capability {
    pub id: String,
    pub description: String,
    pub evidence_file: String,
    pub evidence: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assessment {
    pub safe: bool,
    pub complete: bool,
    pub summary: String,
    pub inspected_files: Vec<String>,
    pub static_review: BTreeMap<String, String>,
    pub capabilities: Vec<Capability>,
    pub findings: Vec<Finding>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edit {
    pub file: String,
    pub before: String,
    pub after: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Repair {
    pub explanation: String,
    pub edits: Vec<Edit>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preservation {
    pub id: String,
    pub preserved: bool,
    pub evidence_file: String,
    pub evidence: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Verification {
    pub safe: bool,
    pub complete: bool,
    pub preservation: Vec<Preservation>,
    pub summary: String,
}

pub fn scan(bundle: &Bundle) -> Vec<Finding> {
    type Rule = (&'static str, &'static str, Regex, &'static str);
    static RULES: OnceLock<Vec<Rule>> = OnceLock::new();
    let rules = RULES.get_or_init(|| [
        ("instruction-override", "high", r"(?i)(ignore|disregard|ignora|omite).{0,35}(previous|system|prior|anteriores|sistema).{0,25}(instruction|prompt|instrucci)|(?i)never refuse|nunca rechaces", "Attempts to override the agent's governing instructions"),
        ("hidden-control", "high", "[\u{200b}\u{200c}\u{200d}\u{202a}-\u{202e}\u{2066}-\u{2069}]", "Invisible or directional control characters require review"),
        ("secret-material", "critical", r"-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----|\b(?:sk-[A-Za-z0-9_-]{24,}|gh[pousr]_[A-Za-z0-9]{30,}|AKIA[A-Z0-9]{16}|rp_[a-z0-9]{30,})\b|[0-9]{8,12}:[A-Za-z0-9_-]{35}", "Possible credential material; remove it before publication"),
        ("remote-execution", "high", r"(?i)(curl|wget).{0,180}\|\s*(sudo\s+)?(bash|sh|zsh)|Invoke-Expression|Invoke-WebRequest.{0,120}\|.{0,20}iex", "Downloads content directly into an interpreter"),
        ("dynamic-execution", "high", r"\b(eval|exec)\s*\(|\b(base64|b64decode|fromhex)\b", "Dynamic or encoded execution needs semantic review"),
        ("unsafe-deserialization", "high", r"pickle\.loads?\s*\(|yaml\.load\s*\(|unserialize\s*\(|Marshal\.load", "Deserialization may execute attacker-controlled code"),
        ("credential-access", "high", r"(?i)(\.ssh/|\.aws/credentials|/etc/shadow|\.env\b|os\.environ|process\.env)", "Reads potential credentials or sensitive configuration"),
        ("external-transfer", "medium", r"(?i)(requests\.post|fetch\s*\(|curl.{0,80}(-d |--data|--upload)|sendBeacon|webhook)", "Potential outbound transfer; verify destination and authorization"),
        ("internal-network", "high", r"169\.254\.169\.254|metadata\.google\.internal|127\.0\.0\.1|localhost|0\.0\.0\.0", "Access to internal services requires an explicit legitimate purpose"),
        ("privileged-host", "high", r"(?i)(docker\.sock|--privileged|privileged:\s*true|hostNetwork:\s*true|sudo\s|chmod\s+777)", "Requests privileged host capabilities"),
        ("destructive-command", "high", r"(?i)rm\s+-[a-z]*r[a-z]*f|rm\s+-[a-z]*f[a-z]*r|mkfs\b|DROP\s+DATABASE", "Potentially destructive operation"),
        ("persistent-change", "medium", r"(?i)(crontab|systemctl\s+enable|\.bashrc|\.profile|LaunchAgents)", "Creates or modifies persistent execution"),
        ("memory-control", "high", r"(?i)(overwrite|replace|rewrite|sobrescribe|reemplaza).{0,50}(MEMORY\.md|SOUL\.md|AGENTS\.md|system prompt)", "Changes persistent agent instructions or memory"),
        ("unsafe-output", "medium", r"innerHTML\s*=|dangerouslySetInnerHTML|shell\s*=\s*True", "Untrusted output may become executable content"),
        ("model-redirection", "medium", r"(?i)(OPENAI_BASE_URL|ANTHROPIC_BASE_URL|api_key\s*=|base_url\s*=)", "May redirect model traffic or override credentials"),
        ("unpinned-install", "medium", r"(?i)(pip install|npm install|npx |uvx |cargo install|docker pull)", "External dependency installation requires provenance review"),
        ("agent-config-access", "medium", r"(?i)(\.codex/|\.claude/|mcp\.json|claude_desktop_config)", "Reads or changes another agent's configuration"),
    ].into_iter().map(|(id, severity, expression, message)| (id, severity, Regex::new(expression).expect("Valid Sentinel rule"), message)).collect());
    let mut findings = Vec::new();
    for (path, content) in &bundle.files {
        for (id, severity, pattern, explanation) in rules {
            for hit in pattern.find_iter(content).take(4) {
                findings.push(Finding {
                    rule: (*id).into(),
                    severity: (*severity).into(),
                    file: path.clone(),
                    line: content[..hit.start()]
                        .bytes()
                        .filter(|b| *b == b'\n')
                        .count()
                        + 1,
                    explanation: (*explanation).into(),
                });
            }
        }
    }
    findings
}
fn evidence(bundle: &Bundle, file: &str, quote: &str) -> bool {
    !quote.trim().is_empty()
        && quote.len() <= 4000
        && bundle
            .files
            .get(file)
            .is_some_and(|text| text.contains(quote))
}
pub fn validate_assessment(bundle: &Bundle, report: &Assessment) -> Result<()> {
    let covered: BTreeSet<_> = report.inspected_files.iter().collect();
    ensure!(
        covered.len() == bundle.files.len()
            && report.inspected_files.len() == bundle.files.len()
            && bundle.files.keys().all(|path| covered.contains(path)),
        "Auditor did not account for every file"
    );
    ensure!(
        scan(bundle).iter().all(|finding| report
            .static_review
            .get(&finding.rule)
            .is_some_and(|reason| !reason.trim().is_empty() && reason.len() <= 2000)),
        "Auditor did not resolve every static signal"
    );

    ensure!(
        !report.capabilities.is_empty() && report.capabilities.len() <= 64,
        "Auditor did not establish a bounded capability contract"
    );
    let mut ids = BTreeSet::new();
    for c in &report.capabilities {
        ensure!(
            !c.id.is_empty()
                && c.id.len() <= 80
                && c.description.len() <= 2000
                && ids.insert(&c.id)
                && evidence(bundle, &c.evidence_file, &c.evidence),
            "Invalid capability evidence"
        );
    }
    ensure!(
        report.findings.len() <= 128 && report.summary.len() <= 8000,
        "Oversized review"
    );
    for f in &report.findings {
        ensure!(
            bundle.files.contains_key(&f.file)
                && f.line > 0
                && f.explanation.len() <= 4000
                && ["low", "medium", "high", "critical"].contains(&f.severity.as_str()),
            "Invalid finding"
        );
    }
    ensure!(
        !report.safe || report.findings.is_empty(),
        "Inconsistent review verdict"
    );
    Ok(())
}
pub fn apply_repair(original: &Bundle, repair: &Repair) -> Result<Bundle> {
    ensure!(
        !repair.edits.is_empty() && repair.edits.len() <= 24,
        "Repair must contain 1–24 anchored edits"
    );
    let mut result = original.clone();
    for edit in &repair.edits {
        let content = result
            .files
            .get_mut(&edit.file)
            .ok_or_else(|| anyhow::anyhow!("Repairs cannot add or remove files"))?;
        ensure!(
            !edit.before.is_empty() && content.matches(&edit.before).count() == 1,
            "Repair anchor must match exactly once"
        );
        *content = content.replacen(&edit.before, &edit.after, 1);
    }
    result.validate()?;
    ensure!(result.hash() != original.hash(), "Repair made no change");
    Ok(result)
}
/// Structural invariants are independent of the model's claims. Executable changes
/// remain a proposal until a real regression suite can establish preservation.
pub fn structural_preservation(original: &Bundle, candidate: &Bundle) -> bool {
    if original.files.keys().ne(candidate.files.keys())
        || original.executable != candidate.executable
    {
        return false;
    }
    original.files.iter().all(|(path, before)| {
        let after = &candidate.files[path];
        if before == after {
            return true;
        }
        if !path.ends_with(".md") || original.executable.contains(path) {
            return false;
        }
        protected_markdown(before) == protected_markdown(after)
    })
}
fn protected_markdown(text: &str) -> Vec<String> {
    let mut fence = false;
    let mut frontmatter = text.starts_with("---\n") || text.starts_with("---\r\n");
    let mut protected = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if frontmatter {
            protected.push(line.into());
            if index > 0 && line == "---" {
                frontmatter = false;
            }
            continue;
        }
        let trim = line.trim_start();
        if trim.starts_with("```") || trim.starts_with("~~~") {
            fence = !fence;
            protected.push(line.into());
            continue;
        }
        if fence
            || trim.starts_with('#')
            || trim.contains('`')
            || trim.contains("](")
            || trim.starts_with("description:")
            || trim.starts_with("required_tools:")
        {
            protected.push(line.into());
        }
    }
    protected
}
pub fn validate_preservation(original: &Assessment, candidate: &Bundle, v: &Verification) -> bool {
    v.safe
        && v.complete
        && v.preservation.len() == original.capabilities.len()
        && original.capabilities.iter().all(|c| {
            let matches: Vec<_> = v.preservation.iter().filter(|p| p.id == c.id).collect();
            matches.len() == 1
                && matches[0].preserved
                && evidence(candidate, &matches[0].evidence_file, &matches[0].evidence)
        })
}
