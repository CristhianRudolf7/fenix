use crate::Runtime;
use anyhow::{Result, bail};
use fenix_core::{Message, Role};
use serde::Serialize;
use serde_json::{Value, json};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

pub(crate) struct ActiveTask {
    pub task_id: String,
    pub cancel: CancellationToken,
}
pub(crate) struct Pending {
    pub view: PendingView,
    pub reply: oneshot::Sender<String>,
}

#[derive(Clone, Serialize)]
pub struct PendingView {
    pub id: String,
    pub task_id: String,
    pub project_id: Option<String>,
    pub kind: String,
    pub question: String,
    pub action: Option<Value>,
}

impl Runtime {
    pub(crate) async fn ask(
        &self,
        task: &str,
        kind: &str,
        question: &str,
        action: Option<Value>,
        cancel: &CancellationToken,
    ) -> Result<String> {
        let request_id = fenix_storage::id();
        let question = self.redactor.text(question);
        let view = PendingView {
            id: request_id.clone(),
            task_id: task.into(),
            project_id: self.store.task_binding(task)?.project_id,
            kind: kind.into(),
            question: question.clone(),
            action,
        };
        let (send, receive) = oneshot::channel();
        self.report_progress(task, crate::progress::Phase::Waiting, None);
        self.pending.lock().await.insert(
            request_id.clone(),
            Pending {
                view: view.clone(),
                reply: send,
            },
        );
        self.store.set_task_status(task, "waiting", None)?;
        self.store.event(task, "question", &json!(view))?;
        let instruction = if kind == "approval" {
            format!("Responde «aprobar {request_id}» o «rechazar {request_id}».")
        } else {
            format!(
                "Responde «responder {request_id} tu respuesta». Los demás mensajes se encolan."
            )
        };
        let details = view
            .action
            .as_ref()
            .map(describe_action)
            .unwrap_or_default();
        let project_label = view
            .project_id
            .as_deref()
            .map(|id| {
                self.store
                    .project(id)
                    .map(|p| format!("Proyecto {} · ", p.name))
            })
            .transpose()?
            .unwrap_or_default();
        self.notify(
            Some(task),
            &format!("{project_label}{question}{details}\n\n{instruction}"),
        )?;
        let result = tokio::select! {
            _=cancel.cancelled()=>Err(anyhow::anyhow!("Task cancelled")),
            _=self.stop.cancelled()=>Err(anyhow::anyhow!("Service stopping")),
            _=tokio::time::sleep(std::time::Duration::from_secs(1800))=>Err(anyhow::anyhow!("Timed out waiting for the owner")),
            reply=receive=>reply.map_err(|_|anyhow::anyhow!("Pending request closed")),
        };
        self.pending.lock().await.remove(&request_id);
        self.store.set_task_status(task, "running", None)?;
        result
    }

    pub(crate) async fn clear_pending(&self, task: &str) {
        self.pending
            .lock()
            .await
            .retain(|_, pending| pending.view.task_id != task);
    }

    pub(crate) async fn approve(
        &self,
        task: &str,
        action: Value,
        cancel: &CancellationToken,
    ) -> Result<bool> {
        let answer = self
            .ask(
                task,
                "approval",
                "Necesito tu aprobación para ejecutar esta acción.",
                Some(action),
                cancel,
            )
            .await?;
        Ok(answer == "approved")
    }

    pub(crate) async fn handle_control(&self, text: &str, origin: &str) -> Result<Option<Value>> {
        let lowered = text.trim().to_lowercase();
        let words = lowered.split_whitespace().collect::<Vec<_>>();
        if words
            .first()
            .is_some_and(|s| matches!(*s, "cancelar" | "cancel" | "/cancel"))
            && words.len() <= 2
        {
            let requested = match words.get(1) {
                Some(id) => Some((*id).to_owned()),
                None => self.active.lock().await.as_ref().map(|a| a.task_id.clone()),
            };
            let Some(id) = requested.as_deref() else {
                bail!("There is no active master task. Use cancelar followed by a task ID");
            };
            if self.store.task(id)?.origin == "subagent" {
                self.store
                    .append(Some(id), &Message::user(text), true, false, origin)?;
                let result = self.cancel_subagent(id).await?;
                self.notify(Some(id), "Cancelación del subagente solicitada. Los efectos que ya ocurrieron no se revierten.")?;
                return Ok(Some(result));
            }
            let active = self.active.lock().await;
            let task = self.store.task(id)?;
            if !matches!(task.status.as_str(), "queued" | "running" | "waiting") {
                bail!("This task has already ended");
            }
            self.store.append(
                Some(id),
                &Message::text(Role::User, text),
                true,
                false,
                origin,
            )?;
            self.store
                .event(id, "cancellation_requested", &json!({"origin":origin}))?;
            if let Some(active) = active.as_ref().filter(|a| a.task_id == id) {
                active.cancel.cancel();
            } else {
                self.store
                    .set_task_status(id, "cancelled", Some("Cancelled by the owner"))?;
            }
            self.notify(
                Some(id),
                "Cancelación solicitada. Los efectos que ya ocurrieron no se revierten.",
            )?;
            return Ok(Some(json!({"cancelled":id})));
        }
        let mut pending = self.pending.lock().await;
        if pending.is_empty() {
            return Ok(None);
        }
        let is_named_control = matches!(
            words.first().copied(),
            Some("aprobar" | "approve" | "rechazar" | "reject" | "responder" | "reply")
        );
        let is_short_decision = matches!(words.first().copied(), Some("sí" | "si" | "yes" | "no"))
            && (words.len() == 1 || words.len() == 2 && pending.contains_key(words[1]));
        let is_control = is_named_control || is_short_decision;
        let current = if let Some(id) = words.get(1).filter(|_| is_control) {
            pending.get(*id)
        } else if pending.len() == 1 {
            pending.values().next()
        } else {
            None
        };
        let Some(current) = current else {
            if is_control {
                bail!("Include the exact pending request ID; no approval or reply was applied");
            }
            return Ok(None);
        };
        let view = current.view.clone();
        let decision = if view.kind == "approval" {
            let approved = matches!(
                words.first().copied(),
                Some("aprobar" | "approve" | "sí" | "si" | "yes")
            );
            let denied = matches!(words.first().copied(), Some("rechazar" | "reject" | "no"));
            if (approved || denied) && (words.len() == 1 || words.len() == 2 && words[1] == view.id)
            {
                Some(if approved {
                    "approved".to_owned()
                } else {
                    "denied".to_owned()
                })
            } else {
                None
            }
        } else {
            let mut pieces = text.splitn(3, char::is_whitespace);
            let prefix = pieces.next().unwrap_or_default().to_lowercase();
            let id = pieces.next().unwrap_or_default();
            let answer = pieces.next().unwrap_or_default().trim();
            if matches!(prefix.as_str(), "responder" | "reply")
                && id == view.id
                && !answer.is_empty()
            {
                Some(answer.to_owned())
            } else {
                None
            }
        };
        if let Some(answer) = decision {
            self.store.append(
                Some(&view.task_id),
                &Message::text(Role::User, text),
                true,
                false,
                origin,
            )?;
            self.store.event(
                &view.task_id,
                "question_answered",
                &json!({"request_id":view.id,"kind":view.kind,"answer":answer,"origin":origin}),
            )?;
            if let Some(pending) = pending.remove(&view.id) {
                let _ = pending.reply.send(answer);
            }
            Ok(Some(json!({"answered":view.id})))
        } else {
            // Unknown text does not authorize anything. It remains a separate FIFO request.
            self.notify(Some(&view.task_id),&format!("La solicitud {} sigue pendiente. Tu nuevo mensaje se encolará; para resolverla, usa la respuesta indicada arriba.",view.id))?;
            Ok(None)
        }
    }

    /// Native replies bind a short answer to its original pending task, even if
    /// another project has become the foreground conversation in the meantime.
    pub(crate) async fn handle_referenced_control(
        &self,
        text: &str,
        origin: &str,
        reply_to_task_id: Option<&str>,
    ) -> Result<Option<Value>> {
        let Some(task_id) = reply_to_task_id else {
            return self.handle_control(text, origin).await;
        };
        self.store.task(task_id)?;
        let prefix = text
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_lowercase();
        if matches!(prefix.as_str(), "cancelar" | "cancel" | "/cancel")
            && text.split_whitespace().count() == 1
        {
            return self
                .handle_control(&format!("cancelar {task_id}"), origin)
                .await;
        }
        if matches!(
            prefix.as_str(),
            "aprobar"
                | "approve"
                | "rechazar"
                | "reject"
                | "responder"
                | "reply"
                | "cancelar"
                | "cancel"
                | "/cancel"
        ) && text.split_whitespace().count() > 1
        {
            return self.handle_control(text, origin).await;
        }
        let candidates = self
            .pending
            .lock()
            .await
            .values()
            .filter(|pending| pending.view.task_id == task_id)
            .map(|pending| pending.view.clone())
            .collect::<Vec<_>>();
        if let [view] = candidates.as_slice() {
            if view.kind == "approval" {
                if matches!(
                    text.trim().to_lowercase().as_str(),
                    "sí" | "si" | "yes" | "no" | "aprobar" | "approve" | "rechazar" | "reject"
                ) {
                    return self
                        .handle_control(&format!("{} {}", text.trim(), view.id), origin)
                        .await;
                }
            } else {
                return self
                    .handle_control(&format!("responder {} {}", view.id, text.trim()), origin)
                    .await;
            }
        }
        // Never apply a reply to an unrelated pending request merely because it
        // happens to be the only one currently waiting.
        Ok(None)
    }
}

fn describe_action(action: &Value) -> String {
    let args = &action["arguments"];
    let text = |name: &str| args[name].as_str().unwrap_or_default();
    match action["tool"].as_str() {
        Some("file_write") => format!(
            "\n\nCrear archivo: {}\n\nContenido:\n{}",
            text("path"),
            text("content")
        ),
        Some("file_edit") => format!(
            "\n\nEditar archivo: {}\n\nTexto que debe coincidir:\n{}\n\nReemplazo:\n{}",
            text("path"),
            text("expected"),
            text("replacement")
        ),
        Some("command_run") => format!(
            "\n\nDirectorio: {}\nComando:\n{}\n\nTiempo máximo: {} segundos.",
            text("cwd"),
            text("command"),
            args["timeout_seconds"]
                .as_u64()
                .unwrap_or(120)
                .clamp(1, 600)
        ),
        _ => format!("\n\n{}", action),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn telegram_reply_resolves_original_request_without_approving_an_unrelated_task() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
            .await
            .unwrap();
        let pending_task = runtime
            .store
            .enqueue("Pending action", "web", None)
            .unwrap();
        let unrelated = runtime
            .store
            .enqueue("Another conversation", "web", None)
            .unwrap();
        let cloned = runtime.clone();
        let task_id = pending_task.id.clone();
        let answer = tokio::spawn(async move {
            cloned.approve(&task_id,json!({"tool":"file_write","arguments":{"path":"note.txt","content":"approved"}}),&CancellationToken::new()).await
        });
        tokio::time::timeout(Duration::from_secs(2), async {
            while runtime.pending.lock().await.is_empty() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(
            runtime
                .handle_referenced_control("sí", "web", Some(&unrelated.id))
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(runtime.pending.lock().await.len(), 1);
        runtime
            .store
            .put("telegram_messages", "fixture:7:40", &pending_task.id)
            .unwrap();
        let mut settings = crate::telegram::TelegramSettings {
            enabled: true,
            username: "fixture".into(),
            owner_id: Some(7),
            pairing_code: None,
            offset: 0,
            last_error: None,
        };
        runtime.receive_telegram(&json!({"message":{"message_id":50,"from":{"id":7},"chat":{"id":7,"type":"private"},"text":"sí","reply_to_message":{"message_id":40}}}),&mut settings).await.unwrap();
        assert!(answer.await.unwrap().unwrap());
        assert!(runtime.pending.lock().await.is_empty());
        assert_eq!(runtime.store.tasks(10).unwrap().len(), 2);
    }

    #[tokio::test]
    async fn concurrent_approvals_require_ids_and_resolve_independently() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
            .await
            .unwrap();
        let first = runtime.store.enqueue("first", "web", None).unwrap();
        let second = runtime.store.enqueue("second", "web", None).unwrap();
        let mut answers = Vec::new();
        for task in [&first, &second] {
            let runtime = runtime.clone();
            let id = task.id.clone();
            answers.push(tokio::spawn(async move {
                runtime
                    .approve(
                        &id,
                        json!({"tool":"file_write","arguments":{"path":id,"content":"approved"}}),
                        &CancellationToken::new(),
                    )
                    .await
            }));
        }
        let views = tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                let pending = runtime.status().await.unwrap()["pending_requests"]
                    .as_array()
                    .unwrap()
                    .clone();
                if pending.len() == 2 {
                    break pending;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(
            runtime
                .handle_control("no quiero cambiar el plan", "web")
                .await
                .unwrap()
                .is_none()
        );
        assert!(runtime.handle_control("yes", "web").await.is_err());
        assert_eq!(runtime.pending.lock().await.len(), 2);
        let first_id = views.iter().find(|v| v["task_id"] == first.id).unwrap()["id"]
            .as_str()
            .unwrap();
        let second_id = views.iter().find(|v| v["task_id"] == second.id).unwrap()["id"]
            .as_str()
            .unwrap();
        runtime
            .handle_control(&format!("reject {second_id}"), "telegram")
            .await
            .unwrap();
        assert!(!answers.pop().unwrap().await.unwrap().unwrap());
        assert_eq!(runtime.pending.lock().await.len(), 1);
        runtime
            .handle_control(&format!("approve {first_id}"), "web")
            .await
            .unwrap();
        assert!(answers.pop().unwrap().await.unwrap().unwrap());
        assert!(runtime.pending.lock().await.is_empty());
    }
}
