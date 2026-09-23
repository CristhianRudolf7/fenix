//! Adaptive plan review scheduling on the existing background worker pool.
use crate::{Runtime, redaction::truncate};
use anyhow::{Context, Result, bail};
use fenix_storage::{
    Plan, PlanBrief, PlanReview, ReviewAssignment, ReviewReport, SpecialistRequest, SubagentSpec,
};
use serde_json::{Value, json};

impl Runtime {
    pub(crate) fn review_tool_allowed(&self, task: &str, name: &str, args: &Value) -> Result<bool> {
        if self.store.completion_audit_for_worker(task)?.is_some() {
            return self.completion_audit_tool_allowed(task, name, args);
        }
        if !self.completion_audit_tool_allowed(task, name, args)? {
            return Ok(false);
        }
        let reviewer = self.store.review_for_worker(task)?.is_some();
        let coordinator = self.store.get::<String>("review_edits", task)?.is_some();
        if !reviewer && !coordinator {
            return Ok(name != "plan_review_read");
        }
        Ok(matches!(
            name,
            "tool_search"
                | "files_list"
                | "document_read"
                | "design_read"
                | "design_list"
                | "design_templates"
                | "file_read"
                | "files_search"
                | "files_glob"
                | "code_query"
                | "memory_search"
                | "memory_context"
                | "skill_list"
                | "skill_search"
                | "skill_load"
                | "task_inspect"
                | "ask_user"
                | "web_search"
                | "web_read"
                | "plan_review_read"
        ) || name == "project_spec"
            && matches!(args["action"].as_str(), Some("get" | "changes"))
            || reviewer && name == "subagent_finish"
            || coordinator
                && name == "plan_manage"
                && matches!(
                    args["action"].as_str(),
                    Some("get" | "revise" | "resolve_review")
                ))
    }

    pub(crate) fn review_evidence_sources(&self, task: &str) -> Result<Vec<String>> {
        let Some(review) = self.store.review_for_worker(task)? else {
            return Ok(vec![]);
        };
        let plan = self.store.plan(&review.plan_id)?;
        // Exact plan membership, never every task in the same project or peer reports.
        Ok(plan
            .nodes
            .iter()
            .filter_map(|n| n.worker_id.clone())
            .collect())
    }

    pub(crate) fn read_review_context(&self, task: &str, args: &Value) -> Result<Value> {
        let review = if let Some(review) = self.store.review_for_worker(task)? {
            review
        } else {
            self.store.plan_review(
                &self
                    .store
                    .get::<String>("review_edits", task)?
                    .context("Not assigned to a plan review")?,
            )?
        };
        let plan = self.store.plan(&review.plan_id)?;
        if plan.version != review.version {
            bail!("Review was superseded");
        }
        let role = review
            .assignments
            .iter()
            .find(|a| a.worker_id.as_deref() == Some(task));
        match args["part"].as_str().unwrap_or("overview") {
            "overview" => Ok(
                json!({"review_id":review.id,"version":review.version,"round":review.round,"reason":review.reason,"scope":review.node_ids,"assignment":role.map(|a| &a.request),"original_request":self.store.task(&plan.task_id)?.request,"objective":plan.objective,"constraints":plan.constraints,"brief":review.brief,"milestone_count":plan.nodes.len(),"outline":plan.nodes.iter().map(|n| json!({"id":n.spec.id,"objective":truncate(&n.spec.objective,400),"depends_on":n.spec.depends_on,"status":n.status})).collect::<Vec<_>>(),"instruction":"Read full milestone details with part=nodes before judging them; use evidence for confirmed answers."}),
            ),
            "nodes" => {
                let offset = args["offset"].as_u64().unwrap_or(0).min(32) as usize;
                let limit = args["limit"].as_u64().unwrap_or(2).clamp(1, 4) as usize;
                Ok(
                    json!({"nodes":plan.nodes.iter().skip(offset).take(limit).collect::<Vec<_>>(),"next_offset":(offset+limit<plan.nodes.len()).then_some(offset+limit)}),
                )
            }
            "evidence" => {
                let mut sources = vec![plan.task_id.clone()];
                if let Some(previous) = &review.previous_review_id {
                    let previous = self.store.plan_review(previous)?;
                    if let Some(editor) = previous.coordinator_id {
                        sources.push(editor);
                    }
                    sources.extend(
                        previous
                            .assignments
                            .iter()
                            .filter_map(|a| a.worker_id.clone()),
                    );
                }
                sources.extend(
                    review
                        .assignments
                        .iter()
                        .filter_map(|a| a.worker_id.clone()),
                );
                let answers = sources
                    .iter()
                    .map(|s| self.store.saved_questions(Some(s)))
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .filter(|q| q["status"] == "answered")
                    .collect::<Vec<_>>();
                // Questions/answers are evidence; peer conclusions are deliberately absent.
                Ok(
                    json!({"confirmed_answers":answers,"worker_journals":self.review_evidence_sources(task)?,"previous_review_id":review.previous_review_id}),
                )
            }
            _ => bail!("Unknown review context part"),
        }
    }

    pub(crate) fn begin_plan_review(
        &self,
        plan: &Plan,
        args: &Value,
        previous: Option<&PlanReview>,
    ) -> Result<Option<PlanReview>> {
        let brief: PlanBrief = if let Some(value) = args.get("brief") {
            serde_json::from_value(value.clone())?
        } else {
            self.store.get("plan_briefs", &plan.id)?.unwrap_or_default()
        };
        brief.validate()?;
        self.store.put("plan_briefs", &plan.id, &brief)?;
        let saved = self.store.plan_evidence(&plan.id, plan.version)?;
        for (section, kind, values) in [
            (
                "requirement",
                fenix_storage::EvidenceKind::Requirement,
                &brief.requirements,
            ),
            (
                "observation",
                fenix_storage::EvidenceKind::Fact,
                &brief.observations,
            ),
            (
                "assumption",
                fenix_storage::EvidenceKind::Fact,
                &brief.assumptions,
            ),
            (
                "question",
                fenix_storage::EvidenceKind::Fact,
                &brief.open_questions,
            ),
        ] {
            for (index, statement) in values.iter().enumerate() {
                let id = format!("note:brief:{section}:{index}");
                if saved.iter().any(|e| e.input.id == id) {
                    continue;
                }
                self.store.record_plan_evidence(&plan.id, plan.version, &plan.task_id, fenix_storage::EvidenceInput {
                    id, kind: kind.clone(), state: if matches!(section, "assumption" | "question") { fenix_storage::EvidenceState::Doubtful } else { fenix_storage::EvidenceState::Pending },
                    statement:statement.clone(), source:"Agent-authored plan brief; check against the original request and explicit answers".into(), node_id:None,
                    question: (section == "question").then(||statement.clone()),
                })?;
            }
        }
        let reason = args["review_reason"].as_str().unwrap_or("");
        let complex =
            plan.nodes.len() >= 3 || plan.nodes.iter().any(|n| !n.spec.depends_on.is_empty());
        let previously_reviewed = self
            .store
            .plan_reviews(&plan.id)?
            .iter()
            .any(|r| r.version < plan.version);
        if previous.is_none()
            && !complex
            && !previously_reviewed
            && reason.is_empty()
            && brief.assumptions.is_empty()
            && brief.open_questions.is_empty()
        {
            return Ok(None);
        }
        let node_ids: Vec<String> = if let Some(nodes) = args.get("review_nodes") {
            serde_json::from_value(nodes.clone())?
        } else if let Some(previous) = previous {
            let old = self
                .store
                .plan_history(&plan.id)?
                .into_iter()
                .find(|r| r["version"] == previous.version);
            let global_changed = old.as_ref().is_none_or(|r| {
                r["plan"]["objective"] != plan.objective
                    || r["plan"]["constraints"] != plan.constraints
            });
            if global_changed {
                vec![]
            } else {
                plan.nodes
                    .iter()
                    .filter(|n| {
                        n.generation == plan.version || previous.node_ids.contains(&n.spec.id)
                    })
                    .map(|n| n.spec.id.clone())
                    .collect()
            }
        } else {
            serde_json::from_value(args.get("review_nodes").cloned().unwrap_or(json!([])))?
        };
        let reason = if !reason.is_empty() {
            reason
        } else if previous.is_some() {
            "Final response check after the master's review revision"
        } else {
            "Initial strategy and intent review"
        };
        let review = PlanReview {
            id: String::new(), plan_id: plan.id.clone(), version: plan.version,
            round: u8::from(previous.is_some()), status: "queued".into(), node_ids: node_ids.clone(),
            reason: self.redactor.text(reason), brief,
            assignments: vec![ReviewAssignment { role:"principal".into(),request:SpecialistRequest {specialty:"Plan scope and coherence".into(),question:"Validate alignment, unsupported requirements, dependencies, feasibility and completion criteria. Investigate every unresolved doubt. Request specialists only for distinct necessary expertise.".into(),node_ids,skills:vec![]},worker_id:None,report:None }],
            coordinator_id:None,resolutions:vec![],previous_review_id:previous.map(|r|r.id.clone()),created_at:String::new(),updated_at:String::new(),
        };
        Ok(Some(self.store.start_plan_review(review)?))
    }

    pub(crate) async fn supersede_plan_reviews(
        &self,
        plan: &str,
        except: Option<&str>,
    ) -> Result<()> {
        for review in self.store.plan_reviews(plan)? {
            if matches!(
                review.status.as_str(),
                "passed" | "superseded" | "interrupted"
            ) {
                continue;
            }
            self.store.close_plan_review(
                &review.id,
                "superseded",
                "A newer plan version superseded this review",
            )?;
            let workers = review
                .assignments
                .iter()
                .filter_map(|a| a.worker_id.clone())
                .collect::<Vec<_>>();
            self.stop_plan_workers(&workers).await?;
            if let Some(editor) = review
                .coordinator_id
                .filter(|id| Some(id.as_str()) != except)
            {
                if let Some(active) = self.active.lock().await.get(&editor) {
                    active.cancel.cancel();
                } else if self.store.task(&editor)?.status == "queued" {
                    self.store
                        .set_task_status(&editor, "cancelled", Some("Review superseded"))?;
                }
            }
        }
        Ok(())
    }

    /// Never awaits model inference while holding the plan lock. Review jobs share
    /// the normal four-call pool and release capacity while waiting for an answer.
    pub(crate) async fn plan_review_cycle(&self, plan: &Plan) -> Result<()> {
        for review in self.store.plan_reviews(&plan.id)? {
            if review.version != plan.version
                || !matches!(
                    review.status.as_str(),
                    "queued" | "running" | "needs_resolution" | "resolving"
                )
            {
                continue;
            }
            for assignment in &review.assignments {
                if !matches!(
                    self.store.plan_review(&review.id)?.status.as_str(),
                    "queued" | "running"
                ) {
                    break;
                }
                if let Some(worker) = &assignment.worker_id {
                    if assignment.report.is_some() {
                        continue;
                    }
                    let job = self.store.subagent(worker)?;
                    if job.status == "completed" {
                        match serde_json::from_value::<ReviewReport>(
                            job.result
                                .as_ref()
                                .map(|r| r["review"].clone())
                                .unwrap_or(Value::Null),
                        ) {
                            Ok(report) => self.store.record_review_report(worker, report)?,
                            Err(_) => self.store.close_plan_review(
                                &review.id,
                                "blocked",
                                "Reviewer returned no valid structured report; inspect its journal",
                            )?,
                        }
                    } else if matches!(
                        job.status.as_str(),
                        "failed" | "blocked" | "cancelled" | "interrupted"
                    ) {
                        self.store.close_plan_review(
                            &review.id,
                            "blocked",
                            "Reviewer could not finish; inspect its journal before continuing",
                        )?;
                        break;
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
                    let spec = SubagentSpec {
                        profile: self.profile_for_skills(&assignment.request.skills)?,
                        objective: format!("Review plan {} v{}: {}",plan.id,plan.version,assignment.request.specialty),
                        expected_output:"An evidence-backed structured review using subagent_finish.review, not execution of the milestones".into(),
                        context:json!({"review_id":review.id,"role":assignment.role,"round":review.round,"question":assignment.request.question,"skills":assignment.request.skills,"instruction":"Use plan_review_read to obtain the original request, brief and relevant milestones; consult evidence for existing answers."}).to_string(),
                        constraints:"Read only; no effects, no delegation; same configured model as the master".into(),
                    };
                    let job = self.store.create_subagent_in_project(
                        &plan.task_id,
                        &spec,
                        plan.project_id.as_deref(),
                    )?;
                    self.store
                        .attach_review_worker(&review.id, &assignment.role, &job.id)?;
                    self.subagents.wake.notify_one();
                }
            }
            let review = self.store.plan_review(&review.id)?;
            if review.status == "needs_resolution" {
                let _configure = self.configure_lock.lock().await;
                let mut binding = self.store.task_binding(&plan.task_id)?;
                binding.reply_to_task_id = Some(plan.task_id.clone());
                let request = format!(
                    "Resolve review {} for plan {} version {}. Read plan_manage get for independent findings and source evidence. This is the sole response to these objections. Clarify every missing owner decision. Resolve with evidence or revise the plan; never manufacture scope or repeat previous effects.",
                    review.id, plan.id, plan.version
                );
                let editor = self
                    .store
                    .enqueue_bound(&request, "plan_review", None, &binding)?;
                self.store.inherit_destination(&editor.id, &plan.task_id)?;
                self.store.put("review_edits", &editor.id, &review.id)?;
                self.store.put("plan_edits", &editor.id, &plan.id)?;
                self.store.put("plan_edit_owner", &plan.id, &editor.id)?;
                self.store.set_review_coordinator(&review.id, &editor.id)?;
                self.wake.notify_one();
            } else if review.status == "resolving"
                && let Some(editor) = &review.coordinator_id
                && matches!(
                    self.store.task(editor)?.status.as_str(),
                    "completed" | "failed" | "blocked" | "cancelled" | "interrupted"
                )
            {
                self.store.close_plan_review(&review.id,"blocked","Resolution turn ended without settling its objections; an owner decision is needed")?;
            }
            if self.store.plan_review(&review.id)?.status == "blocked" {
                let workers = review
                    .assignments
                    .iter()
                    .filter_map(|a| a.worker_id.clone())
                    .collect::<Vec<_>>();
                self.stop_plan_workers(&workers).await?;
            }
        }
        // One review per failed execution, with downstream gating. No automatic
        // retry of effects. A revised plan is required before a failed node reruns.
        let audits = self.store.completion_audits(&plan.id)?;
        let reviews = self.store.plan_reviews(&plan.id)?;
        if reviews.iter().any(|r| {
            r.version == plan.version && !matches!(r.status.as_str(), "passed" | "superseded")
        }) {
            return Ok(());
        }
        let blocked = plan
            .nodes
            .iter()
            .filter(|n| n.status == "blocked" && n.worker_id.is_some())
            .collect::<Vec<_>>();
        let mut nodes = vec![];
        let mut workers = vec![];
        for node in blocked {
            if audits.iter().any(|a| {
                a.version == plan.version && a.node_id == node.spec.id && a.status == "blocked"
            }) {
                continue;
            }
            let worker = node.worker_id.as_ref().unwrap();
            if self
                .store
                .get::<bool>("reviewed_failures", worker)?
                .is_none()
            {
                nodes.push(node.spec.id.clone());
                workers.push(worker.clone());
            }
        }
        if !nodes.is_empty() {
            let final_response = reviews
                .iter()
                .rev()
                .find(|r| r.version == plan.version && r.round == 1);
            self.begin_plan_review(plan,&json!({"review_reason":format!("Execution blockers in workers {}",workers.join(",")),"review_nodes":nodes}),final_response)?;
            for worker in workers {
                self.store.put("reviewed_failures", &worker, &true)?;
            }
        }
        Ok(())
    }
}
