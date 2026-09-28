//! Selective post-execution auditing on the existing asynchronous worker pool.
use crate::{Runtime, redaction::truncate};
use anyhow::{Context, Result, bail};
use fenix_storage::{CompletionReport, EvidenceInput, Plan, SubagentSpec};
use serde_json::{Value, json};

impl Runtime {
    pub(crate) fn validate_completion_observations(
        &self,
        task: &str,
        report: &CompletionReport,
    ) -> Result<()> {
        if report.verdict != "verified" {
            return Ok(());
        }
        for observation in self.store.completion_observations(task, report)? {
            if observation["tool"] == "file_read" {
                use sha2::Digest;
                let result = &observation["result"];
                let path = result["path"]
                    .as_str()
                    .context("File observation has no path")?;
                let contents = crate::files::read_text(std::path::Path::new(path))?;
                let digest = format!("{:x}", sha2::Sha256::digest(contents.as_bytes()));
                if result["sha256"].as_str() != Some(digest.as_str()) {
                    bail!(
                        "Observed file changed; read its current content before verifying: {path}"
                    );
                }
            }
        }
        Ok(())
    }

    pub(crate) fn completion_audit_tool_allowed(
        &self,
        task: &str,
        name: &str,
        args: &Value,
    ) -> Result<bool> {
        if self.store.completion_audit_for_worker(task)?.is_none() {
            return Ok(name != "completion_audit_read");
        }
        Ok(matches!(
            name,
            "tool_search"
                | "file_read"
                | "files_list"
                | "files_search"
                | "files_glob"
                | "document_read"
                | "design_read"
                | "library_read"
                | "library_search"
                | "skill_search"
                | "skill_list"
                | "skill_load"
                | "web_read"
                | "memory_search"
                | "memory_context"
                | "ask_user"
                | "subagent_finish"
                | "completion_audit_read"
                | "task_inspect"
        ) || name == "project_spec"
            && matches!(args["action"].as_str(), Some("get" | "changes"))
            || name == "plan_evidence" && matches!(args["action"].as_str(), Some("get" | "list"))
            || matches!(name, "schedule_manage" | "project_manage")
                && matches!(args["action"].as_str(), Some("get" | "list")))
    }

    pub(crate) fn completion_audit_evidence_sources(&self, task: &str) -> Result<Vec<String>> {
        let Some(audit) = self.store.completion_audit_for_worker(task)? else {
            return Ok(vec![]);
        };
        let plan = self.store.plan(&audit.plan_id)?;
        Ok(plan
            .nodes
            .iter()
            .filter(|n| audit.node_id.is_empty() || n.spec.id == audit.node_id)
            .filter_map(|n| n.worker_id.clone())
            .collect())
    }

    pub(crate) fn read_completion_audit(&self, task: &str, args: &Value) -> Result<Value> {
        let audit = self
            .store
            .completion_audit_for_worker(task)?
            .context("Not a completion auditor")?;
        self.store.check_completion_auditor(task)?;
        let plan = self.store.plan(&audit.plan_id)?;
        let offset = args["offset"].as_u64().unwrap_or(0) as usize;
        match args["part"].as_str().unwrap_or("overview") {
            "overview" => {
                let assigned = plan.nodes.iter().find(|node| node.spec.id == audit.node_id);
                let scope = if audit.node_id.is_empty() {
                    json!({"kind":"integrated", "objective":plan.objective,
                        "instruction":"Verify the completed deliverables against the full original request and explicit amendments, including requirements omitted from the plan."})
                } else {
                    let node = assigned.context("Audited milestone is missing")?;
                    json!({"kind":"milestone", "objective":node.spec.objective,
                        "expected_output":node.spec.expected_output,
                        "instruction":"Verify this assigned outcome, not the later milestones or the entire project. A diagnostic may pass while repair and tests are still pending. Pending ledger state is the input to your audit, not evidence of failure; verify or refute it using actual observations."})
                };
                Ok(
                    json!({"audit_id":audit.id,"plan_id":plan.id,"version":plan.version,
                    "node_id":audit.node_id,"scope":scope,
                    "original_request":self.store.task(&plan.task_id)?.request,
                    "objective":plan.objective,"constraints":plan.constraints,
                    "executor_journals":self.completion_audit_evidence_sources(task)?,
                    "instruction":"Read evidence and relevant nodes in pages. Executor reports are claims: inspect actual deliverables independently. Original requests and explicit owner revisions govern scope. Never invent acceptance requirements."}),
                )
            }
            "nodes" => {
                let nodes = plan
                    .nodes
                    .iter()
                    .filter(|n| audit.node_id.is_empty() || n.spec.id == audit.node_id)
                    .collect::<Vec<_>>();
                Ok(
                    json!({"nodes":nodes.iter().skip(offset).take(4).collect::<Vec<_>>(),"next_offset":(offset.saturating_add(4)<nodes.len()).then_some(offset.saturating_add(4))}),
                )
            }
            "evidence" => {
                let entries = self
                    .store
                    .plan_evidence(&plan.id, plan.version)?
                    .into_iter()
                    .filter(|e| {
                        audit.node_id.is_empty()
                            || e.input.node_id.as_deref() == Some(audit.node_id.as_str())
                    })
                    .collect::<Vec<_>>();
                Ok(
                    json!({"entries":entries.iter().skip(offset).take(8).collect::<Vec<_>>(),"next_offset":(offset.saturating_add(8)<entries.len()).then_some(offset.saturating_add(8))}),
                )
            }
            "answers" => {
                let mut tasks = self.completion_audit_evidence_sources(task)?;
                tasks.extend([plan.task_id.clone(), task.into()]);
                tasks.extend(
                    self.store
                        .plan_evidence(&plan.id, plan.version)?
                        .into_iter()
                        .filter(|e| {
                            audit.node_id.is_empty()
                                || e.input.node_id.as_deref() == Some(audit.node_id.as_str())
                        })
                        .map(|e| e.author_task_id),
                );
                tasks.sort();
                tasks.dedup();
                // Confirmed answers can cross review roles; their conclusions cannot.
                for review in self.store.plan_reviews(&plan.id)? {
                    tasks.extend(review.assignments.into_iter().filter_map(|a| a.worker_id));
                    tasks.extend(review.coordinator_id);
                }
                let answers = tasks
                    .iter()
                    .map(|id| self.store.saved_questions(Some(id)))
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .filter(|q| q["status"] == "answered")
                    .collect::<Vec<_>>();
                Ok(
                    json!({"answers":answers.iter().skip(offset).take(8).collect::<Vec<_>>(),"next_offset":(offset.saturating_add(8)<answers.len()).then_some(offset.saturating_add(8))}),
                )
            }
            "revisions" => {
                let revisions = self.store.plan_history(&plan.id)?;
                Ok(
                    json!({"revisions":revisions.iter().skip(offset).take(1).collect::<Vec<_>>(),"next_offset":(offset.saturating_add(1)<revisions.len()).then_some(offset.saturating_add(1))}),
                )
            }
            _ => bail!("Unknown audit context part"),
        }
    }

    pub(crate) async fn plan_evidence_tool(
        &self,
        task: &str,
        args: &Value,
        cancel: &tokio_util::sync::CancellationToken,
    ) -> Result<Value> {
        let plan = if let Some(id) = args["plan_id"].as_str() {
            let plan = self.store.plan(id)?;
            if self.store.task(task)?.origin == "subagent" {
                if self
                    .store
                    .plan_for_task(task)?
                    .is_none_or(|p| p.id != plan.id)
                {
                    bail!("Evidence is restricted to the assigned plan");
                }
            } else {
                self.authorize_task_read(task, &plan.task_id)?;
            }
            plan
        } else {
            self.store
                .plan_for_task(task)?
                .context("This task has no plan evidence ledger")?
        };
        if args["action"] == "list" {
            let entries = self.store.plan_evidence(&plan.id, plan.version)?;
            // Workers see their own node and predecessor entries, never unrelated branches.
            let entries = if self.store.task(task)?.origin == "subagent"
                && self.store.completion_audit_for_worker(task)?.is_none()
            {
                let sources = self.store.plan_evidence_sources(task)?;
                entries
                    .into_iter()
                    .filter(|e| {
                        e.input.node_id.as_ref().is_none_or(|id| {
                            plan.nodes.iter().any(|n| {
                                &n.spec.id == id
                                    && n.worker_id
                                        .as_ref()
                                        .is_some_and(|w| w == task || sources.contains(w))
                            })
                        })
                    })
                    .collect::<Vec<_>>()
            } else if let Some(audit) = self.store.completion_audit_for_worker(task)? {
                entries
                    .into_iter()
                    .filter(|e| {
                        audit.node_id.is_empty()
                            || e.input.node_id.as_deref() == Some(audit.node_id.as_str())
                    })
                    .collect()
            } else {
                entries
            };
            let offset = args["offset"].as_u64().unwrap_or(0) as usize;
            return Ok(
                json!({"plan_id":plan.id,"version":plan.version,"entries":entries.iter().skip(offset).take(8).collect::<Vec<_>>(),"next_offset":(offset.saturating_add(8)<entries.len()).then_some(offset.saturating_add(8))}),
            );
        }
        if args["action"] != "record" {
            bail!("Unknown evidence action");
        }
        let version = args["expected_version"]
            .as_u64()
            .context("Read the plan version first")?;
        let mut value = args["entry"].clone();
        self.redactor.value(&mut value);
        let mut input: EvidenceInput = serde_json::from_value(value)?;
        if self.store.task(task)?.origin == "subagent" {
            let node = plan
                .nodes
                .iter()
                .find(|n| n.worker_id.as_deref() == Some(task))
                .context("Only milestone executors may record candidates")?;
            input.node_id = Some(node.spec.id.clone());
        }
        let question = input.question.clone();
        let entry = self
            .store
            .record_plan_evidence(&plan.id, version, task, input)?;
        // Persist first. Waiting is on this task only; the master and independent
        // branches stay available. An answer confirms intent, not task completion.
        let answer = if let Some(question) = question {
            Some(
                self.ask(task, "clarification", &question, None, cancel)
                    .await?,
            )
        } else {
            None
        };
        Ok(
            json!({"entry":entry,"answer":answer,"instruction":"Owner answers are retained in the task contract. Use them as sources; they do not automatically verify artifacts. If the answer changes scope, ask the master to revise the plan."}),
        )
    }

    pub(crate) async fn completion_audit_cycle(&self, plan: &Plan) -> Result<()> {
        for audit in self.store.completion_audits(&plan.id)? {
            if audit.version != plan.version
                || !matches!(audit.status.as_str(), "queued" | "running")
            {
                continue;
            }
            if let Some(worker) = &audit.worker_id {
                let job = self.store.subagent(worker)?;
                if job.status == "completed" {
                    let report = job
                        .result
                        .as_ref()
                        .and_then(|r| r.get("completion_audit"))
                        .cloned()
                        .context("Auditor returned no report")
                        .and_then(|r| Ok(serde_json::from_value::<CompletionReport>(r)?));
                    match report.and_then(|r| {
                        self.validate_completion_observations(worker, &r)?;
                        self.store.finish_completion_audit(worker, r)
                    }) {
                        Ok(()) => {}
                        Err(error) => self.store.block_completion_audit(
                            &audit.id,
                            &format!(
                                "Audit could not be accepted: {}",
                                self.redactor.text(&error.to_string())
                            ),
                        )?,
                    }
                } else if matches!(
                    job.status.as_str(),
                    "blocked" | "failed" | "cancelled" | "interrupted"
                ) {
                    self.store.block_completion_audit(&audit.id,"Auditor could not finish; inspect saved evidence and revise the plan before retrying")?;
                }
            } else {
                if self
                    .store
                    .subagents(None, 32)?
                    .iter()
                    .filter(|j| matches!(j.status.as_str(), "queued" | "running" | "waiting"))
                    .count()
                    >= 32
                {
                    break;
                }
                let _active = self.subagents.active.lock().await;
                let spec=SubagentSpec {
                    profile:"general".into(),
                    objective:format!("Verify completion of plan {} v{} {}",plan.id,plan.version,if audit.node_id.is_empty(){"(integrated result)"}else{&audit.node_id}),
                    expected_output:"An independent, evidence-backed completion_audit report. Read actual artifacts; check original intent and relevant owner revisions. Ask unresolved questions.".into(),
                    context:json!({"audit_id":audit.id,"instruction":"Start with completion_audit_read. Fresh read-only context; executor conclusions are claims, not proof."}).to_string(),
                    constraints:"Read-only. No commands, writes, connectors, delegation or repairs. Same configured model as the master. Report unavailable checks honestly; do not invent new requirements.".into(),
                };
                let job = self.store.create_subagent_in_project(
                    &plan.task_id,
                    &spec,
                    plan.project_id.as_deref(),
                )?;
                if let Err(error) = self.store.attach_completion_auditor(&audit.id, &job.id) {
                    self.store.finish_subagent(
                        &job.id,
                        "cancelled",
                        &json!({"summary":"Audit superseded before dispatch"}),
                    )?;
                    return Err(error);
                }
                self.subagents.wake.notify_one();
            }
        }
        Ok(())
    }

    pub(crate) fn completion_evidence_summary(&self, plan: &Plan) -> Result<Value> {
        let entries = self.store.plan_evidence(&plan.id, plan.version)?;
        Ok(
            json!({"total":entries.len(),"verified":entries.iter().filter(|e|e.input.state==fenix_storage::EvidenceState::Verified).count(),"doubtful":entries.iter().filter(|e|e.input.state==fenix_storage::EvidenceState::Doubtful).count(),"pending":entries.iter().filter(|e|e.input.state==fenix_storage::EvidenceState::Pending).count(),"refuted":entries.iter().filter(|e|e.input.state==fenix_storage::EvidenceState::Refuted).count(),"recent":entries.iter().rev().take(4).map(|e|json!({"id":e.input.id,"kind":e.input.kind,"state":e.input.state,"statement":truncate(&e.input.statement,200)})).collect::<Vec<_>>()}),
        )
    }
}
