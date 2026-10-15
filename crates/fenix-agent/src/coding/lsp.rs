//! Bounded, lazy stdio LSP sessions, isolated by canonical workspace and server.
//! Config comes from the owner settings, never executable repository instructions.
use super::symbols;
use crate::files::read_text;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::{Mutex, mpsc},
};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub name: String,
    /// Executable plus arguments; no shell interpolation is performed.
    pub command: Vec<String>,
    /// Extensions without dots, for example ["rs"].
    pub extensions: Vec<String>,
    #[serde(default)]
    pub initialization_options: Value,
}
impl ServerConfig {
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty()
            || self.name.len() > 80
            || self.command.is_empty()
            || self.command.len() > 32
            || self
                .command
                .iter()
                .any(|s| s.len() > 4096 || s.contains('\0'))
            || self.command[0].is_empty()
            || self.extensions.is_empty()
            || self
                .extensions
                .iter()
                .any(|s| s.is_empty() || s.len() > 20 || s.contains(['/', '.']))
        {
            bail!("Invalid language server configuration");
        }
        Ok(())
    }
}
#[derive(Default)]
pub(crate) struct LanguageServers {
    sessions: Mutex<BTreeMap<String, SessionSlot>>,
}
struct SessionSlot {
    session: Arc<Mutex<Option<Session>>>,
    last_used: Instant,
    stopped: CancellationToken,
}
impl LanguageServers {
    pub async fn shutdown(&self) {
        let mut sessions = self.sessions.lock().await;
        for entry in sessions.values() {
            entry.stopped.cancel();
        }
        sessions.clear();
    }
    pub async fn query(
        &self,
        workspace: &Path,
        path: &Path,
        args: &Value,
        configs: &[ServerConfig],
        cancel: &CancellationToken,
    ) -> Result<Value> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let config=configs.iter().find(|c|c.extensions.iter().any(|e|e==extension)).context("No configured language server for this file extension. Configure an installed stdio server in Settings > Coding; nothing was installed or started.")?;
        config.validate()?;
        let workspace = std::fs::canonicalize(workspace)?;
        let workspace = if path.starts_with(&workspace) {
            workspace
        } else {
            path.parent()
                .context("Language query requires a parent directory")?
                .to_owned()
        };
        let key = format!("{}:{}", workspace.display(), serde_json::to_string(config)?);
        // The pool lock only manages slots. Each workspace/server has its own
        // transport lock; unrelated worktrees can query concurrently.
        let (slot, stopped) = {
            let mut sessions = self.sessions.lock().await;
            if cancel.is_cancelled() {
                bail!("Language query cancelled");
            }
            if !sessions.contains_key(&key) && sessions.len() >= 4 {
                let oldest = sessions
                    .iter()
                    .filter(|(_, slot)| Arc::strong_count(&slot.session) == 1)
                    .min_by_key(|(_, slot)| slot.last_used)
                    .map(|(key, _)| key.clone());
                if let Some(oldest) = oldest {
                    sessions.remove(&oldest);
                } else {
                    bail!(
                        "All language server sessions are busy; retry after an active query finishes"
                    );
                }
            }
            let entry = sessions.entry(key.clone()).or_insert_with(|| SessionSlot {
                session: Arc::new(Mutex::new(None)),
                last_used: Instant::now(),
                stopped: CancellationToken::new(),
            });
            entry.last_used = Instant::now();
            (entry.session.clone(), entry.stopped.clone())
        };
        let mut guard = tokio::select! {
            _=cancel.cancelled()=>bail!("Language query cancelled"),
            _=stopped.cancelled()=>bail!("Language servers stopped"),
            result=tokio::time::timeout(Duration::from_secs(30), slot.lock())=>result.context("Language server busy")?,
        };
        let execute = async {
            if guard.is_none() {
                *guard = Some(
                    tokio::time::timeout(
                        Duration::from_secs(30),
                        Session::start(config, &workspace),
                    )
                    .await
                    .context("Language server initialization timed out")??,
                );
            }
            // Read after acquiring the session, not before waiting behind another
            // query. A queued request must not restore an obsolete document.
            let text = read_text(path)?;
            let result = tokio::time::timeout(
                Duration::from_secs(20),
                guard.as_mut().unwrap().query(path, &text, args),
            )
            .await
            .context("Language query timed out")??;
            if read_text(path)? != text {
                bail!("File changed during the language query; retry against the current version");
            }
            Ok(result)
        };
        let result = tokio::select! {
            _=cancel.cancelled()=>Err(anyhow::anyhow!("Language query cancelled")),
            _=stopped.cancelled()=>Err(anyhow::anyhow!("Language servers stopped")),
            result=execute=>result,
        };
        // Failed streams may have unread replies. Drop the process; queued users
        // retain this slot and lazily start a fresh session under the same lock.
        if result.is_err() {
            *guard = None;
        }
        let mut sessions = self.sessions.lock().await;
        if let Some(entry) = sessions.get_mut(&key)
            && Arc::ptr_eq(&entry.session, &slot)
        {
            entry.last_used = Instant::now();
            if result.is_err() && Arc::strong_count(&slot) == 2 {
                sessions.remove(&key);
            }
        }
        result
    }
}
struct Session {
    child: Child,
    _tree: fenix_process::ProcessTree,
    writer: ChildStdin,
    inbox: mpsc::Receiver<Result<Value>>,
    reader_task: tokio::task::JoinHandle<()>,
    next_id: u64,
    capabilities: Value,
    documents: HashMap<String, (i64, String)>,
    diagnostics: HashMap<String, Value>,
    symbols: HashMap<String, Value>,
    workspace: PathBuf,
}
impl Drop for Session {
    fn drop(&mut self) {
        self.reader_task.abort();
        let _ = self.child.start_kill();
    }
}
impl Session {
    async fn start(config: &ServerConfig, workspace: &Path) -> Result<Self> {
        let mut command = Command::new(&config.command[0]);
        command
            .args(&config.command[1..])
            .env_clear()
            .envs(fenix_execution::local_environment(workspace));
        command
            .current_dir(workspace)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true);
        let (mut child, tree) = fenix_process::spawn_program(command)
            .context("Could not start configured language server")?;
        let writer = child.stdin.take().context("Missing LSP stdin")?;
        let mut reader = BufReader::new(child.stdout.take().context("Missing LSP stdout")?);
        let (sender, inbox) = mpsc::channel(16);
        let reader_task = tokio::spawn(async move {
            loop {
                let result = read_frame(&mut reader).await;
                let failed = result.is_err();
                if sender.send(result).await.is_err() || failed {
                    break;
                }
            }
        });
        let mut session = Self {
            child,
            _tree: tree,
            writer,
            inbox,
            reader_task,
            next_id: 0,
            capabilities: Value::Null,
            documents: HashMap::new(),
            diagnostics: HashMap::new(),
            symbols: HashMap::new(),
            workspace: workspace.into(),
        };
        let uri = file_uri(workspace)?;
        let result=session.request("initialize",json!({"processId":std::process::id(),"rootUri":uri,"workspaceFolders":[{"uri":uri,"name":workspace.file_name().unwrap_or_default().to_string_lossy()}],"capabilities":{"general":{"positionEncodings":["utf-16"]},"textDocument":{"documentSymbol":{"hierarchicalDocumentSymbolSupport":true},"publishDiagnostics":{"versionSupport":true},"diagnostic":{},"synchronization":{"didSave":true}},"workspace":{"configuration":true,"applyEdit":false}},"initializationOptions":config.initialization_options})).await?;
        session.capabilities = result["capabilities"].clone();
        if session.capabilities["positionEncoding"]
            .as_str()
            .is_some_and(|encoding| encoding != "utf-16")
        {
            bail!("Language server selected an unsupported position encoding");
        }
        session
            .send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}))
            .await?;
        Ok(session)
    }
    async fn send(&mut self, value: Value) -> Result<()> {
        let data = serde_json::to_vec(&value)?;
        self.writer
            .write_all(format!("Content-Length: {}\r\n\r\n", data.len()).as_bytes())
            .await?;
        self.writer.write_all(&data).await?;
        self.writer.flush().await?;
        Ok(())
    }
    async fn receive(&mut self) -> Result<Value> {
        self.inbox
            .recv()
            .await
            .context("Language server closed stdout")?
    }
    async fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        self.next_id += 1;
        let id = self.next_id;
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}))
            .await?;
        loop {
            let message = self.receive().await?;
            if message.get("method").is_some() {
                self.server_message(&message).await?;
                continue;
            }
            if message["id"] == id {
                if !message["error"].is_null() {
                    bail!("LSP {}: {}", method, message["error"]);
                }
                return Ok(message["result"].clone());
            }
        }
    }
    async fn server_message(&mut self, message: &Value) -> Result<()> {
        if let Some(id) = message.get("id") {
            let result = match message["method"].as_str() {
                Some("workspace/configuration") => json!(vec![
                    Value::Null;
                    message["params"]["items"]
                        .as_array()
                        .map_or(0, Vec::len)
                ]),
                Some("workspace/workspaceFolders") => {
                    json!([{"uri":file_uri(&self.workspace)?,"name":self.workspace.file_name().unwrap_or_default().to_string_lossy()}])
                }
                Some("workspace/applyEdit") => {
                    json!({"applied":false,"failureReason":"Fenix only accepts edits through its approved file tools"})
                }
                _ => Value::Null,
            };
            self.send(json!({"jsonrpc":"2.0","id":id,"result":result}))
                .await?;
        } else if message["method"] == "textDocument/publishDiagnostics" {
            let p = &message["params"];
            if let Some(uri) = p["uri"].as_str()
                && let Some((version, _)) = self.documents.get(uri)
                && p["version"].as_i64().is_none_or(|v| v == *version)
            {
                self.diagnostics.insert(uri.into(), p.clone());
            }
        }
        Ok(())
    }
    async fn sync(&mut self, path: &Path, text: &str) -> Result<String> {
        let uri = file_uri(path)?;
        let old = self.documents.get(&uri).cloned();
        if old.as_ref().is_some_and(|(_, t)| t == text) {
            return Ok(uri);
        }
        let version = old.as_ref().map_or(1, |(v, _)| v + 1);
        let method = if old.is_some() {
            "textDocument/didChange"
        } else {
            "textDocument/didOpen"
        };
        let params = if old.is_some() {
            json!({"textDocument":{"uri":uri,"version":version},"contentChanges":[{"text":text}]})
        } else {
            let language = match path.extension().and_then(|p| p.to_str()).unwrap_or("") {
                "rs" => "rust",
                "py" => "python",
                "ts" => "typescript",
                "tsx" => "typescriptreact",
                "js" => "javascript",
                "go" => "go",
                other => other,
            };
            json!({"textDocument":{"uri":uri,"languageId":language,"version":version,"text":text}})
        };
        // Send incremental replacement when required by the server.
        let mut params = params;
        let sync = self.capabilities["textDocumentSync"]
            .as_i64()
            .or(self.capabilities["textDocumentSync"]["change"].as_i64());
        if sync == Some(2)
            && let Some((_, previous)) = &old
        {
            let last = previous.rsplit('\n').next().unwrap_or("");
            params["contentChanges"][0]["range"] = json!({"start":{"line":0,"character":0},"end":{"line":previous.bytes().filter(|b|*b==b'\n').count(),"character":last.encode_utf16().count()}});
        }
        self.documents.insert(uri.clone(), (version, text.into()));
        self.diagnostics.remove(&uri);
        self.symbols.clear();
        self.send(json!({"jsonrpc":"2.0","method":method,"params":params}))
            .await?;
        // Bound server document state and send didClose when evicting.
        if self.documents.len() > 32
            && let Some(old) = self.documents.keys().find(|k| *k != &uri).cloned()
        {
            self.send(json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":old}}})).await?;
            self.documents.remove(&old);
            self.diagnostics.remove(&old);
        }
        Ok(uri)
    }
    async fn close_document(&mut self, uri: &str) -> Result<()> {
        self.send(json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":uri}}})).await?;
        self.documents.remove(uri);
        self.diagnostics.remove(uri);
        self.symbols.clear();
        Ok(())
    }
    /// Refresh only the bounded set of open documents. Language servers retain
    /// responsibility for indexing unopened files; this is not a repository scan.
    async fn refresh_documents(&mut self, requested: &Path) -> Result<Vec<String>> {
        let uris: Vec<_> = self.documents.keys().cloned().collect();
        let mut unavailable = Vec::new();
        let mut changes = Vec::new();
        for uri in uris {
            let path = url::Url::parse(&uri)?
                .to_file_path()
                .map_err(|_| anyhow::anyhow!("Invalid open document URI"))?;
            if path == requested {
                continue;
            }
            // read_text uses descriptor-based no-symlink access. A file replaced
            // by an escape, deleted, oversized or made binary must not remain as
            // an apparently current open buffer.
            let text = if path.starts_with(&self.workspace) {
                read_text(&path).ok()
            } else {
                None
            };
            if let Some(text) = text {
                if self
                    .documents
                    .get(&uri)
                    .is_some_and(|(_, old)| old != &text)
                {
                    self.sync(&path, &text).await?;
                    changes.push(json!({"uri":uri,"type":2}));
                }
            } else {
                self.close_document(&uri).await?;
                if !path.exists() {
                    changes.push(json!({"uri":uri,"type":3}));
                }
                unavailable.push(uri);
            }
        }
        if !changes.is_empty() {
            self.send(json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles","params":{"changes":changes}})).await?;
        }
        Ok(unavailable)
    }
    async fn document_symbols(&mut self, uri: &str) -> Result<Value> {
        if let Some(cached) = self.symbols.get(uri) {
            return Ok(cached.clone());
        }
        if self.capabilities["documentSymbolProvider"] == false {
            bail!(
                "This language server does not support document symbols; use files_search instead"
            );
        }
        let result = self
            .request(
                "textDocument/documentSymbol",
                json!({"textDocument":{"uri":uri}}),
            )
            .await?;
        // Eight entries of at most 128 KiB: keep the cache small per session.
        if serde_json::to_vec(&result)?.len() <= 128 * 1024 {
            if self.symbols.len() >= 8 {
                self.symbols.clear();
            }
            self.symbols.insert(uri.into(), result.clone());
        }
        Ok(result)
    }
    async fn query(&mut self, path: &Path, text: &str, args: &Value) -> Result<Value> {
        let unavailable = self.refresh_documents(path).await?;
        if !unavailable.is_empty() {
            // Make loss of dependency buffers explicit rather than silently
            // treating an incomplete query as authoritative.
            bail!(
                "Previously open documents are unavailable; retry after checking these paths: {}",
                unavailable.join(", ")
            );
        }
        let uri = self.sync(path, text).await?;
        let operation = args["operation"].as_str().unwrap_or("diagnostics");
        if matches!(operation, "symbol_outline" | "find_symbol" | "read_symbol") {
            let raw = self.document_symbols(&uri).await?;
            let result = symbols::response(&raw, &uri, text, args)?;
            return Ok(
                json!({"path":path,"operation":operation,"version":self.documents[&uri].0,
                "position_encoding":"utf-16","range_base":0,"result":result}),
            );
        }
        let position = if args["name_path"].is_string()
            && matches!(
                operation,
                "definition" | "references" | "hover" | "implementation"
            ) {
            let raw = self.document_symbols(&uri).await?;
            let symbols = symbols::flatten(&raw, &uri);
            symbols::selection(symbols::unique(&symbols, args)?, text)?
        } else {
            json!({"line":args["line"].as_u64().unwrap_or(1).saturating_sub(1),"character":args["character"].as_u64().unwrap_or(1).saturating_sub(1)})
        };
        let mut params = json!({"textDocument":{"uri":uri},"position":position});
        let method = match operation {
            "definition" => "textDocument/definition",
            "references" => {
                params["context"] = json!({"includeDeclaration":true});
                "textDocument/references"
            }
            "hover" => "textDocument/hover",
            "implementation" => "textDocument/implementation",
            "symbols" => {
                let result = self.document_symbols(&uri).await?;
                return Ok(
                    json!({"path":path,"operation":operation,"position_encoding":"utf-16","result":result}),
                );
            }
            "diagnostics" => {
                if self.capabilities["diagnosticProvider"].is_object() {
                    let report = self
                        .request(
                            "textDocument/diagnostic",
                            json!({"textDocument":{"uri":uri}}),
                        )
                        .await?;
                    return Ok(
                        json!({"status":"reported","path":path,"version":self.documents[&uri].0,"diagnostics":report,"source":"lsp_pull"}),
                    );
                }
                // Push diagnostics may arrive asynchronously after document synchronization.
                let receive = async {
                    while !self.diagnostics.contains_key(&uri) {
                        let message = self.receive().await?;
                        self.server_message(&message).await?;
                    }
                    Ok::<(), anyhow::Error>(())
                };
                match tokio::time::timeout(Duration::from_secs(2), receive).await {
                    Ok(result) => result?,
                    Err(_) => {
                        return Ok(
                            json!({"status":"pending","path":path,"version":self.documents[&uri].0,"instruction":"No current diagnostic report yet. Retry later or run the compiler; this is not a passing check."}),
                        );
                    }
                }
                return Ok(
                    json!({"status":"reported","path":path,"version":self.documents[&uri].0,"diagnostics":self.diagnostics[&uri],"source":"lsp_push"}),
                );
            }
            _ => bail!("Unsupported code query"),
        };
        let result = self.request(method, params).await?;
        Ok(json!({"path":path,"operation":operation,"position_encoding":"utf-16","result":result}))
    }
}
fn file_uri(path: &Path) -> Result<String> {
    Ok(url::Url::from_file_path(path)
        .map_err(|_| anyhow::anyhow!("Invalid file URI"))?
        .to_string())
}

async fn read_frame(reader: &mut BufReader<ChildStdout>) -> Result<Value> {
    let mut length = None;
    let mut total = 0;
    loop {
        let mut line = Vec::new();
        // Bound even a malformed server header that never sends a newline.
        let n = (&mut *reader)
            .take(8193)
            .read_until(b'\n', &mut line)
            .await?;
        if n == 0 {
            bail!("Language server closed stdout");
        }
        total += n;
        if total > 8192 {
            bail!("LSP header too large");
        }
        let line = std::str::from_utf8(&line)?.trim();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            length = Some(value.trim().parse::<usize>()?);
        }
    }
    let length = length.context("Missing LSP Content-Length")?;
    if length > 4 * 1024 * 1024 {
        bail!("LSP response exceeds 4 MiB");
    }
    let mut bytes = vec![0; length];
    reader.read_exact(&mut bytes).await?;
    Ok(serde_json::from_slice(&bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture_config() -> Vec<ServerConfig> {
        vec![ServerConfig {
            name: "fixture".into(),
            command: vec![
                "python3".into(),
                format!(
                    "{}/tests/fixtures/lsp_server.py",
                    env!("CARGO_MANIFEST_DIR")
                ),
            ],
            extensions: vec!["rs".into()],
            initialization_options: Value::Null,
        }]
    }
    #[tokio::test]
    async fn independent_workspaces_do_not_wait_for_slow_queries() {
        let slow_dir = tempfile::tempdir().unwrap();
        let fast_dir = tempfile::tempdir().unwrap();
        let slow = slow_dir.path().join("slow.rs");
        let fast = fast_dir.path().join("fast.rs");
        std::fs::write(&slow, "SLOW").unwrap();
        std::fs::write(&fast, "fast").unwrap();
        let pool = Arc::new(LanguageServers::default());
        let cancel = CancellationToken::new();
        let worker_pool = pool.clone();
        let worker_cancel = cancel.clone();
        let slow_workspace = slow_dir.path().to_owned();
        let worker = tokio::spawn(async move {
            worker_pool
                .query(
                    &slow_workspace,
                    &slow,
                    &json!({"operation":"hover"}),
                    &fixture_config(),
                    &worker_cancel,
                )
                .await
        });
        // Observe the actual request in the fixture, not a guessed startup delay.
        tokio::time::timeout(Duration::from_secs(5), async {
            while !slow_dir.path().join("slow-started").exists() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        let result = tokio::time::timeout(
            Duration::from_secs(3),
            pool.query(
                fast_dir.path(),
                &fast,
                &json!({"operation":"hover"}),
                &fixture_config(),
                &CancellationToken::new(),
            ),
        )
        .await;
        cancel.cancel();
        assert!(worker.await.unwrap().is_err());
        assert_eq!(result.unwrap().unwrap()["result"]["contents"], "fast");
        pool.shutdown().await;
    }
    #[tokio::test]
    async fn queued_query_reads_latest_file_after_cancelled_predecessor() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.rs");
        std::fs::write(&file, "SLOW").unwrap();
        let pool = Arc::new(LanguageServers::default());
        let first_cancel = CancellationToken::new();
        let spawn_query = |token: CancellationToken| {
            let pool = pool.clone();
            let path = file.clone();
            let workspace = dir.path().to_owned();
            tokio::spawn(async move {
                pool.query(
                    &workspace,
                    &path,
                    &json!({"operation":"hover"}),
                    &fixture_config(),
                    &token,
                )
                .await
            })
        };
        let first = spawn_query(first_cancel.clone());
        tokio::time::timeout(Duration::from_secs(5), async {
            while !dir.path().join("slow-started").exists() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        let second = spawn_query(CancellationToken::new());
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if pool
                    .sessions
                    .lock()
                    .await
                    .values()
                    .any(|slot| Arc::strong_count(&slot.session) >= 3)
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        std::fs::write(&file, "updated while waiting").unwrap();
        first_cancel.cancel();
        assert!(first.await.unwrap().is_err());
        let result = tokio::time::timeout(Duration::from_secs(3), second)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(result["result"]["contents"], "updated while waiting");
        pool.shutdown().await;
    }
    #[tokio::test]
    async fn symbol_cache_named_queries_and_external_dependency_refresh() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.rs");
        let dependency = dir.path().join("b.rs");
        std::fs::write(&file, "fn run() { 1 }\n").unwrap();
        std::fs::write(&dependency, "old").unwrap();
        let pool = LanguageServers::default();
        let configs = fixture_config();
        let cancel = CancellationToken::new();
        let outline = pool
            .query(
                dir.path(),
                &file,
                &json!({"operation":"symbol_outline","limit":1}),
                &configs,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(outline["result"]["total"], 2);
        assert_eq!(outline["result"]["next_offset"], 1);
        let body = pool
            .query(
                dir.path(),
                &file,
                &json!({"operation":"read_symbol","name_path":"Module/run"}),
                &configs,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(body["result"]["body"], "fn run() { 1 }");
        let hover = pool
            .query(
                dir.path(),
                &file,
                &json!({"operation":"hover","name_path":"run"}),
                &configs,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(hover["result"]["position"]["character"], 3);
        assert_eq!(hover["result"]["symbol_requests"], 1);
        pool.query(
            dir.path(),
            &dependency,
            &json!({"operation":"hover"}),
            &configs,
            &cancel,
        )
        .await
        .unwrap();
        // Same-length writes and changes to a different open file are detected.
        std::fs::write(&dependency, "new").unwrap();
        std::fs::write(&file, "fn run() { 2 }\n").unwrap();
        let changed = pool
            .query(
                dir.path(),
                &file,
                &json!({"operation":"read_symbol","name_path":"run"}),
                &configs,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(changed["result"]["body"], "fn run() { 2 }");
        assert_ne!(
            body["result"]["file_sha256"],
            changed["result"]["file_sha256"]
        );
        let hover = pool
            .query(
                dir.path(),
                &file,
                &json!({"operation":"hover"}),
                &configs,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(
            hover["result"]["documents"][file_uri(&dependency).unwrap()],
            "new"
        );
        assert_eq!(hover["result"]["symbol_requests"], 2);
        std::fs::remove_file(&dependency).unwrap();
        let removed = pool
            .query(
                dir.path(),
                &file,
                &json!({"operation":"hover"}),
                &configs,
                &cancel,
            )
            .await
            .unwrap_err();
        assert!(removed.to_string().contains("unavailable"));
        let fresh = pool
            .query(
                dir.path(),
                &file,
                &json!({"operation":"hover"}),
                &configs,
                &cancel,
            )
            .await
            .unwrap();
        assert!(
            fresh["result"]["documents"]
                .get(file_uri(&dependency).unwrap())
                .is_none()
        );
        pool.shutdown().await;
    }
    #[tokio::test]
    async fn pool_evicts_least_recently_used_idle_session() {
        let dirs: Vec<_> = (0..5).map(|_| tempfile::tempdir().unwrap()).collect();
        let pool = LanguageServers::default();
        let configs = fixture_config();
        let cancel = CancellationToken::new();
        let query = json!({"operation":"hover"});
        let mut pids = Vec::new();
        for dir in &dirs[..4] {
            let file = dir.path().join("a.rs");
            std::fs::write(&file, "ok").unwrap();
            pids.push(
                pool.query(dir.path(), &file, &query, &configs, &cancel)
                    .await
                    .unwrap()["result"]["pid"]
                    .clone(),
            );
        }
        let first = &dirs[0];
        pool.query(
            first.path(),
            &first.path().join("a.rs"),
            &query,
            &configs,
            &cancel,
        )
        .await
        .unwrap();
        std::fs::write(dirs[4].path().join("a.rs"), "ok").unwrap();
        pool.query(
            dirs[4].path(),
            &dirs[4].path().join("a.rs"),
            &query,
            &configs,
            &cancel,
        )
        .await
        .unwrap();
        assert_eq!(pool.sessions.lock().await.len(), 4);
        let reused = pool
            .query(
                first.path(),
                &first.path().join("a.rs"),
                &query,
                &configs,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(reused["result"]["pid"], pids[0]);
        let replaced = pool
            .query(
                dirs[1].path(),
                &dirs[1].path().join("a.rs"),
                &query,
                &configs,
                &cancel,
            )
            .await
            .unwrap();
        assert_ne!(replaced["result"]["pid"], pids[1]);
        pool.shutdown().await;
    }
    #[tokio::test]
    async fn shutdown_interrupts_active_queries_and_allows_new_configuration() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.rs");
        std::fs::write(&file, "SLOW").unwrap();
        let pool = Arc::new(LanguageServers::default());
        let worker_pool = pool.clone();
        let workspace = dir.path().to_owned();
        let path = file.clone();
        let worker = tokio::spawn(async move {
            worker_pool
                .query(
                    &workspace,
                    &path,
                    &json!({"operation":"hover"}),
                    &fixture_config(),
                    &CancellationToken::new(),
                )
                .await
        });
        tokio::time::timeout(Duration::from_secs(5), async {
            while !dir.path().join("slow-started").exists() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        pool.shutdown().await;
        assert!(
            tokio::time::timeout(Duration::from_secs(2), worker)
                .await
                .unwrap()
                .unwrap()
                .is_err()
        );
        std::fs::write(&file, "new configuration").unwrap();
        let result = pool
            .query(
                dir.path(),
                &file,
                &json!({"operation":"hover"}),
                &fixture_config(),
                &CancellationToken::new(),
            )
            .await
            .unwrap();
        assert_eq!(result["result"]["contents"], "new configuration");
        pool.shutdown().await;
    }
    #[tokio::test]
    async fn stdio_lifecycle_sync_diagnostics_and_project_isolation() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        let file = a.path().join("unicode ü.rs");
        let other = b.path().join("b.rs");
        std::fs::write(&file, "BROKEN 🦀\n").unwrap();
        std::fs::write(&other, "other project").unwrap();
        let pool = LanguageServers::default();
        let cancel = CancellationToken::new();
        let configs = fixture_config();
        let query = json!({"operation":"hover","line":1,"character":8});
        let first = pool
            .query(a.path(), &file, &query, &configs, &cancel)
            .await
            .unwrap();
        assert_eq!(first["result"]["position"]["character"], 7);
        let diagnostics = pool
            .query(
                a.path(),
                &file,
                &json!({"operation":"diagnostics"}),
                &configs,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(diagnostics["diagnostics"]["items"][0]["severity"], 1);
        std::fs::write(&file, "fixed").unwrap();
        let second = pool
            .query(a.path(), &file, &query, &configs, &cancel)
            .await
            .unwrap();
        assert_eq!(second["result"]["version"], 2);
        assert_eq!(first["result"]["pid"], second["result"]["pid"]);
        let clean = pool
            .query(
                a.path(),
                &file,
                &json!({"operation":"diagnostics"}),
                &configs,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(clean["diagnostics"]["items"], json!([]));
        let different = pool
            .query(b.path(), &other, &query, &configs, &cancel)
            .await
            .unwrap();
        assert_ne!(first["result"]["pid"], different["result"]["pid"]);
        assert!(
            pool.query(a.path(), &file, &query, &[], &cancel)
                .await
                .is_err()
        );
        pool.shutdown().await;
        assert!(pool.sessions.lock().await.is_empty());
    }
    #[tokio::test]
    async fn cancelled_query_kills_session_and_can_start_fresh() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.rs");
        std::fs::write(&file, "SLOW").unwrap();
        let pool = LanguageServers::default();
        let configs = fixture_config();
        let cancel = CancellationToken::new();
        let token = cancel.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            token.cancel();
        });
        assert!(
            pool.query(
                dir.path(),
                &file,
                &json!({"operation":"hover"}),
                &configs,
                &cancel
            )
            .await
            .is_err()
        );
        assert!(pool.sessions.lock().await.is_empty());
        std::fs::write(&file, "fast").unwrap();
        assert!(
            pool.query(
                dir.path(),
                &file,
                &json!({"operation":"hover"}),
                &configs,
                &CancellationToken::new()
            )
            .await
            .is_ok()
        );
        pool.shutdown().await;
    }
}

#[cfg(test)]
mod live_tests {
    use super::*;
    #[tokio::test]
    #[ignore = "Requires FENIX_TEST_PYLSP pointing to a separately installed pylsp executable"]
    async fn real_python_language_server_definitions_and_diagnostics() {
        let command = std::env::var("FENIX_TEST_PYLSP").expect("Set FENIX_TEST_PYLSP");
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("example.py");
        std::fs::write(
            &file,
            "def add(a, b):\n    return a + b\n\nresult = add(1, 2)\n",
        )
        .unwrap();
        let configs = vec![ServerConfig {
            name: "python".into(),
            command: vec![command],
            extensions: vec!["py".into()],
            initialization_options: Value::Null,
        }];
        let pool = LanguageServers::default();
        let cancel = CancellationToken::new();
        let definition = pool
            .query(
                dir.path(),
                &file,
                &json!({"operation":"definition","line":4,"character":10}),
                &configs,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(
            definition["result"][0]["range"]["start"]["line"], 0,
            "{definition}"
        );
        std::fs::write(&file, "def broken(:\n    pass\n").unwrap();
        let mut diagnostics = Value::Null;
        for _ in 0..3 {
            diagnostics = pool
                .query(
                    dir.path(),
                    &file,
                    &json!({"operation":"diagnostics"}),
                    &configs,
                    &cancel,
                )
                .await
                .unwrap();
            if diagnostics["status"] == "reported" {
                break;
            }
        }
        assert_eq!(diagnostics["status"], "reported", "{diagnostics}");
        assert!(
            diagnostics["diagnostics"]["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["severity"] == 1),
            "{diagnostics}"
        );
        pool.shutdown().await;
    }
}
