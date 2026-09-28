//! Conversation-owned autonomy, independent of authorization and acceptance checks.
use crate::Runtime;
use anyhow::{Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Preference {
    pub mode: String,
    pub scope: String,
    pub source_task: String,
    pub source_quote: String,
}

/// Narrow contradiction check, not a general classifier of user intent.
pub(crate) fn declares_question_optional(text: &str) -> bool {
    let lower = text.to_lowercase();
    [
        "question was unnecessary",
        "question is unnecessary",
        "no blocking question is needed",
        "not an indispensable decision",
        "debo proceder",
        "pregunta fue innecesaria",
        "pregunta es innecesaria",
        "no hace falta bloquear",
        "no es una decisión indispensable",
        "corresponde continuar",
    ]
    .iter()
    .any(|phrase| lower.contains(phrase))
}

impl Runtime {
    fn autonomy_owner(&self, task: &str) -> Result<String> {
        if self.store.task(task)?.origin == "subagent" {
            return Ok(self.store.subagent(task)?.parent_task_id);
        }
        Ok(task.into())
    }

    pub(crate) fn autonomy_context(&self, task: &str) -> Result<Value> {
        let owner = self.autonomy_owner(task)?;
        let binding = self.store.task_binding(task)?;
        let mut selected = self
            .store
            .get::<Preference>("autonomy", &format!("task:{owner}"))?;
        if selected.is_none()
            && let Some(project) = binding.project_id
        {
            selected = self.store.get("autonomy", &format!("project:{project}"))?;
        }
        if selected.is_none() {
            selected = self.store.get("autonomy", "global")?;
        }
        // Ordinary evaluations default to no owner. An explicit interactive
        // transport can override that default without changing task provenance.
        let interactive = !self
            .store
            .get::<bool>("evaluation", "noninteractive")?
            .unwrap_or(self.store.task(&owner)?.origin == "evaluation");
        Ok(
            json!({"mode":if !interactive {"autonomous"} else {selected.as_ref().map_or("assisted", |p|p.mode.as_str())},
            "scope":selected.as_ref().map_or("default", |p|p.scope.as_str()),
            "interactive":interactive,"clarify_uncertain_requirements":true,
            "permissions_unchanged":true,"completion_checks_unchanged":true}),
        )
    }

    pub(crate) fn manage_autonomy(&self, task: &str, args: &Value) -> Result<Value> {
        if args["action"] == "get" {
            return self.autonomy_context(task);
        }
        ensure!(args["action"] == "set", "Unknown autonomy action");
        let owner = self.store.task(task)?;
        ensure!(
            matches!(owner.origin.as_str(), "web" | "telegram" | "live"),
            "Only a direct owner conversation can change autonomy preferences"
        );
        ensure!(
            self.store.get::<String>("plan_edits", task)?.is_none()
                && self.store.get::<String>("review_edits", task)?.is_none(),
            "Use the original owner conversation, not generated review instructions"
        );
        let quote = args["source_quote"].as_str().unwrap_or("").trim();
        ensure!(
            (8..=1000).contains(&quote.len()),
            "Cite the owner's explicit autonomy preference"
        );
        let message = self.store.task_user_message(task)?;
        let updates = self
            .store
            .get::<Vec<String>>("task_updates", task)?
            .unwrap_or_default();
        ensure!(
            message
                .as_ref()
                .is_some_and(|m| m.message.text_content().contains(quote))
                || updates.iter().any(|m| m.contains(quote)),
            "The preference must quote a direct owner message in this task, not a tool or document"
        );
        let mode = args["mode"].as_str().unwrap_or("");
        let scope = args["scope"].as_str().unwrap_or("task");
        ensure!(
            matches!(mode, "assisted" | "autonomous"),
            "Unknown autonomy mode"
        );
        // The existing model interprets intent. The runtime requires provenance and
        // an explicit durable scope, rather than promoting a task choice globally.
        let lower = quote.to_lowercase();
        if scope == "global" {
            ensure!(
                [
                    "siempre",
                    "a partir de ahora",
                    "de ahora en adelante",
                    "por defecto",
                    "todas las tareas",
                    "always",
                    "from now on",
                    "by default",
                    "all tasks"
                ]
                .iter()
                .any(|s| lower.contains(s)),
                "Global autonomy needs an explicit owner-wide preference; otherwise use task scope"
            );
        }
        let key = match scope {
            "task" => format!("task:{task}"),
            "project" => {
                ensure!(
                    ["proyecto", "project"].iter().any(|s| lower.contains(s)),
                    "Project scope must be explicit in the owner preference"
                );
                let project = self
                    .store
                    .task_binding(task)?
                    .project_id
                    .ok_or_else(|| anyhow::anyhow!("No fixed project for this task"))?;
                format!("project:{project}")
            }
            "global" => "global".into(),
            _ => bail!("Unknown autonomy scope"),
        };
        let preference = Preference {
            mode: mode.into(),
            scope: scope.into(),
            source_task: task.into(),
            source_quote: quote.into(),
        };
        self.store.put("autonomy", &key, &preference)?;
        self.store
            .event(task, "autonomy_changed", &json!(preference))?;
        self.autonomy_context(task)
    }

    pub(crate) fn check_clarification_policy(&self, task: &str, args: &Value) -> Result<()> {
        let category = args["category"].as_str().unwrap_or("requirement");
        if category == "requirement" && self.autonomy_context(task)?["interactive"] == false {
            let reason = args["blocking_reason"].as_str().unwrap_or("").trim();
            ensure!(
                !declares_question_optional(reason),
                "Your explanation says this question is unnecessary or work can proceed. Inspect the supplied interface/data and continue authorized work; this is not evidence of an unavailable owner prerequisite."
            );
            let input = args["blocking_input"].as_str().unwrap_or("");
            ensure!(
                matches!(input, "owner_decision" | "private_input" | "authorization"),
                "No interactive owner is available. Technical information (file formats, APIs, dependencies, algorithms) must first be investigated using supplied files and authorized public documentation. This failed question does not establish a blocker. Only indispensable owner_decision, private_input or authorization can qualify; name blocking_input, cite source_quote from the request and explain blocking_reason. Do not invent requirements or bypass permissions."
            );
            let quote = args["source_quote"].as_str().unwrap_or("").trim();
            let request = self.store.task(task)?.request;
            let updates = self
                .store
                .get::<Vec<String>>("task_updates", task)?
                .unwrap_or_default();
            ensure!(
                (8..=2000).contains(&quote.len())
                    && (request.contains(quote) || updates.iter().any(|text| text.contains(quote))),
                "An indispensable missing input must cite an exact source_quote from this task or an owner update, not a tool result or an invented requirement."
            );
            ensure!(
                (20..=1500).contains(&reason.len()),
                "Explain why the cited requested outcome needs this owner decision, private input or authorization and why inspecting available evidence cannot resolve it. Technical uncertainty alone is not a missing owner requirement."
            );
            bail!(
                "noninteractive_input_unavailable: Indispensable {input} for the cited owner requirement is unavailable. Do not invent it or expand authorization. Continue independent authorized work, or report the actual dependency with stop_reason=owner_input_unavailable and this call as blocker_call_id."
            );
        }
        self.check_question_policy(task, category)
    }

    pub(crate) fn check_question_policy(&self, task: &str, category: &str) -> Result<()> {
        ensure!(
            matches!(category, "requirement" | "implementation"),
            "Unknown clarification category"
        );
        let policy = self.autonomy_context(task)?;
        if policy["interactive"] == false {
            if category == "implementation" {
                bail!(
                    "Resolve this implementation choice using available evidence and bounded assumptions. This evaluation has no interactive owner; continue authorized work."
                );
            }
            bail!(
                "No interactive owner is available. A question delivery failure alone does not establish a missing requirement. Inspect available files and prior answers, then continue authorized work. Truly indispensable owner input must pass check_clarification_policy with a task-sourced dependency; never invent permissions or owner intent."
            );
        }
        if category == "implementation" && policy["mode"] == "autonomous" {
            bail!(
                "The owner delegated implementation choices. Inspect evidence, select a reversible in-scope option and record assumptions with task_contract annotate. Do not relabel technical choices as missing requirements. Real scope/intent questions must still be asked."
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
