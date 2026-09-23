//! Resume only complete tool boundaries. Uncertain effects never get replayed.
use crate::Runtime;
use anyhow::{Result, ensure};
use fenix_core::{ContentBlock, Message};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Serialize, Deserialize)]
pub(crate) struct Boundary {
    status: String,
    messages: Vec<Message>,
}
fn complete(messages: &[Message]) -> bool {
    let mut calls = BTreeSet::new();
    for message in messages {
        for block in &message.content {
            match block {
                ContentBlock::ToolCall { id, .. } => {
                    if !calls.insert(id) {
                        return false;
                    }
                }
                ContentBlock::ToolResult { tool_call_id, .. } => {
                    if !calls.remove(tool_call_id) {
                        return false;
                    }
                }
                _ => {}
            }
        }
    }
    calls.is_empty()
}
impl Runtime {
    pub(crate) fn recovery_boundary(
        &self,
        task: &str,
        messages: &[Message],
        safe: bool,
    ) -> Result<()> {
        let safe = safe
            && complete(messages)
            && self.store.get::<bool>("restart_uncertain_effects", task)? != Some(true);
        self.store.put(
            "restart_boundaries",
            task,
            &Boundary {
                status: if safe { "ready" } else { "uncertain" }.into(),
                messages: if safe { messages.to_vec() } else { vec![] },
            },
        )
    }
    pub(crate) fn recovered_messages(&self, task: &str) -> Result<Option<Vec<Message>>> {
        if self.store.get::<bool>("restart_admitted", task)? != Some(true) {
            return Ok(None);
        }
        let boundary = self.store.get::<Boundary>("restart_boundaries", task)?;
        let Some(boundary) = boundary else {
            return Ok(None);
        };
        ensure!(
            boundary.status == "ready" && complete(&boundary.messages),
            "Recovery boundary is no longer valid"
        );
        self.store.delete("restart_admitted", task)?;
        let mut messages = boundary.messages;
        messages.push(Message::user("The service restarted after a complete tool boundary. Continue this SAME task and scope. Prior tool effects are already completed; do not replay them. Recheck mutable external/file state before dependent writes. Previous process handles are unavailable. Repository guidance must be refreshed before writes."));
        Ok(Some(messages))
    }
    pub(crate) fn recover_restart(&self) -> Result<(usize, usize)> {
        let mut after = 0;
        let mut previous = Vec::new();
        loop {
            let page = self.store.tasks_page(after, 200)?;
            if page.is_empty() {
                break;
            }
            for (seq, task) in page {
                after = seq;
                if matches!(
                    task.status.as_str(),
                    "queued" | "running" | "waiting" | "interrupted"
                ) {
                    previous.push(task)
                }
            }
        }
        let plans = self.store.plans()?;
        let mut eligible = BTreeMap::new();
        for task in &previous {
            let tries = self
                .store
                .get::<u32>("restart_attempts", &task.id)?
                .unwrap_or(0);
            let boundary = self.store.get::<Boundary>("restart_boundaries", &task.id)?;
            let ready = (task.status == "queued" && boundary.is_none())
                || boundary
                    .as_ref()
                    .is_some_and(|b| b.status == "ready" && complete(&b.messages));
            let exceptional = self
                .store
                .get::<serde_json::Value>("plan_edits", &task.id)?
                .is_some()
                || self.store.review_for_worker(&task.id)?.is_some()
                || self.store.completion_audit_for_worker(&task.id)?.is_some();
            if ready && task.status != "waiting" && tries < 3 && !exceptional {
                eligible.insert(task.id.clone(), tries);
            }
        }
        // Resume a plan only when every active worker can continue. Paused/revising plans remain paused for review.
        let mut restored_plans = Vec::new();
        for plan in plans {
            let relevant: Vec<_> = previous
                .iter()
                .filter(|t| {
                    self.store
                        .plan_for_task(&t.id)
                        .ok()
                        .flatten()
                        .is_some_and(|p| p.id == plan.id)
                })
                .map(|t| t.id.clone())
                .collect();
            if plan.status == "running" && relevant.iter().all(|id| eligible.contains_key(id)) {
                restored_plans.push(plan);
            } else {
                for id in relevant {
                    eligible.remove(&id);
                }
            }
        }
        let interrupted = self.store.interrupt_unfinished()?;
        for (id, tries) in &eligible {
            self.store.put("restart_attempts", id, &(tries + 1))?;
            // Queued tasks without a boundary build their first context normally.
            if self
                .store
                .get::<Boundary>("restart_boundaries", id)?
                .is_some_and(|b| b.status == "ready")
            {
                self.store.put("restart_admitted", id, &true)?;
            }
        }
        let ids = eligible.keys().cloned().collect::<Vec<_>>();
        let restored = self
            .store
            .restore_checkpointed_tasks(&ids, &restored_plans)?;
        for id in &ids {
            self.store.event(
                id,
                "task_recovered",
                &json!({"same_task_id":true,"effects_replayed":false,"attempt":eligible[id]+1}),
            )?;
        }
        Ok((interrupted, restored))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use fenix_core::{ContentBlock, Role};
    #[test]
    fn pending_calls_are_never_a_resume_boundary() {
        let mut messages = vec![Message {
            role: Role::Assistant,
            content: vec![ContentBlock::ToolCall {
                id: "write".into(),
                name: "file_write".into(),
                arguments: json!({}),
            }],
            provider_data: None,
        }];
        assert!(!complete(&messages));
        messages.push(Message {
            role: Role::Tool,
            content: vec![ContentBlock::ToolResult {
                tool_call_id: "write".into(),
                name: None,
                content: json!({}),
                is_error: false,
            }],
            provider_data: None,
        });
        assert!(complete(&messages));
    }
}
