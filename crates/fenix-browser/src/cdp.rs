//! Bounded local CDP transport. Never expose the debugging socket to the UI/model.
use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use tokio::sync::{Notify, mpsc, oneshot};
use tokio_util::sync::CancellationToken;

type Pending = Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value>>>>>;
pub struct Cdp {
    outgoing: mpsc::Sender<Value>,
    pending: Pending,
    next: AtomicU64,
    pub events: Arc<Mutex<VecDeque<Value>>>,
    pub downloads: Arc<Mutex<crate::downloads::Downloads>>,
    dialogs: Arc<Mutex<HashMap<String, Value>>>,
    dialog_changed: Arc<Notify>,
    stop: CancellationToken,
    closed: CancellationToken,
    network: Arc<Mutex<HashSet<String>>>,
    last_network: Arc<Mutex<std::time::Instant>>,
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
        Self::from_socket(socket)
    }
    fn from_socket<S>(socket: tokio_tungstenite::WebSocketStream<S>) -> Result<Self>
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
    {
        let (mut writer, mut reader) = socket.split();
        let (tx, mut rx) = mpsc::channel::<Value>(32);
        let pending: Pending = Arc::default();
        let replies = pending.clone();
        let events: Arc<Mutex<VecDeque<Value>>> = Arc::default();
        let observed = events.clone();
        let downloads = Arc::new(Mutex::new(crate::downloads::Downloads::default()));
        let transfers = downloads.clone();
        let dialogs: Arc<Mutex<HashMap<String, Value>>> = Arc::default();
        let active_dialogs = dialogs.clone();
        let dialog_changed = Arc::new(Notify::new());
        let notify_dialog = dialog_changed.clone();
        let network = Arc::new(Mutex::new(HashSet::new()));
        let requests = network.clone();
        let last_network = Arc::new(Mutex::new(std::time::Instant::now()));
        let last_request = last_network.clone();
        let stop = CancellationToken::new();
        let stopped = stop.clone();
        let closed = CancellationToken::new();
        let finished = closed.clone();
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
                        if let Some(request) = value["params"]["requestId"].as_str() {
                            let key = format!("{}:{request}", value["sessionId"].as_str().unwrap_or(""));
                            match value["method"].as_str() {
                                Some("Network.requestWillBeSent" | "Network.webSocketCreated") => { requests.lock().unwrap().insert(key); *last_request.lock().unwrap() = std::time::Instant::now(); }
                                Some("Network.loadingFinished" | "Network.loadingFailed" | "Network.webSocketClosed") => { requests.lock().unwrap().remove(&key); *last_request.lock().unwrap() = std::time::Instant::now(); }
                                _ => {}
                            }
                        }
                        if let Some(id) = value["id"].as_u64() {
                            if let Some(reply) = replies.lock().unwrap().remove(&id) {
                                // Remote diagnostics can contain form data. Keep errors generic.
                                let result = if value.get("error").is_some() { Err(anyhow::anyhow!("Browser command failed; refresh the page observation before retrying")) } else { Ok(value["result"].clone()) };
                                let _ = reply.send(result);
                            }
                        } else if matches!(value["method"].as_str(), Some("Runtime.exceptionThrown" | "Network.loadingFailed" | "Page.javascriptDialogOpening" | "Page.javascriptDialogClosed" | "Browser.downloadWillBegin" | "Browser.downloadProgress")) {
                            if let Some(session) = value["sessionId"].as_str() {
                                match value["method"].as_str() {
                                    Some("Page.javascriptDialogOpening") => {
                                        active_dialogs.lock().unwrap().insert(session.into(), json!({"type":value["params"]["type"]}));
                                        notify_dialog.notify_waiters();
                                    }
                                    Some("Page.javascriptDialogClosed") => { active_dialogs.lock().unwrap().remove(session); }
                                    _ => {}
                                }
                            }
                            transfers.lock().unwrap().observe(&value);
                            let mut queue = observed.lock().unwrap();
                            if queue.len() == 100 { queue.pop_front(); }
                            // Never retain console arguments, response bodies, cookies or request headers.
                            queue.push_back(json!({"event":value["method"],"session":value["sessionId"],"type":value["params"]["type"],"state":value["params"]["state"],"guid":value["params"]["guid"]}));
                        }
                    }
                }
            }
            drop(writer);
            drop(reader);
            for (_, reply) in replies.lock().unwrap().drain() {
                let _ = reply.send(Err(anyhow::anyhow!("Browser connection closed")));
            }
            finished.cancel();
        });
        Ok(Self {
            outgoing: tx,
            pending,
            next: AtomicU64::new(1),
            events,
            downloads,
            dialogs,
            dialog_changed,
            stop,
            closed,
            network,
            last_network,
        })
    }
    pub fn is_quiet(&self) -> bool {
        self.pending.lock().unwrap().is_empty()
            && self.network.lock().unwrap().is_empty()
            && self.dialogs.lock().unwrap().is_empty()
            && !self.downloads.lock().unwrap().has_pending()
            && self.last_network.lock().unwrap().elapsed() >= Duration::from_secs(2)
    }
    pub async fn disconnect(&self) {
        self.stop.cancel();
        self.closed.cancelled().await;
    }

    pub fn is_disconnected(&self) -> bool {
        self.closed.is_cancelled()
    }
    pub fn dialog(&self, session: &str) -> Option<Value> {
        self.dialogs.lock().unwrap().get(session).cloned()
    }
    pub async fn wait_dialog(&self, session: &str) -> Value {
        loop {
            let notified = self.dialog_changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if let Some(dialog) = self.dialog(session) {
                return dialog;
            }
            notified.await;
        }
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
