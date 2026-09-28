//! Background milestone scheduling and version-aware conversational edits.
use crate::{Runtime, redaction::truncate};
use anyhow::{Context, Result, bail};
use fenix_storage::{Plan, PlanNodeSpec, SubagentSpec};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};

fn progress_description(status: &str) -> &'static str {
    match status {
        "completed" => "terminado",
        "cancelled" => "detenido",
        "blocked" | "failed" => "necesita atención antes de continuar",
        "running" | "active" => "en curso",
        "waiting" => "esperando tu respuesta",
        "queued" | "pending" | "ready" => "pendiente de empezar",
        "paused" | "revising" | "editing" => "en pausa mientras ajusto el plan",
        _ => "pendiente de revisión",
    }
}

impl Runtime {
    fn plan_display_name(&self, id: Option<&str>, node_id: Option<&str>) -> String {
        id.and_then(|id| self.store.plan(id).ok())
            .map(|plan| {
                let node = node_id.and_then(|id| plan.nodes.iter().find(|node| node.spec.id == id));
                match node {
                    Some(node) => format!(
                        "{} — {}",
                        truncate(&plan.objective, 160),
                        truncate(&node.spec.objective, 160)
                    ),
                    None => truncate(&plan.objective, 240),
                }
            })
            .unwrap_or_else(|| "el trabajo solicitado".into())
    }

    pub(crate) async fn handle_plan_control(
        &self,
        text: &str,
        origin: &str,
        reference: Option<&str>,
    ) -> Result<Option<Value>> {
        let mut parts = text.splitn(2, char::is_whitespace);
        let prefix = parts.next().unwrap_or("").to_lowercase();
        if matches!(prefix.as_str(), "cancelar" | "cancel" | "/cancel") {
            let target = parts
                .next()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .or(reference);
            if let Some(target) = target {
                let plan = self
                    .store
                    .plan(target)
                    .ok()
                    .or(self.store.plan_for_task(target)?);
                if let Some(plan) = plan.filter(|p| p.id == target || p.task_id == target) {
                    return Ok(Some(self.plan_manage(&plan.task_id,&json!({"action":"cancel","plan_id":plan.id,"expected_version":plan.version})).await?));
                }
            }
            return Ok(None);
        }
        if !matches!(
            prefix.as_str(),
            "replanificar" | "replan" | "/replan" | "replanifica"
        ) {
            return Ok(None);
        }
        let rest = parts.next().unwrap_or("").trim();
        let (target, instruction) = if let Some(task) = reference {
            (task, rest)
        } else {
            rest.split_once(char::is_whitespace).context("Use replanificar TASK_OR_PLAN_ID followed by the requested change, or reply to the task message")?
        };
        Ok(Some(self.steer_task(target, instruction, origin).await?))
    }

    pub async fn steer_task(&self, target: &str, instruction: &str, origin: &str) -> Result<Value> {
        let instruction = self.redactor.text(instruction.trim());
        if instruction.is_empty() || instruction.len() > 8000 {
            bail!("Provide 1–8000 bytes describing the requested change");
        }
        let plan = match self.store.plan(target) {
            Ok(plan) => Some(plan),
            Err(_) => self.store.plan_for_task(target)?,
        };
        let plan = plan.or(self
            .store
            .get::<String>("plan_edits", target)?
            .map(|id| self.store.plan(&id))
            .transpose()?);
        if let Some(plan) = plan {
            let _lock = self.plan_lock.lock().await;
            for node in &self.store.plan(&plan.id)?.nodes {
                if let Some(worker) = &node.worker_id {
                    self.store.reconcile_plan_worker(worker)?;
                }
            }
            let mut binding = self.store.task_binding(&plan.task_id)?;
            binding.reply_to_task_id = Some(plan.task_id.clone());
            let request = format!(
                "Revise existing plan {} according to this owner instruction: {}. Read its latest version with plan_manage get. Ask every unresolved owner requirement; use the current autonomy preference for implementation choices. Preserve valid completed milestones and inspect earlier effects before reassigning work. Apply plan_manage revise; do not create a replacement plan or execute milestones in this turn.",
                plan.id, instruction
            );
            let configure = self.configure_lock.lock().await;
            let edit = self.store.enqueue_bound(&request, origin, None, &binding)?;
            self.store.inherit_destination(&edit.id, &plan.task_id)?;
            if self.store.task_destination(&edit.id)?.is_none() {
                self.pin_destination(&edit.id, origin)?;
            }
            self.store.put("plan_edits", &edit.id, &plan.id)?;
            let previous = self.store.get::<String>("plan_edit_owner", &plan.id)?;
            self.store.put("plan_edit_owner", &plan.id, &edit.id)?;
            for previous in previous
                .into_iter()
                .chain(std::iter::once(plan.task_id.clone()))
            {
                let active = self.active.lock().await;
                if let Some(active) = active.get(&previous) {
                    active.cancel.cancel();
                } else if self.store.task(&previous)?.status == "queued" {
                    self.store.set_task_status(
                        &previous,
                        "cancelled",
                        Some("Superseded by a newer plan edit"),
                    )?;
                }
            }
            let (plan, workers) =
                self.store
                    .pause_plan_for_edit(&plan.id, &edit.id, &instruction)?;
            self.supersede_plan_reviews(&plan.id, Some(&edit.id))
                .await?;
            drop(configure);
            self.notify(Some(&plan.task_id), "Cambio recibido. Detengo el trabajo pendiente mientras preparo la nueva versión del plan; las acciones ya realizadas se conservan.")?;
            self.stop_plan_workers(&workers).await?;
            self.bus_wake.notify_waiters();
            self.wake.notify_one();
            return Ok(
                json!({"accepted":true,"task_id":edit.id,"plan_id":plan.id,"version":plan.version,"status":"paused","instruction":"The change was received; affected execution is paused while the revised plan is prepared."}),
            );
        }
        let active = self.active.lock().await;
        let task = self.store.task(target)?;
        if task.origin == "subagent"
            || !matches!(task.status.as_str(), "queued" | "running" | "waiting")
        {
            bail!("This task cannot be replanned; address its active plan or send a new request");
        }
        let mut updates = self
            .store
            .get::<Vec<String>>("task_updates", target)?
            .unwrap_or_default();
        if updates.len() >= 100 {
            bail!(
                "This task has reached its revision limit; start a new task with its saved context"
            );
        }
        updates.push(instruction.clone());
        self.store.put("task_updates", target, &updates)?;
        self.store.publish_message(
            Some(target),
            &fenix_core::Message::user(&instruction),
            false,
            origin,
        )?;
        self.store.publish_event(
            Some(target),
            "task.edit_requested",
            &json!({"instruction":instruction,"revision":updates.len()}),
        )?;
        if let Some(active) = active.get(target) {
            active.revision.cancel();
        }
        drop(active);
        self.notify(Some(target),"Cambio recibido. Replantearé la tarea conservando el historial de las acciones ya realizadas.")?;
        self.wake.notify_one();
        Ok(json!({"accepted":true,"task_id":target,"revision":updates.len()}))
    }

    pub(crate) fn check_plan_editor(&self, task: &str) -> Result<()> {
        if let Some(plan) = self.store.get::<String>("plan_edits", task)?
            && self
                .store
                .get::<String>("plan_edit_owner", &plan)?
                .as_deref()
                != Some(task)
        {
            bail!("A newer owner edit superseded this revision request");
        }
        if let Some(id) = self.store.get::<String>("review_edits", task)? {
            let review = self.store.plan_review(&id)?;
            let plan = self.store.plan(&review.plan_id)?;
            // A successfully applied response may finish its final chat message.
            if review.status != "passed"
                && !(matches!(review.status.as_str(), "superseded" | "blocked")
                    && self
                        .store
                        .get::<bool>("review_response_applied", task)?
                        .unwrap_or(false))
                && (review.status != "resolving" || plan.version != review.version)
            {
                bail!("The review resolution turn is no longer current");
            }
        }
        Ok(())
    }

    pub(crate) fn plan_views(&self) -> Result<Value> {
        Ok(json!(self.store.plans()?.into_iter().take(50).map(|plan| Ok(json!({
            "id":plan.id,"task_id":plan.task_id,"project_id":plan.project_id,"version":plan.version,"status":plan.status,"objective":truncate(&plan.objective,1000),"updated_at":plan.updated_at,"completion_audits":self.store.completion_audits(&plan.id)?.into_iter().filter(|a|a.version==plan.version).collect::<Vec<_>>(),"evidence_summary":self.completion_evidence_summary(&plan)?,
            "nodes":plan.nodes.iter().map(|node|json!({"id":node.spec.id,"objective":truncate(&node.spec.objective,600),"expected_output":truncate(&node.spec.expected_output,600),"depends_on":node.spec.depends_on,"status":node.status,"worker_id":node.worker_id})).collect::<Vec<_>>(),
            "reviews":self.store.plan_reviews(&plan.id)?.iter().filter(|r|r.version==plan.version).map(|r|json!({"id":r.id,"status":r.status,"round":r.round,"node_ids":r.node_ids,"reason":truncate(&r.reason,1000),"reviewers":r.assignments.iter().map(|a|json!({"role":a.role,"specialty":a.request.specialty,"worker_id":a.worker_id})).collect::<Vec<_>>() })).collect::<Vec<_>>()
        }))).collect::<Result<Vec<_>>>()?))
    }

    pub(crate) fn readable_plans(&self, task: &str) -> Result<Vec<Plan>> {
        Ok(self
            .store
            .plans()?
            .into_iter()
            .filter(|p| self.authorize_task_read(task, &p.task_id).is_ok())
            .collect())
    }

    pub async fn plan_manage(&self, task: &str, args: &Value) -> Result<Value> {
        if !self.review_tool_allowed(task, "plan_manage", args)? {
            bail!("This review task cannot perform that plan action");
        }
        if matches!(
            self.store.task(task)?.origin.as_str(),
            "subagent" | "subagent_report"
        ) {
            bail!("Only the master handling an owner request can manage plans");
        }
        let action = args["action"].as_str().context("Plan action required")?;
        if action == "list" {
            return Ok(json!(self.readable_plans(task)?));
        }
        if action == "steer" {
            let target = args["task_id"]
                .as_str()
                .context("Target task ID required")?;
            self.authorize_task_read(task, target)?;
            let instruction = args["instruction"]
                .as_str()
                .context("Change instruction required")?;
            return self
                .steer_task(target, instruction, &self.store.task(task)?.origin)
                .await;
        }
        let _lock = self.plan_lock.lock().await;
        self.check_plan_editor(task)?;
        if let Some(brief) = args.get("brief") {
            serde_json::from_value::<fenix_storage::PlanBrief>(brief.clone())?.validate()?;
        }
        let specs = || -> Result<Vec<PlanNodeSpec>> {
            serde_json::from_value(args["nodes"].clone())
                .context("Provide milestones as a structured nodes array")
        };
        if args["review_reason"]
            .as_str()
            .is_some_and(|r| r.len() > 4000)
        {
            bail!("Review reason exceeds 4000 bytes");
        }
        if matches!(action, "create" | "revise") {
            let nodes = specs()?;
            let review_nodes: Vec<String> =
                serde_json::from_value(args.get("review_nodes").cloned().unwrap_or(json!([])))?;
            if review_nodes.len() > 32
                || review_nodes
                    .iter()
                    .any(|id| !nodes.iter().any(|n| &n.id == id))
            {
                bail!("Review scope must contain current milestone IDs");
            }
        }
        if action == "create" {
            if self.store.get::<String>("plan_edits", task)?.is_some() {
                bail!("This turn edits an existing plan; use revise");
            }
            if let Some(plan) = self
                .store
                .plan_for_task(task)?
                .filter(|p| p.task_id == task)
            {
                return Ok(
                    json!({"plan":plan,"reviews":self.store.plan_reviews(&plan.id)?,"instruction":"This task already has a plan; use get then revise instead of recreating it."}),
                );
            }
            let plan = self.store.create_plan(
                task,
                args["objective"].as_str().context("Objective required")?,
                args["constraints"].as_str().unwrap_or(""),
                specs()?,
            )?;
            if args["audit_required"] == true {
                self.store
                    .put("completion_audit_required", &plan.id, &true)?;
            }
            if let Err(error) = self.begin_plan_review(&plan, args, None) {
                self.store.cancel_plan(&plan.id, plan.version, task)?;
                return Err(error);
            }
            self.bus_wake.notify_waiters();
            return Ok(
                json!({"plan":plan,"reviews":self.store.plan_reviews(&plan.id)?,"instruction":"Milestones run in the background once their applicable reviews pass. Do not execute them again yourself. Continue the conversation; completion and questions arrive asynchronously."}),
            );
        }
        let id = args["plan_id"].as_str().context("Plan ID required")?;
        if let Some(review) = self.store.get::<String>("review_edits", task)? {
            if self.store.plan_review(&review)?.plan_id != id {
                bail!("Resolution is restricted to its assigned plan");
            }
            if self
                .store
                .get::<bool>("review_response_applied", task)?
                .unwrap_or(false)
                && action != "get"
            {
                bail!("The single review response was already applied");
            }
        }
        for node in &self.store.plan(id)?.nodes {
            if let Some(worker) = &node.worker_id {
                self.store.reconcile_plan_worker(worker)?;
            }
        }
        let current = self.store.plan(id)?;
        self.authorize_task_read(task, &current.task_id)?;
        if action == "get" {
            let reviews = self.store.plan_reviews(id)?;
            let resolution_finding_ids = reviews
                .iter()
                .filter(|review| review.version == current.version)
                .map(|review| {
                    let finding_ids = review
                        .assignments
                        .iter()
                        .flat_map(|assignment| {
                            assignment.report.iter().flat_map(move |report| {
                                report.findings.iter().map(move |finding| {
                                    format!("{}/{}", assignment.role, finding.id)
                                })
                            })
                        })
                        .collect::<Vec<_>>();
                    json!({"review_id":review.id,"finding_ids":finding_ids})
                })
                .collect::<Vec<_>>();
            return Ok(json!({
                "plan":current, "revisions":self.store.plan_history(id)?, "reviews":reviews,
                "brief":self.store.get::<fenix_storage::PlanBrief>("plan_briefs",id)?.unwrap_or_default(),
                "completion_audits":self.store.completion_audits(id)?,
                "evidence_summary":self.completion_evidence_summary(&current)?,
                "resolution_finding_ids":resolution_finding_ids
            }));
        }
        let version = args["expected_version"]
            .as_u64()
            .context("Read the plan and supply expected_version")?;
        if current.version != version {
            bail!("Plan changed; read the current version");
        }
        if action == "resolve_review" {
            let review_id = args["review_id"].as_str().context("Review ID required")?;
            if self.store.plan_review(review_id)?.plan_id != id {
                bail!("Review belongs to a different plan");
            }
            let review = self.store.resolve_plan_review(
                review_id,
                task,
                serde_json::from_value(args["resolutions"].clone())?,
            )?;
            self.store.put("review_response_applied", task, &true)?;
            self.bus_wake.notify_waiters();
            return Ok(json!({"review":review}));
        }
        if action == "review" {
            if args["review_reason"]
                .as_str()
                .is_none_or(|r| r.trim().is_empty())
            {
                bail!("Provide the concrete reason for selective review");
            }
            let review = self.begin_plan_review(&current, args, None)?;
            self.bus_wake.notify_waiters();
            return Ok(json!({"review":review}));
        }
        if action == "cancel" {
            let workers = self.store.cancel_plan(id, version, task)?;
            self.supersede_plan_reviews(id, Some(task)).await?;
            self.stop_plan_workers(&workers).await?;
            self.bus_wake.notify_waiters();
            return Ok(json!({"plan":self.store.plan(id)?,"completed_effects_preserved":true}));
        }
        if action != "revise" {
            bail!("Unknown plan action")
        }
        let reason = args["instruction"]
            .as_str()
            .context("Explain the owner's change in instruction")?;
        let previous_review = self
            .store
            .get::<String>("review_edits", task)?
            .map(|r| self.store.plan_review(&r))
            .transpose()?;
        if let Some(review) = &previous_review {
            self.store.record_review_revision_response(
                &review.id,
                task,
                serde_json::from_value(args["resolutions"].clone())
                    .context("Provide evidence-backed resolutions for this revision")?,
            )?;
        }
        let (plan, workers) = self.store.revise_plan(
            id,
            version,
            task,
            fenix_storage::PlanRevision {
                objective: args["objective"].as_str().unwrap_or(&current.objective),
                constraints: args["constraints"].as_str().unwrap_or(&current.constraints),
                nodes: specs()?,
                reason: &self.redactor.text(reason),
            },
        )?;
        self.bus_wake.notify_waiters();
        self.stop_plan_workers(&workers).await?;
        self.stop_plan_workers(&self.store.superseded_plan_workers(id)?)
            .await?;
        let plan = self.store.activate_plan_revision(id, plan.version)?;
        if args["audit_required"] == true {
            self.store.put("completion_audit_required", id, &true)?;
        }
        self.supersede_plan_reviews(id, Some(task)).await?;
        if let Err(error) = self.begin_plan_review(&plan, args, previous_review.as_ref()) {
            let (_, workers) = self.store.pause_plan_for_edit(
                id,
                task,
                "Review admission failed; inspect before dispatch",
            )?;
            self.stop_plan_workers(&workers).await?;
            return Err(error);
        }
        if previous_review.is_some() {
            self.store.put("review_response_applied", task, &true)?;
        }
        self.bus_wake.notify_waiters();
        Ok(
            json!({"plan":plan,"applied":true,"cancelled_workers":workers,"completed_effects_preserved":true}),
        )
    }

    pub(crate) async fn stop_plan_workers(&self, workers: &[String]) -> Result<()> {
        for id in workers {
            let active = self.subagents.active.lock().await;
            if let Some(cancel) = active.get(id) {
                cancel.cancel();
            } else if self.store.subagent(id)?.status == "queued" {
                self.store.finish_subagent(
                    id,
                    "cancelled",
                    &json!({"status":"cancelled","summary":"Milestone superseded before dispatch"}),
                )?;
            }
            drop(active);
            self.settle_task(id).await;
            self.clear_pending(id).await;
        }
        Ok(())
    }

    pub(crate) async fn planning_worker(self: Arc<Self>) {
        loop {
            if self.stop.is_cancelled() {
                break;
            }
            if let Err(error) = self.planning_cycle().await {
                eprintln!(
                    "Planning scheduler: {}",
                    self.redactor.text(&error.to_string())
                );
            }
            tokio::select! {_=self.stop.cancelled()=>break,_=self.bus_wake.notified()=>{},_=tokio::time::sleep(Duration::from_millis(250))=>{}}
        }
    }

    async fn planning_cycle(&self) -> Result<()> {
        let _lock = self.plan_lock.lock().await;
        for plan in self
            .store
            .plans()?
            .into_iter()
            .filter(|p| matches!(p.status.as_str(), "running" | "blocked"))
        {
            for node in &plan.nodes {
                if let Some(worker) = &node.worker_id {
                    self.store.reconcile_plan_worker(worker)?;
                }
            }
            let plan = self.store.plan(&plan.id)?;
            self.completion_audit_cycle(&plan).await?;
            let plan = self.store.plan(&plan.id)?;
            self.plan_review_cycle(&plan).await?;
            if plan.status != "running" {
                continue;
            }
            for node in &plan.nodes {
                if node.status != "pending"
                    || self.store.review_blocks_node(&plan, &node.spec.id)?
                    || !node.spec.depends_on.iter().all(|d| {
                        plan.nodes
                            .iter()
                            .any(|p| p.spec.id == *d && p.status == "completed")
                    })
                {
                    continue;
                }
                let prerequisites = plan
                    .nodes
                    .iter()
                    .filter(|p| node.spec.depends_on.contains(&p.spec.id))
                    .map(|n| json!({"id":n.spec.id,"result":n.result,"worker_id":n.worker_id}))
                    .collect::<Vec<_>>();
                let context = json!({"plan_id":plan.id,"plan_version":plan.version,"milestone":node.spec.id,"generation":node.generation,"global_objective":plan.objective,"prerequisites":prerequisites,"previous_execution":node.result,"suggested_skills":node.spec.skills,"instruction":"Load relevant enabled skills on demand. Preserve prior effects. Always ask unresolved owner requirements; resolve implementation details according to the runtime autonomy preference. Report blockers instead of creating another subagent."});
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
                let spec = SubagentSpec {
                    profile: "general".into(),
                    objective: node.spec.objective.clone(),
                    expected_output: node.spec.expected_output.clone(),
                    context: truncate(&context.to_string(), 24000),
                    constraints: plan.constraints.clone(),
                };
                // Claim and pin are atomic relative to the local worker dispatcher.
                let _active = self.subagents.active.lock().await;
                let job = self.store.create_subagent_in_project(
                    &plan.task_id,
                    &spec,
                    plan.project_id.as_deref(),
                )?;
                if let Err(error) =
                    self.store
                        .attach_plan_worker(&plan.id, plan.version, &node.spec.id, &job.id)
                {
                    self.store.finish_subagent(&job.id,"cancelled",&json!({"status":"cancelled","summary":"Plan changed before worker dispatch"}))?;
                    return Err(error);
                }
                self.store.inherit_destination(&job.id, &plan.task_id)?;
                self.subagents.wake.notify_one();
            }
            self.store.settle_plan(&plan.id)?;
        }
        Ok(())
    }

    pub(crate) async fn event_worker(self: Arc<Self>) {
        loop {
            if self.stop.is_cancelled() {
                break;
            }
            if let Err(error) = self.deliver_events() {
                eprintln!("Event delivery: {}", self.redactor.text(&error.to_string()));
            }
            tokio::select! {_=self.stop.cancelled()=>break,_=self.bus_wake.notified()=>{},_=tokio::time::sleep(Duration::from_millis(150))=>{}}
        }
    }

    pub fn deliver_events(&self) -> Result<()> {
        for event in self
            .store
            .bus_events(self.store.subscriber_cursor("channel_delivery")?, 100)?
        {
            if event.topic == "message.created"
                && event.data["role"] == "assistant"
                && event.data["destination"][0] == "telegram"
                && let Some(address) = event.data["destination"][1].as_str()
            {
                self.store.queue_event_delivery(
                    &event,
                    address,
                    event.data["text"].as_str().unwrap_or(""),
                )?;
            }
            self.store
                .acknowledge_event("channel_delivery", event.seq)?;
        }
        for event in self
            .store
            .bus_events(self.store.subscriber_cursor("plan_notifications")?, 100)?
        {
            if event.topic == "plan.audit_updated" {
                let audit = &event.data["audit"];
                let label = match audit["status"].as_str() {
                    Some("queued") => {
                        Some("El trabajo está listo para comprobar que cumple lo que pediste")
                    }
                    Some("passed") => Some("He comprobado el resultado"),
                    Some("blocked") => Some(
                        "Encontré puntos que necesito resolver antes de dar el trabajo por terminado",
                    ),
                    _ => None,
                };
                if let Some(label) = label {
                    self.publish_once(
                        &event.id,
                        event.task_id.as_deref(),
                        &format!(
                            "{label}: {}.\n{}",
                            self.plan_display_name(
                                audit["plan_id"].as_str(),
                                audit["node_id"].as_str()
                            ),
                            truncate(audit["report"]["summary"].as_str().unwrap_or(""), 2000)
                        ),
                    )?;
                }
            }
            if event.topic == "plan.review_updated"
                && matches!(
                    event.data["status"].as_str(),
                    Some("queued" | "passed" | "blocked" | "resolving")
                )
            {
                let label = match event.data["status"].as_str().unwrap_or("") {
                    "queued" => "Voy a revisar el plan",
                    "passed" => "He revisado el plan",
                    "resolving" => "Estoy ajustando el plan",
                    _ => "Necesito aclarar algunos puntos del plan antes de continuar",
                };
                let mut text = format!(
                    "{label}: {}.\n{}",
                    self.plan_display_name(event.data["plan_id"].as_str(), None),
                    event.data["reason"].as_str().unwrap_or("")
                );
                if event.data["status"] == "blocked" {
                    let review = self.store.plan_review(
                        event.data["review_id"]
                            .as_str()
                            .context("Review event is missing its ID")?,
                    )?;
                    for finding in review
                        .assignments
                        .iter()
                        .flat_map(|a| a.report.iter().flat_map(|r| r.findings.iter()))
                        .filter(|f| f.severity == "blocking")
                        .take(6)
                    {
                        text.push_str(&format!(
                            "\n• {} — {}",
                            truncate(&finding.description, 500),
                            truncate(&finding.evidence, 300)
                        ));
                    }
                    text.push_str(
                        "\nPuedes responder aquí o decirme qué quieres cambiar del plan.",
                    );
                }
                self.publish_once(&event.id, event.task_id.as_deref(), &text)?;
            }
            if matches!(
                event.topic.as_str(),
                "plan.created"
                    | "plan.updated"
                    | "plan.finished"
                    | "plan.cancelled"
                    | "plan.edit_requested"
            ) {
                let plan: Plan = serde_json::from_value(event.data["plan"].clone())?;
                let complete = plan
                    .nodes
                    .iter()
                    .filter(|n| n.status == "completed")
                    .count();
                let mut text = format!(
                    "{}: {}.\nPasos completados: {}/{}",
                    plan.objective,
                    progress_description(&plan.status),
                    complete,
                    plan.nodes.len()
                );
                if matches!(plan.status.as_str(), "completed" | "blocked") {
                    for node in &plan.nodes {
                        text.push_str(&format!(
                            "\n• {}: {}{}",
                            node.spec.objective,
                            progress_description(&node.status),
                            node.result
                                .as_ref()
                                .and_then(|r| r["summary"].as_str())
                                .map(|s| format!(" — {}", truncate(s, 800)))
                                .unwrap_or_default()
                        ));
                    }
                }
                // Stable event ID makes this handler safe to retry between publish and ACK.
                self.publish_once(&event.id, Some(&plan.task_id), &text)?;
            }
            self.store
                .acknowledge_event("plan_notifications", event.seq)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod communication_tests {
    use super::*;

    #[tokio::test]
    async fn plan_notifications_use_objectives_and_keep_routing_ids_internal() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
            .await
            .unwrap();
        let task = runtime
            .store
            .enqueue_bound(
                "Prepare the monthly report",
                "web",
                None,
                &fenix_storage::TaskBinding::default(),
            )
            .unwrap();
        let plan = runtime
            .store
            .create_plan(
                &task.id,
                "Informe mensual",
                "",
                vec![PlanNodeSpec {
                    id: "internal-report-node".into(),
                    profile: "general".into(),
                    objective: "Preparar el informe".into(),
                    expected_output: "report.md".into(),
                    depends_on: vec![],
                    skills: vec![],
                }],
            )
            .unwrap();
        for (topic, data) in [
            (
                "plan.review_updated",
                json!({"status":"passed","plan_id":plan.id,"plan_version":1,"reason":"El plan cubre lo solicitado."}),
            ),
            (
                "plan.audit_updated",
                json!({"audit":{"status":"queued","plan_id":plan.id,"node_id":"internal-report-node"}}),
            ),
        ] {
            runtime
                .store
                .publish_event(Some(&task.id), topic, &data)
                .unwrap();
        }
        runtime.deliver_events().unwrap();
        let messages = runtime.store.conversation(None, 50).unwrap();
        let replies: Vec<_> = messages
            .iter()
            .filter(|m| m.message.role == fenix_core::Role::Assistant)
            .collect();
        assert_eq!(replies.len(), 3);
        for reply in &replies {
            assert!(reply.text.contains("Informe mensual"));
            assert!(!reply.text.contains(&plan.id));
            assert!(!reply.text.contains(&task.id));
            assert!(!reply.text.contains("internal-report-node"));
        }
        assert!(
            replies
                .iter()
                .any(|reply| reply.text.contains("Preparar el informe"))
        );
        runtime.deliver_events().unwrap();
        assert_eq!(
            runtime.store.conversation(None, 50).unwrap().len(),
            messages.len()
        );
        assert_eq!(runtime.store.plan(&plan.id).unwrap().task_id, task.id);
    }
}
