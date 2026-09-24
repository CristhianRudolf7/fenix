//! Evidence-backed completion gates. An executor's success is a claim, not an audit.
use crate::{
    Store,
    events::publish,
    id, now,
    plans::{self, Plan},
};
use anyhow::{Context, Result, bail};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Task,
    Requirement,
    Artifact,
    Fact,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceState {
    Pending,
    Verified,
    Doubtful,
    Refuted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceInput {
    pub id: String,
    pub kind: EvidenceKind,
    pub state: EvidenceState,
    pub statement: String,
    pub source: String,
    #[serde(default)]
    pub node_id: Option<String>,
    #[serde(default)]
    pub question: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEntry {
    #[serde(flatten)]
    pub input: EvidenceInput,
    pub author_task_id: String,
    pub updated_at: String,
    pub audit_id: Option<String>,
    pub observations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionCheck {
    pub entry_id: String,
    pub state: EvidenceState,
    pub detail: String,
    /// Successful reads made by this auditor, not invented references or peer claims.
    pub call_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionReport {
    pub verdict: String,
    pub summary: String,
    pub checks: Vec<CompletionCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionAudit {
    pub id: String,
    pub plan_id: String,
    pub version: u64,
    /// Empty for the integrated final audit against the original owner request.
    pub node_id: String,
    pub status: String,
    pub worker_id: Option<String>,
    pub report: Option<CompletionReport>,
    pub created_at: String,
    pub updated_at: String,
}

fn read(db: &Connection, id: &str) -> Result<CompletionAudit> {
    let body: String = db
        .query_row(
            "SELECT body FROM completion_audits WHERE id=?1",
            [id],
            |r| r.get(0),
        )
        .optional()?
        .context("Completion audit not found")?;
    Ok(serde_json::from_str(&body)?)
}

fn write(db: &Connection, audit: &mut CompletionAudit) -> Result<()> {
    audit.updated_at = now();
    db.execute("INSERT INTO completion_audits VALUES (?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(id) DO UPDATE SET worker_id=excluded.worker_id,status=excluded.status,body=excluded.body", params![audit.id,audit.plan_id,audit.version,audit.node_id,audit.worker_id,audit.status,serde_json::to_string(audit)?])?;
    let plan = plans::read(db, &audit.plan_id)?;
    publish(
        db,
        Some(&plan.task_id),
        "plan.audit_updated",
        &json!({"audit":audit}),
    )?;
    Ok(())
}

fn entries(db: &Connection, plan: &str, version: u64) -> Result<Vec<EvidenceEntry>> {
    db.prepare("SELECT body FROM plan_evidence WHERE plan_id=?1 AND version=?2 ORDER BY entry_id")?
        .query_map(params![plan, version], |r| r.get::<_, String>(0))?
        .map(|r| Ok(serde_json::from_str(&r?)?))
        .collect()
}

fn save_entry(db: &Connection, plan: &Plan, entry: &EvidenceEntry) -> Result<()> {
    db.execute("INSERT INTO plan_evidence VALUES (?1,?2,?3,?4) ON CONFLICT(plan_id,version,entry_id) DO UPDATE SET body=excluded.body", params![plan.id,plan.version,entry.input.id,serde_json::to_string(entry)?])?;
    publish(
        db,
        Some(&plan.task_id),
        "plan.evidence_updated",
        &json!({"plan_id":plan.id,"version":plan.version,"entry":entry}),
    )?;
    Ok(())
}

pub(crate) fn seed(db: &Connection, plan: &Plan) -> Result<()> {
    let original: String = db.query_row(
        "SELECT request FROM tasks WHERE id=?1",
        [&plan.task_id],
        |r| r.get(0),
    )?;
    let mut inputs = vec![
        EvidenceInput {
            id: "owner_request".into(),
            kind: EvidenceKind::Requirement,
            state: EvidenceState::Pending,
            statement: original,
            source: format!("user_task:{}", plan.task_id),
            node_id: None,
            question: None,
        },
        EvidenceInput {
            id: "plan_objective".into(),
            kind: EvidenceKind::Task,
            state: EvidenceState::Pending,
            statement: format!("{}\nConstraints: {}", plan.objective, plan.constraints),
            source: "agent_plan; compare with original request and owner revisions".into(),
            node_id: None,
            question: None,
        },
    ];
    inputs.extend(plan.nodes.iter().map(|node| EvidenceInput {
        id: format!("milestone:{}", node.spec.id),
        kind: EvidenceKind::Task,
        state: EvidenceState::Pending,
        statement: format!(
            "{}\nExpected output: {}",
            node.spec.objective, node.spec.expected_output
        ),
        source: format!("plan:{}:v{}", plan.id, plan.version),
        node_id: Some(node.spec.id.clone()),
        question: None,
    }));
    for input in inputs {
        save_entry(
            db,
            plan,
            &EvidenceEntry {
                input,
                author_task_id: plan.task_id.clone(),
                updated_at: now(),
                audit_id: None,
                observations: vec![],
            },
        )?;
    }
    // Carry scoped candidate knowledge forward, but never reuse an old verdict as
    // proof for changed requirements or artifacts. Keep the old version retrievable.
    if plan.version > 1 {
        let previous: Option<u64> = db.query_row(
            "SELECT max(version) FROM plan_evidence WHERE plan_id=?1 AND version<?2",
            params![plan.id, plan.version],
            |r| r.get(0),
        )?;
        if let Some(previous) = previous {
            for mut entry in entries(db, &plan.id, previous)? {
                if entry.input.id.starts_with("note:")
                    && !entry.input.id.starts_with("note:brief:")
                    && entry
                        .input
                        .node_id
                        .as_ref()
                        .is_none_or(|id| plan.nodes.iter().any(|n| &n.spec.id == id))
                {
                    entry.input.state = if entry.input.question.is_some() {
                        EvidenceState::Doubtful
                    } else {
                        EvidenceState::Pending
                    };
                    entry.audit_id = None;
                    entry.observations.clear();
                    save_entry(db, plan, &entry)?;
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn capture_artifacts(
    db: &Connection,
    plan: &Plan,
    node: &str,
    worker: &str,
    result: Option<&serde_json::Value>,
) -> Result<()> {
    let Some(artifacts) = result
        .and_then(|r| r["artifacts"].as_str())
        .filter(|s| !s.trim().is_empty())
    else {
        return Ok(());
    };
    let input = EvidenceInput {
        id: format!("artifact:{node}"),
        kind: EvidenceKind::Artifact,
        state: EvidenceState::Pending,
        statement: artifacts.chars().take(6000).collect(),
        source: format!("Executor claim task:{worker}; inspect actual deliverables"),
        node_id: Some(node.into()),
        question: None,
    };
    save_entry(
        db,
        plan,
        &EvidenceEntry {
            input,
            author_task_id: worker.into(),
            updated_at: now(),
            audit_id: None,
            observations: vec![],
        },
    )
}

pub(crate) fn needs_final(db: &Connection, plan: &Plan) -> Result<bool> {
    if plan.nodes.len() >= 3 || plan.nodes.iter().any(|n| !n.spec.depends_on.is_empty()) {
        return Ok(true);
    }
    let explicit: bool = db.query_row("SELECT EXISTS(SELECT 1 FROM records WHERE kind='completion_audit_required' AND id=?1 AND data='true')",[&plan.id],|r|r.get(0))?;
    let long_worker: bool = db.query_row("SELECT EXISTS(SELECT 1 FROM events e JOIN plan_workers w ON w.task_id=e.task_id WHERE w.plan_id=?1 AND e.kind='tool_result' AND json_extract(e.data,'$.tool') NOT IN ('tool_search','skill_load','command_status','subagent_finish') GROUP BY e.task_id HAVING count(*)>=12)",[&plan.id],|r|r.get(0))?;
    Ok(explicit || long_worker)
}

pub(crate) fn requires_node(
    db: &Connection,
    plan: &Plan,
    node: &str,
    worker: &str,
) -> Result<bool> {
    let flagged: bool = db.query_row("SELECT EXISTS(SELECT 1 FROM records WHERE kind='decision_review_required' AND id=?1 AND data='true')", [worker], |r|r.get(0))?;
    if flagged {
        return Ok(true);
    }
    let substantial: bool = db.query_row("SELECT count(*)>=12 FROM events WHERE task_id=?1 AND kind='tool_result' AND json_extract(data,'$.tool') NOT IN ('tool_search','skill_load','command_status','subagent_finish')",[worker],|r|r.get(0))?;
    Ok(substantial && plan.nodes.len() > 1
        || plan
            .nodes
            .iter()
            .any(|n| n.spec.depends_on.iter().any(|d| d == node)))
}

pub(crate) fn queue(db: &Connection, plan: &Plan, node_id: &str) -> Result<()> {
    if entries(db, &plan.id, plan.version)?.is_empty() {
        seed(db, plan)?;
    }
    let exists: bool = db.query_row("SELECT EXISTS(SELECT 1 FROM completion_audits WHERE plan_id=?1 AND version=?2 AND node_id=?3)",params![plan.id,plan.version,node_id],|r|r.get(0))?;
    if !exists {
        write(
            db,
            &mut CompletionAudit {
                id: id(),
                plan_id: plan.id.clone(),
                version: plan.version,
                node_id: node_id.into(),
                status: "queued".into(),
                worker_id: None,
                report: None,
                created_at: now(),
                updated_at: now(),
            },
        )?;
    }
    Ok(())
}

pub(crate) fn final_status(db: &Connection, plan: &Plan) -> Result<Option<String>> {
    Ok(db
        .query_row(
            "SELECT status FROM completion_audits WHERE plan_id=?1 AND version=?2 AND node_id=''",
            params![plan.id, plan.version],
            |r| r.get(0),
        )
        .optional()?)
}

pub(crate) fn supersede(db: &Connection, plan: &Plan) -> Result<Vec<String>> {
    let ids = db.prepare("SELECT id FROM completion_audits WHERE plan_id=?1 AND version<>?2 AND status IN ('queued','running')")?.query_map(params![plan.id,plan.version],|r|r.get::<_,String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
    let mut workers = vec![];
    for id in ids {
        let mut audit = read(db, &id)?;
        workers.extend(audit.worker_id.clone());
        audit.status = "superseded".into();
        write(db, &mut audit)?;
    }
    Ok(workers)
}

fn current(db: &Connection, audit: &CompletionAudit) -> Result<Plan> {
    let plan = plans::read(db, &audit.plan_id)?;
    if plan.version != audit.version
        || plan.status != "running"
        || !matches!(audit.status.as_str(), "queued" | "running")
    {
        bail!("Completion audit was superseded or already settled");
    }
    Ok(plan)
}

impl Store {
    pub fn completion_observations(
        &self,
        worker: &str,
        report: &CompletionReport,
    ) -> Result<Vec<serde_json::Value>> {
        let db = self.db()?;
        let calls = report
            .checks
            .iter()
            .flat_map(|c| &c.call_ids)
            .collect::<BTreeSet<_>>();
        calls.into_iter().map(|call| {
            let body: String = db.query_row("SELECT data FROM events WHERE task_id=?1 AND kind='tool_result' AND json_extract(data,'$.call_id')=?2 ORDER BY seq DESC LIMIT 1",params![worker,call],|r|r.get(0))?;
            Ok(serde_json::from_str(&body)?)
        }).collect()
    }

    pub fn completion_audits(&self, plan: &str) -> Result<Vec<CompletionAudit>> {
        self.db()?
            .prepare("SELECT body FROM completion_audits WHERE plan_id=?1 ORDER BY rowid")?
            .query_map([plan], |r| r.get::<_, String>(0))?
            .map(|r| Ok(serde_json::from_str(&r?)?))
            .collect()
    }

    pub fn completion_audit_for_worker(&self, worker: &str) -> Result<Option<CompletionAudit>> {
        let db = self.db()?;
        let id: Option<String> = db
            .query_row(
                "SELECT id FROM completion_audits WHERE worker_id=?1",
                [worker],
                |r| r.get(0),
            )
            .optional()?;
        id.map(|id| read(&db, &id)).transpose()
    }

    pub fn check_completion_auditor(&self, worker: &str) -> Result<()> {
        if let Some(audit) = self.completion_audit_for_worker(worker)? {
            current(&*self.db()?, &audit)?;
        }
        Ok(())
    }

    pub fn plan_evidence(&self, plan: &str, version: u64) -> Result<Vec<EvidenceEntry>> {
        entries(&*self.db()?, plan, version)
    }

    /// Only candidates enter through the agent tool. Verified is reserved for a
    /// successful independent observation; source labels never grant authority.
    pub fn record_plan_evidence(
        &self,
        plan_id: &str,
        version: u64,
        author: &str,
        mut input: EvidenceInput,
    ) -> Result<EvidenceEntry> {
        if !input.id.starts_with("note:")
            || input.id.len() > 100
            || input.statement.trim().is_empty()
            || input.statement.len() > 6000
            || input.source.trim().is_empty()
            || input.source.len() > 2000
            || input
                .question
                .as_ref()
                .is_some_and(|q| q.trim().is_empty() || q.len() > 2000)
        {
            bail!(
                "Evidence requires a note: ID, a bounded statement and source; questions must be nonempty"
            );
        }
        if matches!(
            input.state,
            EvidenceState::Verified | EvidenceState::Refuted
        ) {
            bail!(
                "Executors record pending/doubtful candidates; only an independent audit can verify them"
            );
        }
        if input.question.is_some() {
            input.state = EvidenceState::Doubtful;
        }
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let plan = plans::read(&tx, plan_id)?;
        if plan.version != version || !matches!(plan.status.as_str(), "running" | "blocked") {
            bail!("Read the current active plan version before recording evidence");
        }
        if input
            .node_id
            .as_ref()
            .is_some_and(|id| !plan.nodes.iter().any(|n| &n.spec.id == id))
        {
            bail!("Unknown evidence milestone");
        }
        if final_status(&tx, &plan)?.as_deref() == Some("passed") {
            bail!("The result was already audited; revise the plan before adding new evidence");
        }
        if let Some(node) = &input.node_id {
            let verified: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM completion_audits WHERE plan_id=?1 AND version=?2 AND node_id=?3 AND status='passed')",params![plan_id,version,node],|r|r.get(0))?;
            if verified {
                bail!("This milestone was already audited; revise before changing its evidence");
            }
        }
        let existing = entries(&tx, plan_id, version)?;
        if existing.iter().any(|e| e.input.id == input.id) {
            bail!(
                "Evidence is append-only within a plan version; use a new note: ID for corrections"
            );
        }
        if existing.len() >= 200 {
            bail!(
                "Plan evidence catalog is full; retain detailed material in referenced artifacts"
            );
        }
        let entry = EvidenceEntry {
            input,
            author_task_id: author.into(),
            updated_at: now(),
            audit_id: None,
            observations: vec![],
        };
        save_entry(&tx, &plan, &entry)?;
        tx.commit()?;
        Ok(entry)
    }

    pub fn attach_completion_auditor(&self, id: &str, worker: &str) -> Result<()> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let mut audit = read(&tx, id)?;
        let plan = current(&tx, &audit)?;
        if audit.worker_id.is_some() {
            bail!("Audit already assigned");
        }
        let valid:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM subagents s JOIN tasks t ON t.id=s.id WHERE s.id=?1 AND s.parent_task_id=?2 AND s.project_id IS ?3 AND t.status='queued' AND s.attempts=0 AND NOT EXISTS(SELECT 1 FROM plan_workers WHERE task_id=s.id) AND NOT EXISTS(SELECT 1 FROM plan_review_workers WHERE task_id=s.id))",params![worker,plan.task_id,plan.project_id],|r|r.get(0))?;
        if !valid {
            bail!("Auditor must be a fresh queued sibling in the plan's exact scope");
        }
        audit.worker_id = Some(worker.into());
        audit.status = "running".into();
        write(&tx, &mut audit)?;
        tx.commit()?;
        Ok(())
    }

    pub fn validate_completion_report(
        &self,
        worker: &str,
        report: &CompletionReport,
    ) -> Result<()> {
        let audit = self
            .completion_audit_for_worker(worker)?
            .context("Not a completion auditor")?;
        let db = self.db()?;
        validate_report(&db, &audit, worker, report)
    }

    pub fn finish_completion_audit(&self, worker: &str, report: CompletionReport) -> Result<()> {
        let audit = self
            .completion_audit_for_worker(worker)?
            .context("Not a completion auditor")?;
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let mut audit = read(&tx, &audit.id)?;
        validate_report(&tx, &audit, worker, &report)?;
        let mut plan = plans::read(&tx, &audit.plan_id)?;
        let pass = report.verdict == "verified";
        for check in &report.checks {
            let mut entry = entries(&tx, &plan.id, plan.version)?
                .into_iter()
                .find(|e| e.input.id == check.entry_id)
                .context("Evidence entry disappeared")?;
            entry.input.state = check.state.clone();
            if check.state == EvidenceState::Verified {
                entry.input.question = None;
            }
            entry.audit_id = Some(audit.id.clone());
            entry.observations = check
                .call_ids
                .iter()
                .map(|id| format!("task:{worker}/call:{id}"))
                .collect();
            entry.updated_at = now();
            save_entry(&tx, &plan, &entry)?;
        }
        audit.status = if pass { "passed" } else { "blocked" }.into();
        audit.report = Some(report);
        write(&tx, &mut audit)?;
        if !audit.node_id.is_empty() {
            let node = plan
                .nodes
                .iter_mut()
                .find(|n| n.spec.id == audit.node_id && n.status == "verifying")
                .context("Milestone is no longer awaiting verification")?;
            node.status = if pass { "completed" } else { "blocked" }.into();
            // Preserve the executor result and the independent audit separately.
            plan.updated_at = now();
            plans::write(&tx, &plan)?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn block_completion_audit(&self, id: &str, reason: &str) -> Result<()> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let mut audit = read(&tx, id)?;
        let mut plan = current(&tx, &audit)?;
        audit.status = "blocked".into();
        audit.report = Some(CompletionReport {
            verdict: "incomplete".into(),
            summary: reason.into(),
            checks: vec![],
        });
        if let Some(node) = plan
            .nodes
            .iter_mut()
            .find(|n| n.spec.id == audit.node_id && n.status == "verifying")
        {
            node.status = "blocked".into();
        }
        plans::write(&tx, &plan)?;
        write(&tx, &mut audit)?;
        tx.commit()?;
        Ok(())
    }
}

fn validate_report(
    db: &Connection,
    audit: &CompletionAudit,
    worker: &str,
    report: &CompletionReport,
) -> Result<()> {
    let plan = current(db, audit)?;
    if !matches!(
        report.verdict.as_str(),
        "verified" | "incomplete" | "needs_user"
    ) || report.summary.trim().is_empty()
        || report.summary.len() > 6000
        || report.checks.len() > 200
    {
        bail!("Invalid completion report");
    }
    let evidence = entries(db, &plan.id, plan.version)?;
    let relevant = evidence
        .iter()
        .filter(|e| {
            audit.node_id.is_empty() || e.input.node_id.as_deref() == Some(audit.node_id.as_str())
        })
        .collect::<Vec<_>>();
    let mut ids = BTreeSet::new();
    let mut observed_deliverable = false;
    for check in &report.checks {
        if !ids.insert(&check.entry_id)
            || !relevant.iter().any(|e| e.input.id == check.entry_id)
            || check.detail.trim().is_empty()
            || check.detail.len() > 3000
            || check.call_ids.len() > 12
        {
            bail!("Checks must reference unique assigned entries with bounded findings");
        }
        if check.state == EvidenceState::Refuted
            && relevant.iter().any(|e| {
                e.input.id == check.entry_id
                    && (e.input.kind != EvidenceKind::Fact || e.input.question.is_some())
            })
        {
            bail!(
                "Only factual candidates without an unresolved owner question may be refuted; revise requirements instead"
            );
        }
        if matches!(
            check.state,
            EvidenceState::Verified | EvidenceState::Refuted
        ) && check.call_ids.is_empty()
        {
            bail!("Verified checks require observed evidence");
        }
        for call in &check.call_ids {
            let body: String = db.query_row("SELECT data FROM events WHERE task_id=?1 AND kind='tool_result' AND json_extract(data,'$.call_id')=?2 ORDER BY seq DESC LIMIT 1",params![worker,call],|r|r.get(0)).optional()?.context("Observation call not found for this auditor")?;
            let observation: serde_json::Value = serde_json::from_str(&body)?;
            let tool = observation["tool"].as_str().unwrap_or("");
            let result = &observation["result"];
            if observation["is_error"] != false
                || result["executed"] == false
                || result["denied"] == true
                || result["code"] == "tool_schema_loaded"
                || !result["tool_schema_loaded"].is_null()
                || !matches!(
                    tool,
                    "file_read"
                        | "document_read"
                        | "design_read"
                        | "library_read"
                        | "web_read"
                        | "memory_search"
                        | "memory_context"
                        | "schedule_manage"
                        | "project_manage"
                        | "files_list"
                        | "files_search"
                        | "files_glob"
                        | "ask_user"
                )
            {
                bail!("Evidence must reference this auditor's successful observation call: {call}");
            }
            observed_deliverable |= tool != "ask_user";
        }
    }
    if report.verdict == "verified" {
        if relevant.is_empty()
            || relevant.iter().any(|e| {
                !report.checks.iter().any(|c| {
                    c.entry_id == e.input.id
                        && (c.state == EvidenceState::Verified
                            || c.state == EvidenceState::Refuted
                                && e.input.kind == EvidenceKind::Fact
                                && e.input.question.is_none())
                })
            })
        {
            bail!(
                "Verification must cover every assigned entry, including doubts and original requirements"
            );
        }
        let unanswered: bool = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM questions WHERE task_id=?1 AND status='pending')",
            [worker],
            |r| r.get(0),
        )?;
        if unanswered {
            bail!("Unanswered owner questions prevent verification");
        }
        // An answer confirms intent, never execution. At least one actual artifact
        // observation must back a pass, even when the owner clarified requirements.
        if !observed_deliverable {
            bail!("Read the actual deliverable before passing completion");
        }
    }
    Ok(())
}
