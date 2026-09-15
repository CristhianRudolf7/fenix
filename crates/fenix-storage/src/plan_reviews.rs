//! Durable, version-scoped reviews. Reviewers advise; they never approve user intent.
use crate::{Plan, Store, events::publish, id, now, plans};
use anyhow::{Context, Result, bail};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PlanBrief {
    /// Each entry includes its source (owner message, confirmed preference, or document).
    pub requirements: Vec<String>,
    pub observations: Vec<String>,
    pub assumptions: Vec<String>,
    pub open_questions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecialistRequest {
    pub specialty: String,
    pub question: String,
    #[serde(default)]
    pub node_ids: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewFinding {
    pub id: String,
    pub category: String,
    pub severity: String,
    pub description: String,
    pub evidence: String,
    #[serde(default)]
    pub node_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewReport {
    pub verdict: String,
    pub findings: Vec<ReviewFinding>,
    #[serde(default)]
    pub specialists: Vec<SpecialistRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewResolution {
    /// Assignment role followed by '/' and the finding ID.
    pub finding_id: String,
    pub disposition: String,
    pub rationale: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewAssignment {
    pub role: String,
    pub request: SpecialistRequest,
    pub worker_id: Option<String>,
    pub report: Option<ReviewReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanReview {
    pub id: String,
    pub plan_id: String,
    pub version: u64,
    /// Initial review = 0; one response/revision check = 1.
    pub round: u8,
    pub status: String,
    /// Empty means the entire plan; otherwise these nodes and their dependants.
    pub node_ids: Vec<String>,
    pub reason: String,
    pub brief: PlanBrief,
    pub assignments: Vec<ReviewAssignment>,
    pub coordinator_id: Option<String>,
    pub resolutions: Vec<ReviewResolution>,
    pub previous_review_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn read(db: &Connection, id: &str) -> Result<PlanReview> {
    let body: String = db
        .query_row("SELECT body FROM plan_reviews WHERE id=?1", [id], |r| {
            r.get(0)
        })
        .optional()?
        .context("Plan review not found")?;
    Ok(serde_json::from_str(&body)?)
}

fn write(db: &Connection, review: &mut PlanReview) -> Result<()> {
    review.updated_at = now();
    db.execute("INSERT INTO plan_reviews VALUES (?1,?2,?3,?4,?5) ON CONFLICT(id) DO UPDATE SET status=excluded.status,body=excluded.body", params![review.id,review.plan_id,review.version,review.status,serde_json::to_string(review)?])?;
    let plan = plans::read(db, &review.plan_id)?;
    publish(
        db,
        Some(&plan.task_id),
        "plan.review_updated",
        &json!({"plan_id":review.plan_id,"plan_version":review.version,"review_id":review.id,"status":review.status,"round":review.round,"reason":review.reason}),
    )?;
    Ok(())
}

fn bounded(text: &str, max: usize) -> bool {
    !text.trim().is_empty() && text.len() <= max
}

impl PlanBrief {
    pub fn validate(&self) -> Result<()> {
        for list in [
            &self.requirements,
            &self.observations,
            &self.assumptions,
            &self.open_questions,
        ] {
            if list.len() > 20 || list.iter().any(|s| !bounded(s, 1500)) {
                bail!("Brief sections accept up to 20 nonempty entries of 1500 bytes");
            }
        }
        if serde_json::to_vec(self)?.len() > 12000 {
            bail!("Plan brief exceeds 12000 bytes");
        }
        Ok(())
    }
}

fn validate_specialist(request: &SpecialistRequest, plan: &Plan) -> Result<()> {
    if !bounded(&request.specialty, 100)
        || !bounded(&request.question, 2000)
        || request.skills.len() > 5
        || request.skills.iter().any(|s| !bounded(s, 120))
    {
        bail!("Specialists need a bounded specialty, precise question and up to five skills");
    }
    validate_nodes(&request.node_ids, plan)
}

fn validate_nodes(nodes: &[String], plan: &Plan) -> Result<()> {
    if nodes.len() > 32
        || nodes
            .iter()
            .any(|id| !plan.nodes.iter().any(|n| &n.spec.id == id))
    {
        bail!("Review scope must contain existing milestone IDs");
    }
    Ok(())
}

impl Store {
    pub fn plan_review(&self, id: &str) -> Result<PlanReview> {
        read(&*self.db()?, id)
    }

    /// Provider-reported usage only. Missing telemetry stays unknown; token
    /// totals do not imply a monetary price or successful task execution.
    pub fn plan_review_metrics(&self, plan: &str) -> Result<Value> {
        let reviews = self.plan_reviews(plan)?;
        let db = self.db()?;
        let mut output = vec![];
        for review in reviews {
            let mut calls = 0u64;
            let mut input = 0u64;
            let mut generated = 0u64;
            let mut total = 0u64;
            let mut missing = 0u64;
            let tasks = review
                .assignments
                .iter()
                .filter_map(|a| a.worker_id.as_ref())
                .chain(review.coordinator_id.iter());
            for task in tasks {
                let rows = db.prepare("SELECT json_extract(data,'$.usage.input_tokens'),json_extract(data,'$.usage.output_tokens'),json_extract(data,'$.usage.total_tokens') FROM events WHERE task_id=?1 AND kind IN ('model_response','context_compacted')")?
                    .query_map([task],|r|Ok((r.get::<_,Option<u64>>(0)?,r.get::<_,Option<u64>>(1)?,r.get::<_,Option<u64>>(2)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
                for (i, o, t) in rows {
                    calls += 1;
                    input += i.unwrap_or(0);
                    generated += o.unwrap_or(0);
                    total += t.or_else(|| i.zip(o).map(|(i, o)| i + o)).unwrap_or(0);
                    if i.is_none() || o.is_none() {
                        missing += 1;
                    }
                }
            }
            let start = chrono::DateTime::parse_from_rfc3339(&review.created_at)?;
            let end = if matches!(
                review.status.as_str(),
                "passed" | "blocked" | "superseded" | "interrupted"
            ) {
                chrono::DateTime::parse_from_rfc3339(&review.updated_at)?
            } else {
                chrono::Utc::now().fixed_offset()
            };
            output.push(json!({"review_id":review.id,"model_responses":calls,"reported_input_tokens":input,"reported_output_tokens":generated,"reported_total_tokens":total,"responses_with_missing_usage":missing,"elapsed_ms_including_waits":(end-start).num_milliseconds().max(0),"includes":"Reviewer and resolution turns; excludes milestone execution and unreported failed API calls"}));
        }
        Ok(json!(output))
    }

    pub fn plan_reviews(&self, plan: &str) -> Result<Vec<PlanReview>> {
        self.db()?
            .prepare("SELECT body FROM plan_reviews WHERE plan_id=?1 ORDER BY rowid")?
            .query_map([plan], |r| r.get::<_, String>(0))?
            .map(|r| Ok(serde_json::from_str(&r?)?))
            .collect()
    }

    pub fn review_for_worker(&self, task: &str) -> Result<Option<PlanReview>> {
        let db = self.db()?;
        let id: Option<String> = db
            .query_row(
                "SELECT review_id FROM plan_review_workers WHERE task_id=?1",
                [task],
                |r| r.get(0),
            )
            .optional()?;
        id.map(|id| read(&db, &id)).transpose()
    }

    pub fn start_plan_review(&self, mut review: PlanReview) -> Result<PlanReview> {
        review.brief.validate()?;
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let plan = plans::read(&tx, &review.plan_id)?;
        if plan.version != review.version
            || !matches!(plan.status.as_str(), "running" | "blocked")
            || review.round > 1
            || !bounded(&review.reason, 4000)
        {
            bail!("Review does not match an active plan version");
        }
        validate_nodes(&review.node_ids, &plan)?;
        if review.assignments.is_empty()
            || review.assignments.len() > 3
            || review
                .assignments
                .iter()
                .filter(|a| a.role == "principal")
                .count()
                != 1
        {
            bail!("Reviews require one principal and at most two specialists");
        }
        let mut roles = BTreeSet::new();
        for a in &review.assignments {
            if !bounded(&a.role, 100)
                || !roles.insert(&a.role)
                || a.worker_id.is_some()
                || a.report.is_some()
            {
                bail!("New review roles must be unique and unassigned");
            }
            validate_specialist(&a.request, &plan)?;
        }
        let mut existing = tx
            .prepare("SELECT body FROM plan_reviews WHERE plan_id=?1 AND version=?2")?
            .query_map(params![plan.id, plan.version], |r| r.get::<_, String>(0))?
            .map(|r| Ok(serde_json::from_str::<PlanReview>(&r?)?))
            .collect::<Result<Vec<_>>>()?;
        if let Some(old) = existing
            .iter_mut()
            .find(|r| r.reason == review.reason && r.node_ids == review.node_ids)
        {
            return Ok(old.clone());
        }
        if existing.iter().any(|r| {
            matches!(
                r.status.as_str(),
                "queued" | "running" | "needs_resolution" | "resolving" | "blocked"
            )
        }) {
            bail!("Resolve the current review before requesting another for this plan version");
        }
        review.id = id();
        review.created_at = now();
        review.status = "queued".into();
        write(&tx, &mut review)?;
        tx.commit()?;
        Ok(review)
    }

    pub fn attach_review_worker(&self, review_id: &str, role: &str, worker: &str) -> Result<()> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let mut review = read(&tx, review_id)?;
        let plan = plans::read(&tx, &review.plan_id)?;
        if plan.version != review.version
            || !matches!(plan.status.as_str(), "running" | "blocked")
            || !matches!(review.status.as_str(), "queued" | "running")
        {
            bail!("Review was superseded");
        }
        let valid: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM subagents s JOIN tasks t ON t.id=s.id WHERE s.id=?1 AND s.parent_task_id=?2 AND s.project_id IS ?3 AND t.status='queued')", params![worker,plan.task_id,plan.project_id], |r|r.get(0))?;
        if !valid {
            bail!("Reviewer must be a queued child of the plan owner in the same project");
        }
        let a = review
            .assignments
            .iter_mut()
            .find(|a| a.role == role)
            .context("Unknown review role")?;
        if a.worker_id.is_some() {
            bail!("Review role was already dispatched");
        }
        a.worker_id = Some(worker.into());
        tx.execute(
            "INSERT INTO plan_review_workers VALUES (?1,?2)",
            params![worker, review_id],
        )?;
        review.status = "running".into();
        write(&tx, &mut review)?;
        tx.commit()?;
        Ok(())
    }

    pub fn check_review_worker(&self, task: &str) -> Result<()> {
        if let Some(review) = self.review_for_worker(task)? {
            let plan = self.plan(&review.plan_id)?;
            if review.version != plan.version
                || !matches!(plan.status.as_str(), "running" | "blocked")
                || !matches!(review.status.as_str(), "queued" | "running")
            {
                bail!("Reviewer belongs to an obsolete or closed review");
            }
        }
        Ok(())
    }

    pub fn validate_review_report(&self, worker: &str, report: &ReviewReport) -> Result<()> {
        let review = self
            .review_for_worker(worker)?
            .context("Not a plan reviewer")?;
        self.check_review_worker(worker)?;
        let plan = self.plan(&review.plan_id)?;
        let role = &review
            .assignments
            .iter()
            .find(|a| a.worker_id.as_deref() == Some(worker))
            .context("Unknown reviewer")?
            .role;
        if !matches!(
            report.verdict.as_str(),
            "pass" | "changes_requested" | "needs_user"
        ) || report.findings.len() > 12
            || report.specialists.len() > 2
            || (role != "principal" && !report.specialists.is_empty())
            || (review.round == 1 && !report.specialists.is_empty())
        {
            bail!("Invalid review verdict, finding count or specialist request");
        }
        let mut ids = BTreeSet::new();
        for f in &report.findings {
            if !bounded(&f.id, 80)
                || f.id.contains('/')
                || !ids.insert(&f.id)
                || !bounded(&f.description, 2000)
                || !bounded(&f.evidence, 2000)
                || !matches!(
                    f.category.as_str(),
                    "intent" | "scope" | "dependency" | "feasibility" | "evidence"
                )
                || !matches!(f.severity.as_str(), "blocking" | "advisory")
            {
                bail!(
                    "Findings require unique IDs, concrete evidence and a valid category/severity"
                );
            }
            validate_nodes(&f.node_ids, &plan)?;
        }
        let blocking = report.findings.iter().any(|f| f.severity == "blocking");
        if (report.verdict == "pass" && blocking) || (report.verdict != "pass" && !blocking) {
            bail!("Verdict must agree with blocking findings");
        }
        for s in &report.specialists {
            validate_specialist(s, &plan)?;
        }
        if serde_json::to_vec(report)?.len() > 16000 {
            bail!("Review report exceeds 16000 bytes");
        }
        Ok(())
    }

    pub fn record_review_report(&self, worker: &str, report: ReviewReport) -> Result<()> {
        self.validate_review_report(worker, &report)?;
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let review_id: String = tx.query_row(
            "SELECT review_id FROM plan_review_workers WHERE task_id=?1",
            [worker],
            |r| r.get(0),
        )?;
        let mut review = read(&tx, &review_id)?;
        let plan = plans::read(&tx, &review.plan_id)?;
        if review.version != plan.version
            || !matches!(plan.status.as_str(), "running" | "blocked")
            || !matches!(review.status.as_str(), "queued" | "running")
        {
            bail!("Review changed before result application");
        }
        let a = review
            .assignments
            .iter_mut()
            .find(|a| a.worker_id.as_deref() == Some(worker))
            .context("Unknown reviewer")?;
        if a.report.is_some() {
            return Ok(());
        }
        a.report = Some(report.clone());
        for request in report.specialists {
            if review.assignments.len() >= 3 {
                break;
            }
            if review.assignments.iter().any(|a| {
                a.request.specialty == request.specialty && a.request.question == request.question
            }) {
                continue;
            }
            review.assignments.push(ReviewAssignment {
                role: format!("specialist_{}", review.assignments.len()),
                request,
                worker_id: None,
                report: None,
            });
        }
        if review.assignments.iter().all(|a| a.report.is_some()) {
            let all_pass = review
                .assignments
                .iter()
                .all(|a| a.report.as_ref().is_some_and(|r| r.verdict == "pass"));
            review.status = if all_pass {
                "passed"
            } else if review.round == 1 {
                "blocked"
            } else {
                "needs_resolution"
            }
            .into();
        }
        write(&tx, &mut review)?;
        tx.commit()?;
        Ok(())
    }

    pub fn set_review_coordinator(&self, review_id: &str, task: &str) -> Result<()> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let mut review = read(&tx, review_id)?;
        if review.status != "needs_resolution" || review.coordinator_id.is_some() {
            bail!("Review already has a resolution turn");
        }
        review.coordinator_id = Some(task.into());
        review.status = "resolving".into();
        write(&tx, &mut review)?;
        tx.commit()?;
        Ok(())
    }

    pub fn resolve_plan_review(
        &self,
        id: &str,
        source: &str,
        resolutions: Vec<ReviewResolution>,
    ) -> Result<PlanReview> {
        self.update_review_resolution(id, source, resolutions, true)
    }

    pub fn record_review_revision_response(
        &self,
        id: &str,
        source: &str,
        resolutions: Vec<ReviewResolution>,
    ) -> Result<PlanReview> {
        self.update_review_resolution(id, source, resolutions, false)
    }

    fn update_review_resolution(
        &self,
        id: &str,
        source: &str,
        resolutions: Vec<ReviewResolution>,
        close: bool,
    ) -> Result<PlanReview> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let mut review = read(&tx, id)?;
        let plan = plans::read(&tx, &review.plan_id)?;
        if review.status != "resolving"
            || review.coordinator_id.as_deref() != Some(source)
            || plan.version != review.version
            || !matches!(plan.status.as_str(), "running" | "blocked")
        {
            bail!("Only the current resolution turn may settle this review");
        }
        let findings = review
            .assignments
            .iter()
            .flat_map(|a| {
                a.report.iter().flat_map(move |r| {
                    r.findings
                        .iter()
                        .map(move |f| (format!("{}/{}", a.role, f.id), f))
                })
            })
            .collect::<Vec<_>>();
        let mut ids = BTreeSet::new();
        for r in &resolutions {
            if !ids.insert(&r.finding_id)
                || !findings.iter().any(|(id, _)| id == &r.finding_id)
                || !matches!(
                    r.disposition.as_str(),
                    "addressed" | "dismissed" | "needs_user"
                )
                || !bounded(&r.rationale, 2000)
                || !bounded(&r.evidence, 2000)
            {
                bail!(
                    "Each resolution requires an existing finding, rationale and concrete evidence"
                );
            }
        }
        if findings
            .iter()
            .filter(|(_, f)| f.severity == "blocking")
            .any(|(id, _)| !resolutions.iter().any(|r| &r.finding_id == id))
        {
            bail!("Resolve every blocking finding explicitly; do not vote");
        }
        if close {
            review.status = if resolutions.iter().any(|r| r.disposition == "needs_user") {
                "blocked"
            } else {
                "passed"
            }
            .into();
        } else if resolutions.iter().any(|r| r.disposition == "needs_user") {
            bail!(
                "Do not revise past an unanswered owner decision; ask or leave the review blocked"
            );
        }
        review.resolutions = resolutions;
        write(&tx, &mut review)?;
        tx.commit()?;
        Ok(review)
    }

    pub fn close_plan_review(&self, id: &str, status: &str, reason: &str) -> Result<()> {
        if !matches!(status, "blocked" | "superseded") {
            bail!("Invalid review closure");
        }
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let mut review = read(&tx, id)?;
        if matches!(
            review.status.as_str(),
            "passed" | "superseded" | "interrupted"
        ) {
            return Ok(());
        }
        review.status = status.into();
        review.reason = reason.into();
        write(&tx, &mut review)?;
        tx.commit()?;
        Ok(())
    }

    pub fn review_blocks_node(&self, plan: &Plan, node: &str) -> Result<bool> {
        blocks(&*self.db()?, plan, node)
    }
}

pub(crate) fn blocks(db: &Connection, plan: &Plan, node: &str) -> Result<bool> {
    let reviews = db.prepare("SELECT body FROM plan_reviews WHERE plan_id=?1 AND version=?2 AND status NOT IN ('passed','superseded')")?
        .query_map(params![plan.id,plan.version], |r|r.get::<_,String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
    for body in reviews {
        let r: PlanReview = serde_json::from_str(&body)?;
        if r.node_ids.is_empty() {
            return Ok(true);
        }
        let mut affected = r.node_ids.into_iter().collect::<BTreeSet<_>>();
        loop {
            let before = affected.len();
            for n in &plan.nodes {
                if n.spec.depends_on.iter().any(|d| affected.contains(d)) {
                    affected.insert(n.spec.id.clone());
                }
            }
            if before == affected.len() {
                break;
            }
        }
        if affected.contains(node) {
            return Ok(true);
        }
    }
    Ok(false)
}
