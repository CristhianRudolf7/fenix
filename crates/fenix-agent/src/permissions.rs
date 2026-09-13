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
            kind: kind.into(),
            question: question.clone(),
            action,
        };
        let (send, receive) = oneshot::channel();
        self.report_progress(task, crate::progress::Phase::Waiting, None);
        *self.pending.lock().await = Some(Pending {
            view: view.clone(),
            reply: send,
        });
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
        self.notify(Some(task), &format!("{question}{details}\n\n{instruction}"))?;
        let result = tokio::select! {
            _=cancel.cancelled()=>Err(anyhow::anyhow!("Task cancelled")),
            _=self.stop.cancelled()=>Err(anyhow::anyhow!("Service stopping")),
            _=tokio::time::sleep(std::time::Duration::from_secs(1800))=>Err(anyhow::anyhow!("Timed out waiting for the owner")),
            reply=receive=>reply.map_err(|_|anyhow::anyhow!("Pending request closed")),
        };
        *self.pending.lock().await = None;
        self.store.set_task_status(task, "running", None)?;
        result
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
            let active = self.active.lock().await;
            let requested = words
                .get(1)
                .copied()
                .or_else(|| active.as_ref().map(|a| a.task_id.as_str()));
            let Some(id) = requested else {
                bail!("There is no active task. Use cancelar followed by a queued task ID");
            };
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
        let Some(current) = pending.as_ref() else {
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
            if let Some(pending) = pending.take() {
                let _ = pending.reply.send(answer);
            }
            Ok(Some(json!({"answered":view.id})))
        } else {
            // Unknown text does not authorize anything. It remains a separate FIFO request.
            self.notify(Some(&view.task_id),&format!("La solicitud {} sigue pendiente. Tu nuevo mensaje se encolará; para resolverla, usa la respuesta indicada arriba.",view.id))?;
            Ok(None)
        }
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
