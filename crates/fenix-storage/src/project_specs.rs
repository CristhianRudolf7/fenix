//! Agreed project behavior and explicit amendments, separate from execution proof.
use super::*;
use anyhow::ensure;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectRequirement {
    pub id: String,
    pub statement: String,
    pub revision: u64,
    pub active: bool,
    pub source: String,
    pub author_task_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecDeltaInput {
    pub requirement_id: String,
    /// add, modify or remove. IDs stay stable; removed IDs retain tombstones.
    pub action: String,
    #[serde(default)]
    pub statement: String,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequirementDelta {
    pub action: String,
    pub requirement_id: String,
    pub before: Option<ProjectRequirement>,
    pub after: ProjectRequirement,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectSpecChange {
    pub id: String,
    pub project_id: String,
    pub task_id: String,
    pub base_version: u64,
    pub status: String,
    pub reason: String,
    pub deltas: Vec<RequirementDelta>,
    pub created_at: String,
    pub resolved_by: Option<String>,
    pub resolution: Option<String>,
    pub applied_version: Option<u64>,
}

fn version(db: &Connection, project: &str) -> Result<u64> {
    projects::query_project(db, project)?;
    Ok(db
        .query_row(
            "SELECT version FROM project_specs WHERE project_id=?1",
            [project],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(0))
}

fn requirement(db: &Connection, project: &str, id: &str) -> Result<Option<ProjectRequirement>> {
    let body: Option<String> = db
        .query_row(
            "SELECT body FROM project_requirements WHERE project_id=?1 AND id=?2",
            params![project, id],
            |r| r.get(0),
        )
        .optional()?;
    body.map(|body| Ok(serde_json::from_str(&body)?))
        .transpose()
}

fn change(db: &Connection, project: &str, id: &str) -> Result<ProjectSpecChange> {
    let body: String = db
        .query_row(
            "SELECT body FROM project_spec_changes WHERE project_id=?1 AND id=?2",
            params![project, id],
            |r| r.get(0),
        )
        .optional()?
        .context("Project change not found in this scope")?;
    Ok(serde_json::from_str(&body)?)
}

fn authorize_write(db: &Connection, task: &str, project: &str, master: bool) -> Result<()> {
    let binding = projects::read_binding(db, task)?;
    ensure!(
        binding.bound
            && binding.mode == "project"
            && binding.project_id.as_deref() == Some(project),
        "Contract writes require this task's fixed project scope"
    );
    let (origin, status): (String, String) =
        db.query_row("SELECT origin,status FROM tasks WHERE id=?1", [task], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })?;
    ensure!(
        status == "running" && origin != "subagent_report",
        "Only a running execution task may amend a project contract"
    );
    ensure!(
        !master || origin != "subagent",
        "Only the master may consolidate or discard contract changes"
    );
    let reviewer: bool = db.query_row("SELECT EXISTS(SELECT 1 FROM plan_review_workers WHERE task_id=?1 UNION SELECT 1 FROM completion_audits WHERE worker_id=?1)", [task], |r|r.get(0))?;
    ensure!(!reviewer, "Reviewers cannot amend project contracts");
    Ok(())
}

fn conflicts(db: &Connection, proposal: &ProjectSpecChange) -> Result<Vec<Value>> {
    let mut found = Vec::new();
    for delta in &proposal.deltas {
        let expected = if proposal.status == "applied" {
            Some(delta.after.clone())
        } else {
            delta.before.clone()
        };
        if requirement(db, &proposal.project_id, &delta.requirement_id)? != expected {
            found.push(json!({"kind":"stale_requirement","requirement_id":delta.requirement_id}));
        }
    }
    let mut query = db.prepare("SELECT body FROM project_spec_changes WHERE project_id=?1 AND status='proposed' AND id<>?2 ORDER BY created_at,id")?;
    for row in query.query_map(params![proposal.project_id, proposal.id], |r| {
        r.get::<_, String>(0)
    })? {
        let other: ProjectSpecChange = serde_json::from_str(&row?)?;
        for delta in &proposal.deltas {
            if other
                .deltas
                .iter()
                .any(|d| d.requirement_id == delta.requirement_id)
            {
                found.push(json!({"kind":"overlapping_proposal","requirement_id":delta.requirement_id,"change_id":other.id,"task_id":other.task_id}));
            }
        }
    }
    Ok(found)
}

impl Store {
    /// Bounded reads; deleted requirements remain in history and as revision tombstones.
    pub fn project_spec(&self, project: &str, offset: usize, limit: usize) -> Result<Value> {
        let db = self.db()?;
        let revision = version(&db, project)?;
        let total: usize = db.query_row("SELECT count(*) FROM project_requirements WHERE project_id=?1 AND json_extract(body,'$.active')=1", [project], |r|r.get(0))?;
        let limit = limit.clamp(1, 50);
        let mut query = db.prepare("SELECT body FROM project_requirements WHERE project_id=?1 AND json_extract(body,'$.active')=1 ORDER BY id LIMIT ?2 OFFSET ?3")?;
        let requirements = query
            .query_map(params![project, limit, offset], |r| r.get::<_, String>(0))?
            .map(|row| Ok(serde_json::from_str::<ProjectRequirement>(&row?)?))
            .collect::<Result<Vec<_>>>()?;
        let next = offset.saturating_add(requirements.len());
        Ok(
            json!({"project_id":project,"version":revision,"total":total,"requirements":requirements,"next_offset":(next<total).then_some(next),"meaning":"Agreed behavior, not proof of implementation or user authorization"}),
        )
    }

    pub fn project_spec_changes(&self, project: &str, offset: usize) -> Result<Value> {
        let db = self.db()?;
        version(&db, project)?;
        let total: usize = db.query_row(
            "SELECT count(*) FROM project_spec_changes WHERE project_id=?1",
            [project],
            |r| r.get(0),
        )?;
        let mut query = db.prepare("SELECT body FROM project_spec_changes WHERE project_id=?1 ORDER BY created_at DESC,id DESC LIMIT 20 OFFSET ?2")?;
        let rows = query
            .query_map(params![project, offset], |r| r.get::<_, String>(0))?
            .map(|row| Ok(serde_json::from_str::<ProjectSpecChange>(&row?)?))
            .collect::<Result<Vec<_>>>()?;
        let next = offset.saturating_add(rows.len());
        // The index omits full deltas; retrieve a change explicitly to inspect its evidence.
        let rows = rows.into_iter().map(|c| json!({"id":c.id,"task_id":c.task_id,"base_version":c.base_version,"status":c.status,"reason":c.reason,"requirement_ids":c.deltas.iter().map(|d|&d.requirement_id).collect::<Vec<_>>(),"created_at":c.created_at,"applied_version":c.applied_version})).collect::<Vec<_>>();
        Ok(json!({"changes":rows,"total":total,"next_offset":(next<total).then_some(next)}))
    }

    pub fn project_spec_change(&self, project: &str, id: &str) -> Result<Value> {
        let db = self.db()?;
        let proposal = change(&db, project, id)?;
        let issues = if proposal.status == "proposed" {
            conflicts(&db, &proposal)?
        } else {
            vec![]
        };
        Ok(json!({"change":proposal,"conflicts":issues}))
    }

    pub fn propose_project_spec(
        &self,
        task: &str,
        project: &str,
        expected: u64,
        reason: &str,
        inputs: &[SpecDeltaInput],
    ) -> Result<Value> {
        ensure!(
            !reason.trim().is_empty() && reason.len() <= 4000,
            "A bounded change rationale is required"
        );
        ensure!(
            !inputs.is_empty() && inputs.len() <= 50,
            "Use 1–50 explicit deltas per change"
        );
        let mut db = self.db()?;
        let tx = db.transaction()?;
        authorize_write(&tx, task, project, false)?;
        ensure!(
            version(&tx, project)? == expected,
            "Project contract changed; reread before proposing"
        );
        let mut seen = BTreeSet::new();
        let mut deltas = Vec::new();
        for input in inputs {
            let key = &input.requirement_id;
            ensure!(
                !key.is_empty()
                    && key.len() <= 120
                    && key.bytes().all(|c| c.is_ascii_lowercase()
                        || c.is_ascii_digit()
                        || b"._-".contains(&c)),
                "Use stable lowercase requirement IDs (letters, digits, dot, underscore, dash)"
            );
            ensure!(
                seen.insert(key),
                "A change cannot amend a requirement twice"
            );
            ensure!(
                !input.source.trim().is_empty() && input.source.len() <= 2000,
                "Every delta needs its owner/document provenance; assumptions remain pending"
            );
            let before = requirement(&tx, project, key)?;
            let active = before.as_ref().is_some_and(|r| r.active);
            match input.action.as_str() {
                "add" => ensure!(!active, "Requirement already exists; use modify"),
                "modify" | "remove" => {
                    ensure!(active, "Requirement does not exist; reread the contract")
                }
                _ => bail!("Unknown requirement operation"),
            }
            let removing = input.action == "remove";
            ensure!(
                !removing || input.statement.trim().is_empty(),
                "A removal must not contain replacement text"
            );
            ensure!(
                removing || (!input.statement.trim().is_empty() && input.statement.len() <= 8000),
                "A requirement needs 1–8000 bytes of behavior text"
            );
            let after = ProjectRequirement {
                id: key.clone(),
                statement: if removing {
                    before.as_ref().unwrap().statement.clone()
                } else {
                    input.statement.trim().into()
                },
                revision: before.as_ref().map_or(1, |r| r.revision + 1),
                active: !removing,
                source: input.source.trim().into(),
                author_task_id: task.into(),
            };
            deltas.push(RequirementDelta {
                action: input.action.clone(),
                requirement_id: key.clone(),
                before,
                after,
            });
        }
        // Repeated tool calls return the same pending proposal instead of duplicating it.
        let encoded = serde_json::to_value(&deltas)?;
        let existing: Vec<String> = tx.prepare("SELECT body FROM project_spec_changes WHERE task_id=?1 AND project_id=?2 AND status='proposed'")?.query_map(params![task,project],|r|r.get(0))?.collect::<rusqlite::Result<_>>()?;
        for body in existing {
            let old: ProjectSpecChange = serde_json::from_str(&body)?;
            if old.reason == reason.trim() && serde_json::to_value(&old.deltas)? == encoded {
                let issues = conflicts(&tx, &old)?;
                return Ok(json!({"change":old,"conflicts":issues,"duplicate":true}));
            }
        }
        let proposal = ProjectSpecChange {
            id: id(),
            project_id: project.into(),
            task_id: task.into(),
            base_version: expected,
            status: "proposed".into(),
            reason: reason.trim().into(),
            deltas,
            created_at: now(),
            resolved_by: None,
            resolution: None,
            applied_version: None,
        };
        tx.execute("INSERT INTO project_spec_changes(id,project_id,task_id,status,body,created_at) VALUES (?1,?2,?3,?4,?5,?6)",params![proposal.id,project,task,proposal.status,serde_json::to_string(&proposal)?,proposal.created_at])?;
        let issues = conflicts(&tx, &proposal)?;
        events::publish(
            &tx,
            Some(task),
            "project_spec.proposed",
            &json!({"project_id":project,"change_id":proposal.id,"base_version":expected,"conflicts":issues}),
        )?;
        tx.commit()?;
        Ok(json!({"change":proposal,"conflicts":issues}))
    }

    /// Consolidation changes the agreed contract, never execution/audit status.
    pub fn resolve_project_spec(
        &self,
        task: &str,
        project: &str,
        change_id: &str,
        expected: u64,
        apply: bool,
        reason: &str,
    ) -> Result<ProjectSpecChange> {
        ensure!(
            !reason.trim().is_empty() && reason.len() <= 4000,
            "Resolution needs a rationale and source/evidence references"
        );
        let mut db = self.db()?;
        let tx = db.transaction()?;
        authorize_write(&tx, task, project, true)?;
        let mut proposal = change(&tx, project, change_id)?;
        let target = if apply { "applied" } else { "discarded" };
        if proposal.status == target {
            return Ok(proposal);
        }
        ensure!(
            proposal.status == "proposed",
            "Resolved changes cannot be reopened; propose a new delta"
        );
        let current = version(&tx, project)?;
        ensure!(
            current == expected,
            "Project contract changed; reread before consolidation"
        );
        if apply {
            let issues = conflicts(&tx, &proposal)?;
            ensure!(
                issues.is_empty(),
                "Reconcile stale or overlapping requirements before consolidation: {}",
                json!(issues)
            );
            for delta in &proposal.deltas {
                tx.execute("INSERT INTO project_requirements(project_id,id,body) VALUES (?1,?2,?3) ON CONFLICT(project_id,id) DO UPDATE SET body=excluded.body",params![project,delta.requirement_id,serde_json::to_string(&delta.after)?])?;
            }
            tx.execute("INSERT INTO project_specs(project_id,version) VALUES (?1,?2) ON CONFLICT(project_id) DO UPDATE SET version=excluded.version",params![project,current+1])?;
            proposal.applied_version = Some(current + 1);
        }
        proposal.status = target.into();
        proposal.resolved_by = Some(task.into());
        proposal.resolution = Some(reason.trim().into());
        tx.execute(
            "UPDATE project_spec_changes SET status=?2,body=?3 WHERE id=?1",
            params![
                change_id,
                proposal.status,
                serde_json::to_string(&proposal)?
            ],
        )?;
        events::publish(
            &tx,
            Some(task),
            "project_spec.resolved",
            &json!({"project_id":project,"change_id":change_id,"status":proposal.status,"applied_version":proposal.applied_version}),
        )?;
        tx.commit()?;
        Ok(proposal)
    }

    /// Fail before a worker's Git merge if its declared requirements are stale or contested.
    pub fn check_task_spec_changes(&self, task: &str) -> Result<()> {
        let db = self.db()?;
        let mut query = db.prepare("SELECT body FROM project_spec_changes WHERE task_id=?1 ORDER BY created_at DESC,id DESC")?;
        let mut covered = BTreeSet::new();
        for row in query.query_map([task], |r| r.get::<_, String>(0))? {
            let mut proposal: ProjectSpecChange = serde_json::from_str(&row?)?;
            // A later explicit amendment supersedes this task's own older delta.
            proposal
                .deltas
                .retain(|d| covered.insert(d.requirement_id.clone()));
            if proposal.deltas.is_empty() {
                continue;
            }
            ensure!(
                proposal.status != "discarded",
                "Worker declared a discarded requirement change {}; assign reconciled work to a new task before integration",
                proposal.id
            );
            let issues = conflicts(&db, &proposal)?;
            ensure!(
                issues.is_empty(),
                "Project requirement conflict in change {}: {}. Reconcile with the master before integration",
                proposal.id,
                json!(issues)
            );
        }
        Ok(())
    }
}
