use crate::Runtime;
use anyhow::{Result, bail};
use fenix_core::{Message, Role};
use serde::Serialize;
use serde_json::{Value, json};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

pub(crate) struct ActiveTask {
    pub cancel: CancellationToken,
    pub revision: CancellationToken,
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
        if kind == "clarification"
            && let Some(answer) = self.answered_question(task, question)?
        {
            self.store.event(
                task,
                "clarification_reused",
                &json!({"question":question,"source":"explicit_user_reply"}),
            )?;
            return Ok(answer);
        }
        if self.autonomy_context(task)?["interactive"] == false {
            bail!(
                "noninteractive_input_unavailable: Interactive approval or clarification is unavailable in this evaluation. Do not grant permission or invent an answer. Report blocked using stop_reason=owner_input_unavailable and this call as blocker_call_id."
            );
        }
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
        self.browser.task_waiting(task, true).await;
        self.store.event(task, "question", &json!(view))?;
        self.store.save_question(&json!(view))?;
        self.bus_wake.notify_waiters();
        let instruction = if kind == "approval" {
            "Puedes confirmar o rechazar esta acción con tus propias palabras."
        } else {
            "Puedes responder aquí con tus propias palabras."
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
            reply=receive=>reply.map_err(|_|anyhow::anyhow!("Pending request closed")),
        };
        self.browser.task_waiting(task, false).await;
        self.pending.lock().await.remove(&request_id);
        if result.is_err() {
            self.store
                .resolve_question(&request_id, "cancelled", None)?;
        }
        if result.is_ok() {
            self.store.set_task_status(task, "running", None)?;
        }
        if kind == "clarification"
            && let Ok(answer) = &result
        {
            self.contract_answer(task, &question, answer)?;
        }
        result
    }

    pub(crate) async fn clear_pending(&self, task: &str) {
        let mut pending = self.pending.lock().await;
        let ids = pending
            .values()
            .filter(|p| p.view.task_id == task)
            .map(|p| p.view.id.clone())
            .collect::<Vec<_>>();
        for id in ids {
            let _ = self.store.resolve_question(&id, "cancelled", None);
            pending.remove(&id);
        }
        // Also close durable questions whose in-memory sender no longer exists.
        if let Ok(questions) = self.store.saved_questions(Some(task)) {
            for question in questions {
                if question["status"] == "pending"
                    && let Some(id) = question["id"].as_str()
                {
                    let _ = self.store.resolve_question(id, "cancelled", None);
                }
            }
        }
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
                "¿Me das permiso para hacer lo siguiente?",
                Some(action),
                cancel,
            )
            .await?;
        Ok(answer == "approved")
    }

    pub(crate) async fn handle_control(&self, text: &str, origin: &str) -> Result<Option<Value>> {
        self.handle_control_text(text, text, origin).await
    }

    pub(crate) async fn handle_control_text(
        &self,
        text: &str,
        user_text: &str,
        origin: &str,
    ) -> Result<Option<Value>> {
        let lowered = text.trim().to_lowercase();
        let words = lowered.split_whitespace().collect::<Vec<_>>();
        if words
            .first()
            .is_some_and(|s| matches!(*s, "cancelar" | "cancel" | "/cancel"))
            && words.len() <= 2
        {
            let requested = match words.get(1) {
                Some(id) => Some((*id).to_owned()),
                None => {
                    let active = self.active.lock().await;
                    if active.len() == 1 {
                        active.keys().next().cloned()
                    } else {
                        None
                    }
                }
            };
            let Some(id) = requested.as_deref() else {
                return Ok(None);
            };
            let Ok(task) = self.store.task(id) else {
                return Ok(None);
            };
            if task.origin == "subagent" {
                self.store
                    .append(Some(id), &Message::user(user_text), true, false, origin)?;
                let result = self.cancel_subagent(id).await?;
                self.notify(Some(id), "He pedido que se detenga este trabajo. Los cambios ya realizados se conservan.")?;
                return Ok(Some(result));
            }
            let active = self.active.lock().await;
            let task = self.store.task(id)?;
            if !matches!(task.status.as_str(), "queued" | "running" | "waiting") {
                bail!("This task has already ended");
            }
            self.store.append(
                Some(id),
                &Message::text(Role::User, user_text),
                true,
                false,
                origin,
            )?;
            self.store
                .event(id, "cancellation_requested", &json!({"origin":origin}))?;
            if let Some(active) = active.get(id) {
                active.cancel.cancel();
            } else {
                self.store
                    .set_task_status(id, "cancelled", Some("Cancelled by the owner"))?;
            }
            drop(active);
            self.browser.task_finished(id).await;
            self.clear_pending(id).await;
            self.bus_wake.notify_waiters();
            self.notify(
                Some(id),
                "He pedido que se detenga este trabajo. Los cambios ya realizados se conservan.",
            )?;
            return Ok(Some(json!({"cancelled":id})));
        }
        let mut pending = self.pending.lock().await;
        if pending.is_empty() {
            if matches!(
                words.first().copied(),
                Some("aprobar" | "approve" | "rechazar" | "reject" | "responder" | "reply")
            ) {
                bail!("That request is no longer pending; no answer or approval was applied");
            }
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
                return Ok(None);
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
            self.check_plan_editor(&view.task_id)?;
            self.store.check_plan_worker(&view.task_id)?;
            self.store.append(
                Some(&view.task_id),
                &Message::text(Role::User, user_text),
                true,
                false,
                origin,
            )?;
            self.store.event(
                &view.task_id,
                "question_answered",
                &json!({"request_id":view.id,"kind":view.kind,"answer":answer,"origin":origin}),
            )?;
            self.store
                .resolve_question(&view.id, "answered", Some(&answer))?;
            self.bus_wake.notify_waiters();
            if let Some(pending) = pending.remove(&view.id) {
                let _ = pending.reply.send(answer);
            }
            Ok(Some(json!({"answered":view.id})))
        } else {
            // Unknown text does not authorize anything. It remains a separate FIFO request.
            Ok(None)
        }
    }

    /// Resolve an interpreted user reply against a still-pending request. IDs
    /// remain internal; the conversation stores only the user's original words.
    pub(crate) async fn answer_pending(
        &self,
        request_id: &str,
        text: &str,
        approval: Option<bool>,
        origin: &str,
        reference: Option<&str>,
    ) -> Result<Value> {
        let view = self
            .pending
            .lock()
            .await
            .get(request_id)
            .map(|p| p.view.clone());
        let Some(view) = view else {
            bail!("Esta pregunta ya no está pendiente. No se aplicó ninguna confirmación.");
        };
        if reference.is_some_and(|id| id != view.task_id) {
            bail!("La respuesta corresponde a otra tarea. Indica cuál quieres continuar.");
        }
        if text.trim().is_empty() {
            bail!("Escribe tu respuesta para continuar.");
        }
        let command = if view.kind == "approval" {
            let Some(approved) = approval else {
                bail!("Confirma si quieres que realice esta acción o que la rechace.");
            };
            format!(
                "{} {}",
                if approved { "approve" } else { "reject" },
                view.id
            )
        } else {
            format!("responder {} {}", view.id, text.trim())
        };
        self.handle_control_text(&command, text, origin)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("La pregunta ya no está disponible. No se aplicó la respuesta.")
            })
    }

    /// Native replies bind a short answer to its original pending task, even if
    /// another project has become the foreground conversation in the meantime.
    pub(crate) async fn handle_referenced_control(
        &self,
        text: &str,
        origin: &str,
        reply_to_task_id: Option<&str>,
    ) -> Result<Option<Value>> {
        // Recognize unambiguous short stop requests before treating a reply as
        // an answer to a pending clarification. Never guess between multiple tasks.
        let natural_stop = matches!(
            text.trim()
                .trim_end_matches(['.', '!'])
                .to_lowercase()
                .as_str(),
            "deserta la tarea"
                | "cancela la tarea"
                | "cancela eso"
                | "detén la tarea"
                | "deten la tarea"
                | "detén eso"
                | "cancel the task"
                | "stop the task"
        );
        if natural_stop {
            let candidates = self
                .store
                .tasks(200)?
                .into_iter()
                .filter(|task| {
                    task.origin != "subagent"
                        && matches!(task.status.as_str(), "queued" | "running" | "waiting")
                })
                .collect::<Vec<_>>();
            let target = reply_to_task_id
                .or_else(|| (candidates.len() == 1).then(|| candidates[0].id.as_str()));
            if let Some(id) = target {
                return self
                    .handle_control_text(&format!("cancelar {id}"), text, origin)
                    .await;
            }
            return Ok(None);
        }
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
                        .handle_control_text(&format!("{} {}", text.trim(), view.id), text, origin)
                        .await;
                }
            } else {
                return self
                    .handle_control_text(
                        &format!("responder {} {}", view.id, text.trim()),
                        text,
                        origin,
                    )
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
        Some("library_export") => format!("Copiar documento de biblioteca a {}", text("path")),
        Some("document_create") => {
            format!("Crear documento {}", args["path"].as_str().unwrap_or(""))
        }
        Some("document_edit") => format!(
            "Editar documento {} ({})",
            args["path"].as_str().unwrap_or(""),
            args["operation"]["action"].as_str().unwrap_or("")
        ),
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
        Some("file_patch") => format!("\n\nAplicar parche:\n{}", text("patch")),
        Some("command_run") => format!(
            "\n\nEjecutar este comando en {}:\n\n```sh\n{}\n```{}",
            text("cwd"),
            text("command"),
            if args["persistent"] == true {
                "\nSeguirá funcionando en segundo plano después de esta tarea."
            } else {
                ""
            }
        ),
        Some("command_input") => format!(
            "\n\nEnviar lo siguiente al programa que está en ejecución:\n\n```text\n{}\n```{}{}",
            text("text"),
            if args["eof"] == true {
                "\nDespués cerraré su entrada de datos."
            } else {
                ""
            },
            if args["rows"].is_number() || args["columns"].is_number() {
                "\nTambién ajustaré el tamaño de su ventana."
            } else {
                ""
            }
        ),
        _ => format!("\n\n{}", action),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn natural_answers_resolve_pending_questions_without_exposing_ids() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
            .await
            .unwrap();
        for (kind, words, approval, expected) in [
            ("approval", "Sí, hazlo por favor", Some(true), "approved"),
            ("approval", "No, no lo hagas", Some(false), "denied"),
            (
                "clarification",
                "Elige tú el tema y usa un enlace temporal",
                None,
                "Elige tú el tema y usa un enlace temporal",
            ),
        ] {
            let task = runtime
                .store
                .enqueue("Build a landing page", "web", None)
                .unwrap();
            let cloned = runtime.clone();
            let task_id = task.id.clone();
            let answer = tokio::spawn(async move {
                cloned
                    .ask(
                        &task_id,
                        kind,
                        "¿Cómo quieres continuar?",
                        None,
                        &CancellationToken::new(),
                    )
                    .await
            });
            let view = tokio::time::timeout(Duration::from_secs(2), async {
                loop {
                    if let Some(view) = runtime
                        .pending
                        .lock()
                        .await
                        .values()
                        .next()
                        .map(|p| p.view.clone())
                    {
                        break view;
                    }
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
            let unrelated = runtime
                .store
                .enqueue("Unrelated task", "web", None)
                .unwrap();
            assert!(
                runtime
                    .answer_pending(&view.id, words, approval, "web", Some(&unrelated.id))
                    .await
                    .is_err()
            );
            if kind == "approval" {
                assert!(
                    runtime
                        .answer_pending(&view.id, "¿Es seguro?", None, "web", None)
                        .await
                        .is_err()
                );
            }
            let messages = runtime.store.conversation(None, 20).unwrap();
            assert!(messages.iter().all(|m| !m.text.contains(&view.id)));
            runtime
                .answer_pending(&view.id, words, approval, "web", None)
                .await
                .unwrap();
            assert_eq!(answer.await.unwrap().unwrap(), expected);
            assert!(runtime.pending.lock().await.is_empty());
            let messages = runtime.store.conversation(None, 20).unwrap();
            assert!(messages.iter().any(|m| m.text == words));
            assert!(messages.iter().all(|m| !m.text.contains(&view.id)));
            assert!(
                runtime
                    .answer_pending(&view.id, words, approval, "web", None)
                    .await
                    .is_err()
            );
        }
    }

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
        assert!(
            runtime
                .handle_control("yes", "web")
                .await
                .unwrap()
                .is_none()
        );
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
