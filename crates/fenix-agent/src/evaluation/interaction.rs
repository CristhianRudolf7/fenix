//! Transport only: benchmark notifications enter the real runtime; grading stays outside.
use crate::Runtime;
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{UnixStream, unix::OwnedWriteHalf},
    sync::mpsc,
};

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Incoming {
    Message {
        run_id: String,
        text: String,
        #[serde(default)]
        notification: bool,
    },
    Stop,
}

pub(super) struct Session {
    writer: OwnedWriteHalf,
    incoming: mpsc::Receiver<Result<Incoming>>,
    reader: tokio::task::JoinHandle<()>,
    prefix: String,
    initialized: bool,
    runs: HashSet<String>,
    tasks: HashMap<String, String>,
    questions: HashSet<String>,
    reply_to: Option<String>,
}
impl Drop for Session {
    fn drop(&mut self) {
        self.reader.abort();
    }
}
impl Session {
    pub async fn connect(path: &Path, prefix: &str) -> Result<Self> {
        let stream = UnixStream::connect(path)
            .await
            .context("Cannot connect evaluation transport")?;
        let (read, writer) = stream.into_split();
        let (send, incoming) = mpsc::channel(32);
        let reader = tokio::spawn(async move {
            let mut read = BufReader::new(read);
            loop {
                let mut line = Vec::new();
                let result: Result<Incoming> = async {
                    let bytes = (&mut read)
                        .take(65_537)
                        .read_until(b'\n', &mut line)
                        .await?;
                    if bytes == 0 {
                        bail!("Evaluation transport disconnected before stop");
                    }
                    if bytes > 65_536 {
                        bail!("Evaluation notification exceeds transport limit");
                    }
                    Ok(serde_json::from_slice(&line)?)
                }
                .await;
                let failed = result.is_err();
                if send.send(result).await.is_err() || failed {
                    break;
                }
            }
        });
        let mut session = Self {
            writer,
            incoming,
            reader,
            prefix: prefix.into(),
            initialized: false,
            runs: HashSet::new(),
            tasks: HashMap::new(),
            questions: HashSet::new(),
            reply_to: None,
        };
        session.send(json!({"type":"ready"})).await?;
        Ok(session)
    }
    async fn send(&mut self, value: Value) -> Result<()> {
        self.writer
            .write_all(format!("{value}\n").as_bytes())
            .await?;
        Ok(())
    }
    pub async fn tick(&mut self, rt: &Runtime) -> Result<bool> {
        for _ in 0..32 {
            let event = match self.incoming.try_recv() {
                Ok(event) => event?,
                Err(mpsc::error::TryRecvError::Empty) => break,
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    bail!("Evaluation transport closed")
                }
            };
            match event {
                Incoming::Stop => return Ok(true),
                Incoming::Message {
                    run_id,
                    text,
                    notification,
                } => {
                    if run_id.is_empty()
                        || run_id.len() > 200
                        || text.trim().is_empty()
                        || text.len() > 32_000
                    {
                        bail!("Invalid evaluation notification");
                    }
                    if !self.runs.insert(run_id.clone()) {
                        continue;
                    }
                    // Only explicit external replies reach a pending clarification.
                    // Approval interpretation uses the existing runtime rules.
                    let result = if !self.initialized || notification {
                        let request = if self.initialized {
                            text.clone()
                        } else {
                            format!("{}\n\n{}", self.prefix, text)
                        };
                        if request.len() > 32_000 {
                            bail!("Initial evaluation request exceeds context input limit");
                        }
                        let task =
                            rt.store
                                .enqueue(&rt.redactor.text(&request), "evaluation", None)?;
                        rt.pin_destination(&task.id, "evaluation")?;
                        rt.wake.notify_one();
                        self.initialized = true;
                        json!({"task_id":task.id})
                    } else {
                        rt.submit_referenced(&text, "evaluation", self.reply_to.as_deref())
                            .await?
                    };
                    if let Some(task) = result["task_id"].as_str() {
                        self.tasks.insert(task.into(), run_id);
                    } else if let Some(task) = &self.reply_to {
                        self.tasks.insert(task.clone(), run_id);
                    }
                    if !notification {
                        self.reply_to = None;
                    }
                }
            }
        }
        let pending = rt
            .pending
            .lock()
            .await
            .values()
            .map(|p| p.view.clone())
            .collect::<Vec<_>>();
        for question in pending {
            if self.questions.insert(question.id.clone()) {
                if let Some(run) = self.tasks.get(&question.task_id).cloned() {
                    self.reply_to = Some(question.task_id.clone());
                    self.send(json!({"type":"response","run_id":run,"state":"final","message":question.question})).await?;
                }
            }
        }
        for (id, run) in self.tasks.clone() {
            let task = rt.store.task(&id)?;
            if !matches!(task.status.as_str(), "completed" | "failed" | "cancelled") {
                continue;
            }
            // Resolve the task's own final message, never a different worker's output.
            let mut before = None;
            let mut answer = None;
            loop {
                let page = rt.store.conversation(before, 200)?;
                if page.is_empty() {
                    break;
                }
                before = page.first().map(|m| m.seq);
                answer = page
                    .iter()
                    .rev()
                    .find(|m| m.task_id.as_deref() == Some(&id) && m.role == "assistant")
                    .map(|m| m.text.clone());
                if answer.is_some() {
                    break;
                }
            }
            let state = if task.status == "completed" {
                "final"
            } else {
                "error"
            };
            self.send(json!({"type":"response","run_id":run,"state":state,
                "message":answer.or(task.error).unwrap_or_else(|| "Task ended without an answer".into())})).await?;
            self.tasks.remove(&id);
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::Settings,
        evaluation::{EvaluationConfig, run},
    };
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[tokio::test]
    async fn notifications_keep_one_runtime_and_duplicate_delivery_does_not_repeat_work() {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("bridge.sock");
        let listener = tokio::net::UnixListener::bind(&socket).unwrap();
        let server = MockServer::start().await;
        Mock::given(method("POST")).and(path("/responses")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"id":"fixture","status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":"Recorded"}]}],"usage":{"input_tokens":20,"output_tokens":5}}))).mount(&server).await;
        let key = dir.path().join("key");
        std::fs::write(&key, "private-interactive-fixture").unwrap();
        let mut settings = Settings::initial(dir.path().into());
        settings.base_url = Some(server.uri());
        let output = dir.path().join("trial");
        let config = EvaluationConfig {
            instruction: "Respond briefly to the user's messages.".into(),
            settings,
            api_key_file: key,
            timeout_seconds: 40,
            waiting_seconds: 1,
            benchmark: "fixture".into(),
            dataset_revision: "fixture".into(),
            task_name: "two-turns".into(),
            proxy_upstream: Some(*server.address()),
            decisions: None,
            interaction_socket: Some(socket),
        };
        let output_copy = output.clone();
        let handle = tokio::spawn(async move { run(config, &output_copy, false).await });
        let (stream, _) = listener.accept().await.unwrap();
        let (read, mut write) = stream.into_split();
        let mut read = BufReader::new(read).lines();
        let ready = read.next_line().await.unwrap().unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&ready).unwrap()["type"],
            "ready"
        );
        for id in ["first", "second"] {
            let line = format!(
                "{}\n",
                json!({"type":"message","run_id":id,"text":format!("Remember the label {id}; acknowledge it briefly.")})
            );
            write.write_all(line.as_bytes()).await.unwrap();
            if id == "first" {
                write.write_all(line.as_bytes()).await.unwrap();
            }
            let response =
                tokio::time::timeout(std::time::Duration::from_secs(20), read.next_line())
                    .await
                    .unwrap()
                    .unwrap()
                    .unwrap();
            let response: Value = serde_json::from_str(&response).unwrap();
            assert_eq!(response["run_id"], id);
            assert_eq!(response["state"], "final");
            assert!(!output.join("ready.json").exists());
        }
        write.write_all(b"{\"type\":\"stop\"}\n").await.unwrap();
        handle.await.unwrap().unwrap();
        let report: Value =
            serde_json::from_slice(&std::fs::read(output.join("report.json")).unwrap()).unwrap();
        assert_eq!(report["stop_reason"], "benchmark_complete");
        assert!(report["reward"].is_null());
        assert_eq!(report["traces"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn interactive_questions_accept_replies_but_not_notifications() -> Result<()> {
        tokio::time::timeout(std::time::Duration::from_secs(15), async {
            let dir = tempfile::tempdir()?;
            let rt = Runtime::open(&dir.path().join("runtime"), Some(&dir.path().join("skills"))).await?;
            let task = rt.store.enqueue("Choose the intended contact", "evaluation", None)?;
            assert_eq!(rt.autonomy_context(&task.id)?["interactive"], false);
            rt.store.put("evaluation", "noninteractive", &false)?;
            assert_eq!(rt.autonomy_context(&task.id)?["interactive"], true);
            rt.check_clarification_policy(&task.id, &json!({"question":"Which contact?"}))?;
            let socket = dir.path().join("questions.sock");
            let listener = tokio::net::UnixListener::bind(&socket)?;
            let mut session = Session::connect(&socket, "Environment instructions").await?;
            session.initialized = true;
            session.tasks.insert(task.id.clone(), "initial".into());
            let (stream, _) = listener.accept().await?;
            let (read, mut write) = stream.into_split();
            let mut read = BufReader::new(read).lines();
            read.next_line().await?; // Ready handshake.
            let runtime = rt.clone();
            let id = task.id.clone();
            let answer = tokio::spawn(async move {
                runtime.ask(&id, "clarification", "Which contact?", None,
                    &tokio_util::sync::CancellationToken::new()).await
            });
            while rt.pending.lock().await.is_empty() {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            session.tick(&rt).await?;
            let question: Value = serde_json::from_str(&read.next_line().await?.unwrap())?;
            assert_eq!(question["message"], "Which contact?");
            write.write_all(b"{\"type\":\"message\",\"run_id\":\"notification\",\"notification\":true,\"text\":\"A new email arrived\"}\n").await?;
            while session.tasks.len() < 2 {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                session.tick(&rt).await?;
            }
            assert_eq!(rt.pending.lock().await.len(), 1);
            assert!(!answer.is_finished());
            write.write_all(b"{\"type\":\"message\",\"run_id\":\"reply\",\"text\":\"The intended contact is Alex.\"}\n").await?;
            while !answer.is_finished() {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                session.tick(&rt).await?;
            }
            assert_eq!(answer.await??, "The intended contact is Alex.");
            assert!(rt.pending.lock().await.is_empty());
            rt.store.put("evaluation", "noninteractive", &true)?;
            assert_eq!(rt.autonomy_context(&task.id)?["interactive"], false);
            rt.shutdown().await?;
            Ok::<_, anyhow::Error>(())
        }).await??;
        Ok(())
    }
}
