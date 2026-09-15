//! Bounded local CDP transport. Never expose the debugging socket to the UI/model.
use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, VecDeque},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

type Pending = Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value>>>>>;
pub struct Cdp {
    outgoing: mpsc::Sender<Value>,
    pending: Pending,
    next: AtomicU64,
    pub events: Arc<Mutex<VecDeque<Value>>>,
    stop: CancellationToken,
}
impl Cdp {
    pub async fn connect(endpoint: &str) -> Result<Self> {
        let config = tokio_tungstenite::tungstenite::protocol::WebSocketConfig::default()
            .max_message_size(Some(16 * 1024 * 1024))
            .max_frame_size(Some(16 * 1024 * 1024));
        let (socket, _) =
            tokio_tungstenite::connect_async_with_config(endpoint, Some(config), true)
                .await
                .context("Could not connect to the local browser")?;
        let (mut writer, mut reader) = socket.split();
        let (tx, mut rx) = mpsc::channel::<Value>(32);
        let pending: Pending = Arc::default();
        let replies = pending.clone();
        let events: Arc<Mutex<VecDeque<Value>>> = Arc::default();
        let observed = events.clone();
        let stop = CancellationToken::new();
        let stopped = stop.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = stopped.cancelled() => break,
                    message = rx.recv() => {
                        let Some(message) = message else { break; };
                        if writer.send(message.to_string().into()).await.is_err() { break; }
                    }
                    message = reader.next() => {
                        let Some(Ok(message)) = message else { break; };
                        let Ok(text) = message.to_text() else { continue; };
                        let Ok(value) = serde_json::from_str::<Value>(text) else { continue; };
                        if let Some(id) = value["id"].as_u64() {
                            if let Some(reply) = replies.lock().unwrap().remove(&id) {
                                // Remote diagnostics can contain form data. Keep errors generic.
                                let result = if value.get("error").is_some() { Err(anyhow::anyhow!("Browser command failed; refresh the page observation before retrying")) } else { Ok(value["result"].clone()) };
                                let _ = reply.send(result);
                            }
                        } else if matches!(value["method"].as_str(), Some("Runtime.exceptionThrown" | "Network.loadingFailed" | "Page.javascriptDialogOpening" | "Page.javascriptDialogClosed" | "Browser.downloadWillBegin" | "Browser.downloadProgress")) {
                            let mut queue = observed.lock().unwrap();
                            if queue.len() == 100 { queue.pop_front(); }
                            // Never retain console arguments, response bodies, cookies or request headers.
                            queue.push_back(json!({"event":value["method"],"session":value["sessionId"],"type":value["params"]["type"],"state":value["params"]["state"],"guid":value["params"]["guid"]}));
                        }
                    }
                }
            }
            for (_, reply) in replies.lock().unwrap().drain() {
                let _ = reply.send(Err(anyhow::anyhow!("Browser connection closed")));
            }
        });
        Ok(Self {
            outgoing: tx,
            pending,
            next: AtomicU64::new(1),
            events,
            stop,
        })
    }
    pub async fn call(&self, session: Option<&str>, method: &str, params: Value) -> Result<Value> {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().unwrap().insert(id, tx);
        struct Remove(Pending, u64);
        impl Drop for Remove {
            fn drop(&mut self) {
                self.0.lock().unwrap().remove(&self.1);
            }
        }
        let _remove = Remove(self.pending.clone(), id);
        let mut message = json!({"id":id,"method":method,"params":params});
        if let Some(session) = session {
            message["sessionId"] = json!(session);
        }
        tokio::time::timeout(Duration::from_secs(15), async {
            self.outgoing
                .send(message)
                .await
                .context("Browser transport closed")?;
            rx.await.context("Browser response channel closed")?
        })
        .await
        .context("Browser command timed out; its effect may already have occurred")?
    }
}
impl Drop for Cdp {
    fn drop(&mut self) {
        self.stop.cancel();
    }
}
