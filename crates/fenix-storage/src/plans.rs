//! Versioned task graphs. Revisions invalidate only changed nodes and dependants.
use crate::{Store, events::publish, id, now};
use anyhow::{Context, Result, bail};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanNodeSpec {
    pub id: String,
    pub objective: String,
    pub expected_output: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
}

pub struct PlanRevision<'a> {
    pub objective: &'a str,
    pub constraints: &'a str,
    pub nodes: Vec<PlanNodeSpec>,
    pub reason: &'a str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanNode {
    #[serde(flatten)]
    pub spec: PlanNodeSpec,
    pub generation: u64,
    pub status: String,
    pub worker_id: Option<String>,
    pub result: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub task_id: String,
    pub project_id: Option<String>,
    pub version: u64,
    pub status: String,
    pub objective: String,
    pub constraints: String,
    pub nodes: Vec<PlanNode>,
    pub updated_at: String,
}

pub(crate) fn read(db: &Connection, id: &str) -> Result<Plan> {
    let text: String = db
        .query_row("SELECT body FROM plans WHERE id=?1", [id], |r| r.get(0))
        .optional()?
        .context("Plan not found")?;
    Ok(serde_json::from_str(&text)?)
}

fn write(db: &Connection, plan: &Plan) -> Result<()> {
    db.execute("INSERT INTO plans(id,task_id,version,status,body,updated_at) VALUES (?1,?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET version=excluded.version,status=excluded.status,body=excluded.body,updated_at=excluded.updated_at",params![plan.id,plan.task_id,plan.version,plan.status,serde_json::to_string(plan)?,plan.updated_at])?;
    Ok(())
}

fn validate(objective: &str, constraints: &str, nodes: &[PlanNodeSpec]) -> Result<()> {
    if objective.trim().is_empty()
        || objective.len() > 8000
        || constraints.len() > 8000
        || nodes.is_empty()
        || nodes.len() > 32
    {
        bail!("Plans require an objective, up to 8000 bytes of constraints and 1–32 milestones");
    }
    let mut ids = BTreeSet::new();
    for n in nodes {
        if n.id.is_empty()
            || n.id.len() > 80
            || !n
                .id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-'))
            || !ids.insert(n.id.clone())
        {
            bail!("Milestone IDs must be unique, short ASCII identifiers");
        }
        if n.objective.trim().is_empty()
            || n.expected_output.trim().is_empty()
            || n.objective.len() > 4000
            || n.expected_output.len() > 4000
            || n.skills.len() > 20
            || n.skills.iter().any(|s| s.len() > 120)
            || n.depends_on.len() > 32
        {
            bail!("Each milestone requires a bounded objective and expected output");
        }
    }
    if nodes
        .iter()
        .any(|n| n.depends_on.iter().any(|d| !ids.contains(d)))
    {
        bail!("Unknown milestone dependency");
    }
    let mut ready = BTreeSet::new();
    loop {
        let before = ready.len();
        for n in nodes {
            if n.depends_on.iter().all(|d| ready.contains(d)) {
                ready.insert(n.id.clone());
            }
        }
        if ready.len() == nodes.len() {
            return Ok(());
        }
        if ready.len() == before {
            bail!("Milestone dependencies must not contain cycles");
        }
    }
}

impl Store {
    pub fn plan(&self, id: &str) -> Result<Plan> {
        read(&*self.db()?, id)
    }

    pub fn plans(&self) -> Result<Vec<Plan>> {
        self.db()?.prepare("SELECT body FROM plans ORDER BY CASE WHEN status IN ('running','revising','paused','blocked') THEN 0 ELSE 1 END,updated_at DESC LIMIT 200")?
            .query_map([],|r|r.get::<_,String>(0))?.map(|row|Ok(serde_json::from_str(&row?)?)).collect()
    }

    pub fn plan_for_task(&self, task: &str) -> Result<Option<Plan>> {
        let id:Option<String>=self.db()?.query_row("SELECT id FROM plans WHERE task_id=?1 UNION SELECT plan_id FROM plan_workers WHERE task_id=?1 UNION SELECT r.plan_id FROM plan_review_workers w JOIN plan_reviews r ON r.id=w.review_id WHERE w.task_id=?1 LIMIT 1",[task],|r|r.get(0)).optional()?;
        id.map(|id| self.plan(&id)).transpose()
    }

    pub fn create_plan(
        &self,
        task: &str,
        objective: &str,
        constraints: &str,
        nodes: Vec<PlanNodeSpec>,
    ) -> Result<Plan> {
        validate(objective, constraints, &nodes)?;
        let owner = self.task(task)?;
        let binding = self.task_binding(task)?;
        if !binding.bound
            || owner.origin == "subagent"
            || owner.origin == "subagent_report"
            || binding.mode == "clarification"
            || binding.mode == "coordination"
        {
            bail!("A plan requires one resolved scope and a master task");
        }
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let existing: Option<String> = tx
            .query_row("SELECT id FROM plans WHERE task_id=?1", [task], |r| {
                r.get(0)
            })
            .optional()?;
        if let Some(id) = existing {
            return read(&tx, &id);
        }
        let plan = Plan {
            id: id(),
            task_id: task.into(),
            project_id: binding.project_id,
            version: 1,
            status: "running".into(),
            objective: objective.into(),
            constraints: constraints.into(),
            nodes: nodes
                .into_iter()
                .map(|spec| PlanNode {
                    spec,
                    generation: 1,
                    status: "pending".into(),
                    worker_id: None,
                    result: None,
                })
                .collect(),
            updated_at: now(),
        };
        write(&tx, &plan)?;
        tx.execute(
            "INSERT INTO plan_revisions VALUES (?1,1,?2,?3,'Initial plan',?4)",
            params![plan.id, serde_json::to_string(&plan)?, task, now()],
        )?;
        publish(&tx, Some(task), "plan.created", &json!({"plan":plan}))?;
        tx.commit()?;
        Ok(plan)
    }

    /// Atomically invalidate old worker generations before requesting cancellation.
    /// The plan remains revising until runtime effect barriers have settled.
    pub fn revise_plan(
        &self,
        plan_id: &str,
        expected_version: u64,
        source: &str,
        revision: PlanRevision<'_>,
    ) -> Result<(Plan, Vec<String>)> {
        let PlanRevision {
            objective,
            constraints,
            nodes: specs,
            reason,
        } = revision;
        validate(objective, constraints, &specs)?;
        if reason.trim().is_empty() || reason.len() > 8000 {
            bail!("A bounded revision reason is required");
        }
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let mut plan = read(&tx, plan_id)?;
        if plan.version != expected_version || plan.status == "revising" {
            bail!("Plan changed; read the current version before revising");
        }
        let global_changed = plan.objective != objective || plan.constraints != constraints;
        let mut changed = BTreeSet::new();
        for old in &plan.nodes {
            if old.status != "completed" && global_changed || !specs.iter().any(|s| s == &old.spec)
            {
                changed.insert(old.spec.id.clone());
            }
        }
        loop {
            let count = changed.len();
            for spec in &specs {
                if spec.depends_on.iter().any(|d| changed.contains(d)) {
                    changed.insert(spec.id.clone());
                }
            }
            if count == changed.len() {
                break;
            }
        }
        let cancelled = plan
            .nodes
            .iter()
            .filter(|n| changed.contains(&n.spec.id) && n.status == "running")
            .filter_map(|n| n.worker_id.clone())
            .collect::<Vec<_>>();
        let generation = plan.version + 1;
        let nodes = specs
            .into_iter()
            .map(|spec| {
                match plan
                    .nodes
                    .iter()
                    .find(|n| n.spec.id == spec.id && !changed.contains(&spec.id))
                {
                    Some(old) => old.clone(),
                    None => {
                        let previous = plan.nodes.iter().find(|n| n.spec.id == spec.id).map(
                            |n| json!({"previous_worker":n.worker_id,"previous_result":n.result}),
                        );
                        PlanNode {
                            spec,
                            generation,
                            status: "pending".into(),
                            worker_id: None,
                            result: previous,
                        }
                    }
                }
            })
            .collect();
        plan.version = generation;
        plan.status = "revising".into();
        plan.objective = objective.into();
        plan.constraints = constraints.into();
        plan.nodes = nodes;
        plan.updated_at = now();
        write(&tx, &plan)?;
        tx.execute(
            "INSERT INTO plan_revisions VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                plan.id,
                plan.version,
                serde_json::to_string(&plan)?,
                source,
                reason,
                now()
            ],
        )?;
        publish(
            &tx,
            Some(&plan.task_id),
            "plan.revising",
            &json!({"plan":plan,"source_task_id":source,"cancelled_workers":cancelled,"reason":reason}),
        )?;
        tx.commit()?;
        Ok((plan, cancelled))
    }

    pub fn activate_plan_revision(&self, id: &str, version: u64) -> Result<Plan> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let mut plan = read(&tx, id)?;
        if plan.version != version || plan.status != "revising" {
            bail!("Revision is no longer applicable");
        }
        plan.status = "running".into();
        plan.updated_at = now();
        write(&tx, &plan)?;
        publish(
            &tx,
            Some(&plan.task_id),
            "plan.updated",
            &json!({"plan":plan}),
        )?;
        tx.commit()?;
        Ok(plan)
    }

    pub fn cancel_plan(
        &self,
        id: &str,
        expected_version: u64,
        source: &str,
    ) -> Result<Vec<String>> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let mut plan = read(&tx, id)?;
        if plan.version != expected_version {
            bail!("Plan changed; read the current version");
        }
        let workers = plan
            .nodes
            .iter()
            .filter(|n| n.status == "running")
            .filter_map(|n| n.worker_id.clone())
            .collect();
        plan.status = "cancelled".into();
        plan.version += 1;
        plan.updated_at = now();
        for node in &mut plan.nodes {
            if node.status != "completed" {
                node.status = "cancelled".into();
                node.generation = plan.version;
            }
        }
        write(&tx, &plan)?;
        tx.execute(
            "INSERT INTO plan_revisions VALUES (?1,?2,?3,?4,'Cancellation',?5)",
            params![
                plan.id,
                plan.version,
                serde_json::to_string(&plan)?,
                source,
                now()
            ],
        )?;
        publish(
            &tx,
            Some(&plan.task_id),
            "plan.cancelled",
            &json!({"plan":plan}),
        )?;
        tx.commit()?;
        Ok(workers)
    }

    /// Stop dispatch immediately while a natural-language edit is interpreted.
    /// Completed milestones survive; interrupted effects remain in worker journals.
    pub fn pause_plan_for_edit(
        &self,
        id: &str,
        source: &str,
        instruction: &str,
    ) -> Result<(Plan, Vec<String>)> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let mut plan = read(&tx, id)?;
        let workers = plan
            .nodes
            .iter()
            .filter(|n| n.status == "running")
            .filter_map(|n| n.worker_id.clone())
            .collect::<Vec<_>>();
        plan.version += 1;
        plan.status = "paused".into();
        plan.updated_at = now();
        for node in &mut plan.nodes {
            if node.status != "completed" {
                // Retain prior worker ID as evidence even when the result has not arrived.
                node.result = Some(
                    json!({"previous_result":node.result,"previous_worker":node.worker_id,"instruction":"Inspect prior worker events and actual state before repeating effects"}),
                );
                node.worker_id = None;
                node.status = "pending".into();
                node.generation = plan.version;
            }
        }
        write(&tx, &plan)?;
        tx.execute(
            "INSERT INTO plan_revisions VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                plan.id,
                plan.version,
                serde_json::to_string(&plan)?,
                source,
                instruction,
                now()
            ],
        )?;
        publish(
            &tx,
            Some(&plan.task_id),
            "plan.edit_requested",
            &json!({"plan":plan,"instruction":instruction,"source_task_id":source}),
        )?;
        tx.commit()?;
        Ok((plan, workers))
    }

    pub fn attach_plan_worker(
        &self,
        plan_id: &str,
        version: u64,
        node_id: &str,
        worker: &str,
    ) -> Result<()> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let mut plan = read(&tx, plan_id)?;
        if plan.version != version || plan.status != "running" {
            bail!("Plan changed before dispatch");
        }
        if crate::plan_reviews::blocks(&tx, &plan, node_id)? {
            bail!("This milestone is awaiting plan review or resolution");
        }
        let valid:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM subagents s JOIN tasks t ON t.id=s.id JOIN task_bindings b ON b.task_id=s.id WHERE s.id=?1 AND s.parent_task_id=?2 AND b.project_id IS ?3 AND t.status='queued')",params![worker,plan.task_id,plan.project_id],|r|r.get(0))?;
        if !valid {
            bail!(
                "A plan worker must be a queued child of the plan's owner in the same fixed scope"
            );
        }
        let complete = plan
            .nodes
            .iter()
            .filter(|n| n.status == "completed")
            .map(|n| n.spec.id.clone())
            .collect::<BTreeSet<_>>();
        let node = plan
            .nodes
            .iter_mut()
            .find(|n| n.spec.id == node_id)
            .context("Unknown milestone")?;
        if node.status != "pending" || !node.spec.depends_on.iter().all(|d| complete.contains(d)) {
            bail!("Milestone is not ready");
        }
        tx.execute(
            "INSERT INTO plan_workers(task_id,plan_id,node_id,generation) VALUES (?1,?2,?3,?4)",
            params![worker, plan_id, node_id, node.generation],
        )?;
        node.worker_id = Some(worker.into());
        node.status = "running".into();
        plan.updated_at = now();
        write(&tx, &plan)?;
        publish(
            &tx,
            Some(&plan.task_id),
            "plan.dispatched",
            &json!({"plan_id":plan_id,"plan_version":version,"node_id":node_id,"worker_id":worker}),
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Only predecessor milestones and previous generations of this milestone
    /// are readable by a worker; other projects/independent branches stay private.
    pub fn superseded_plan_workers(&self, plan: &str) -> Result<Vec<String>> {
        let workers=self.db()?.prepare("SELECT w.task_id FROM plan_workers w JOIN tasks t ON w.task_id=t.id WHERE w.plan_id=?1 AND t.status IN ('queued','running','waiting')")?.query_map([plan],|r|r.get::<_,String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(workers
            .into_iter()
            .filter(|w| self.check_plan_worker(w).is_err())
            .collect())
    }

    pub fn plan_evidence_sources(&self, worker: &str) -> Result<Vec<String>> {
        let db = self.db()?;
        let pin: Option<(String, String)> = db
            .query_row(
                "SELECT plan_id,node_id FROM plan_workers WHERE task_id=?1",
                [worker],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let Some((id, node)) = pin else {
            return Ok(vec![]);
        };
        let plan = read(&db, &id)?;
        let mut allowed = BTreeSet::from([node]);
        loop {
            let before = allowed.len();
            for node in &plan.nodes {
                if allowed.contains(&node.spec.id) {
                    allowed.extend(node.spec.depends_on.iter().cloned());
                }
            }
            if before == allowed.len() {
                break;
            }
        }
        Ok(db
            .prepare("SELECT task_id,node_id FROM plan_workers WHERE plan_id=?1")?
            .query_map([id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?
            .into_iter()
            .filter(|(task, node)| task != worker && allowed.contains(node))
            .map(|(task, _)| task)
            .collect())
    }

    pub fn check_plan_worker(&self, worker: &str) -> Result<()> {
        self.check_review_worker(worker)?;
        let db = self.db()?;
        let pin: Option<(String, String, u64)> = db
            .query_row(
                "SELECT plan_id,node_id,generation FROM plan_workers WHERE task_id=?1",
                [worker],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        if let Some((id, node, generation)) = pin {
            let plan = read(&db, &id)?;
            if !matches!(plan.status.as_str(), "running" | "revising")
                || !plan.nodes.iter().any(|n| {
                    n.spec.id == node
                        && n.generation == generation
                        && n.worker_id.as_deref() == Some(worker)
                        && n.status == "running"
                })
            {
                bail!("This milestone was superseded; do not execute or apply stale results");
            }
        }
        Ok(())
    }

    pub fn reconcile_plan_worker(&self, worker: &str) -> Result<bool> {
        let job = self.subagent(worker)?;
        if !matches!(
            job.status.as_str(),
            "completed" | "blocked" | "failed" | "cancelled" | "interrupted"
        ) {
            return Ok(false);
        }
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let pin: Option<(String, String, u64, bool)> = tx
            .query_row(
                "SELECT plan_id,node_id,generation,reconciled FROM plan_workers WHERE task_id=?1",
                [worker],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .optional()?;
        let Some((id, node, generation, reconciled)) = pin else {
            return Ok(false);
        };
        if reconciled {
            return Ok(true);
        };
        let mut plan = read(&tx, &id)?;
        let mut applied = false;
        if matches!(plan.status.as_str(), "running" | "revising") {
            if let Some(node) = plan.nodes.iter_mut().find(|n| {
                n.spec.id == node
                    && n.generation == generation
                    && n.worker_id.as_deref() == Some(worker)
            }) {
                node.status = if job.status == "completed" {
                    "completed"
                } else {
                    "blocked"
                }
                .into();
                node.result = job.result.clone();
                applied = true;
            }
            plan.updated_at = now();
            write(&tx, &plan)?;
        }
        tx.execute(
            "UPDATE plan_workers SET reconciled=1 WHERE task_id=?1",
            [worker],
        )?;
        publish(
            &tx,
            Some(&plan.task_id),
            if applied {
                "plan.result"
            } else {
                "plan.stale_result"
            },
            &json!({"plan_id":id,"plan_version":plan.version,"worker_id":worker,"status":job.status,"result":job.result,"applied":applied}),
        )?;
        tx.commit()?;
        Ok(true)
    }

    pub fn settle_plan(&self, id: &str) -> Result<()> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let mut plan = read(&tx, id)?;
        if plan.status != "running" {
            return Ok(());
        }
        let pending_review: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM plan_reviews WHERE plan_id=?1 AND version=?2 AND status IN ('queued','running','needs_resolution','resolving'))",params![plan.id,plan.version],|r|r.get(0))?;
        let unresolved_review: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM plan_reviews WHERE plan_id=?1 AND version=?2 AND status NOT IN ('passed','superseded'))",params![plan.id,plan.version],|r|r.get(0))?;
        let all_done = !unresolved_review && plan.nodes.iter().all(|n| n.status == "completed");
        let mut has_work = pending_review;
        for n in &plan.nodes {
            if n.status == "running"
                || n.status == "pending"
                    && n.spec.depends_on.iter().all(|d| {
                        plan.nodes
                            .iter()
                            .any(|p| p.spec.id == *d && p.status == "completed")
                    })
                    && !crate::plan_reviews::blocks(&tx, &plan, &n.spec.id)?
            {
                has_work = true;
            }
        }
        if all_done || !has_work {
            plan.status = if all_done { "completed" } else { "blocked" }.into();
            plan.updated_at = now();
            write(&tx, &plan)?;
            publish(
                &tx,
                Some(&plan.task_id),
                "plan.finished",
                &json!({"plan":plan}),
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn plan_history(&self, id: &str) -> Result<Vec<Value>> {
        Ok(self.db()?.prepare("SELECT version,body,source_task_id,reason,created_at FROM plan_revisions WHERE plan_id=?1 ORDER BY version DESC LIMIT 100")?.query_map([id],|r|Ok(json!({"version":r.get::<_,u64>(0)?,"plan":crate::parse_json(r.get(1)?)?,"source_task_id":r.get::<_,String>(2)?,"reason":r.get::<_,String>(3)?,"created_at":r.get::<_,String>(4)?})))?.collect::<rusqlite::Result<_>>()?)
    }
}
