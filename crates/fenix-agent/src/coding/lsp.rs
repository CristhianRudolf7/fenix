//! Bounded, lazy stdio LSP sessions, isolated by canonical workspace and server.
//! Config comes from the owner settings, never executable repository instructions.
use crate::files::read_text;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    time::Duration,
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
    sessions: Mutex<BTreeMap<String, Session>>,
}
impl LanguageServers {
    pub async fn shutdown(&self) {
        self.sessions.lock().await.clear();
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
        let text = read_text(path)?;
        let workspace = std::fs::canonicalize(workspace)?;
        let key = format!("{}:{}", workspace.display(), serde_json::to_string(config)?);
        // Waiting for another query is cancellable and bounded as well.
        let mut sessions = tokio::select! {
            _=cancel.cancelled()=>bail!("Language query cancelled"),
            result=tokio::time::timeout(Duration::from_secs(30), self.sessions.lock())=>result.context("Language server pool busy")?,
        };
        if !sessions.contains_key(&key) {
            if sessions.len() >= 4 {
                sessions.pop_first();
            }
            let start =
                tokio::time::timeout(Duration::from_secs(30), Session::start(config, &workspace));
            let session = tokio::select! {_=cancel.cancelled()=>bail!("Language server startup cancelled"),r=start=>r.context("Language server initialization timed out")??};
            sessions.insert(key.clone(), session);
        }
        let result = {
            let session = sessions.get_mut(&key).unwrap();
            tokio::select! {
                _=cancel.cancelled()=>Err(anyhow::anyhow!("Language query cancelled")),
                result=tokio::time::timeout(Duration::from_secs(20),session.query(path,&text,args))=>result.context("Language query timed out").and_then(|r|r),
            }
        };
        // Drop failed streams rather than reusing a transport with unread replies.
        if result.is_err() {
            sessions.remove(&key);
        }
        result
    }
}
struct Session {
    child: Child,
    writer: ChildStdin,
    inbox: mpsc::Receiver<Result<Value>>,
    reader_task: tokio::task::JoinHandle<()>,
    next_id: u64,
    capabilities: Value,
    documents: HashMap<String, (i64, String)>,
    diagnostics: HashMap<String, Value>,
    workspace: PathBuf,
}
impl Drop for Session {
    fn drop(&mut self) {
        self.reader_task.abort();
        if let Some(id) = self.child.id() {
            let _ = nix::sys::signal::killpg(
                nix::unistd::Pid::from_raw(id as i32),
                nix::sys::signal::Signal::SIGKILL,
            );
        }
        let _ = self.child.start_kill();
    }
}
impl Session {
    async fn start(config: &ServerConfig, workspace: &Path) -> Result<Self> {
        let mut command = Command::new(&config.command[0]);
        command
            .args(&config.command[1..])
            .current_dir(workspace)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .process_group(0);
        let mut child = command
            .spawn()
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
            writer,
            inbox,
            reader_task,
            next_id: 0,
            capabilities: Value::Null,
            documents: HashMap::new(),
            diagnostics: HashMap::new(),
            workspace: workspace.into(),
        };
        let uri = file_uri(workspace)?;
        let result=session.request("initialize",json!({"processId":std::process::id(),"rootUri":uri,"workspaceFolders":[{"uri":uri,"name":workspace.file_name().unwrap_or_default().to_string_lossy()}],"capabilities":{"general":{"positionEncodings":["utf-16"]},"textDocument":{"publishDiagnostics":{"versionSupport":true},"diagnostic":{},"synchronization":{"didSave":true}},"workspace":{"configuration":true,"applyEdit":false}},"initializationOptions":config.initialization_options})).await?;
        session.capabilities = result["capabilities"].clone();
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
    async fn query(&mut self, path: &Path, text: &str, args: &Value) -> Result<Value> {
        let uri = self.sync(path, text).await?;
        let operation = args["operation"].as_str().unwrap_or("diagnostics");
        let position = json!({"line":args["line"].as_u64().unwrap_or(1).saturating_sub(1),"character":args["character"].as_u64().unwrap_or(1).saturating_sub(1)});
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
                params = json!({"textDocument":{"uri":uri}});
                "textDocument/documentSymbol"
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
