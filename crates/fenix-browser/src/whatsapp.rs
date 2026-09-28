//! First-party local WhatsApp Web connection. Pairing images are owner-only and ephemeral.
use super::*;

pub(crate) const PROFILE: &str = "integration-whatsapp";
const URL: &str = "https://web.whatsapp.com/";

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct Connection {
    created: bool,
    enabled: bool,
    linked: bool,
    grant_pending: bool,
    tab: Option<String>,
}

impl Browser {
    fn connection(&self) -> Result<Connection> {
        let path = self.root.join("whatsapp.json");
        if !path.exists() {
            return Ok(Connection::default());
        }
        Ok(serde_json::from_slice(&fenix_execution::fs::read(
            &path, 16384,
        )?)?)
    }
    fn save_connection(&self, connection: &Connection) -> Result<()> {
        write_json(
            &self.root.join("whatsapp.json"),
            &serde_json::to_value(connection)?,
        )
    }
    // Called while holding integration_lock; explicit owner control cancels automatic handoff.
    pub(crate) fn cancel_whatsapp_handoff(&self) -> Result<()> {
        let mut connection = self.connection()?;
        if connection.grant_pending {
            connection.grant_pending = false;
            self.save_connection(&connection)?;
        }
        Ok(())
    }
    pub async fn whatsapp_connect(&self) -> Result<Value> {
        self.whatsapp_open_at(URL, true).await
    }
    async fn whatsapp_open_at(&self, url: &str, owner: bool) -> Result<Value> {
        let _guard = self.integration_lock.lock().await;
        let mut connection = self.connection()?;
        let allowed = self
            .profiles
            .lock()
            .await
            .get(PROFILE)
            .is_some_and(|p| p.allow_agent);
        if !owner && (!connection.enabled || !connection.linked || !allowed) {
            bail!("Connect WhatsApp in /integrations and authorize agent access first");
        }
        if owner {
            if !connection.created && self.profiles.lock().await.contains_key(PROFILE) {
                bail!(
                    "The integration profile name is already in use; no existing profile was changed"
                );
            }
            if !connection.created {
                self.profile_record(PROFILE, false).await?;
            }
            connection.created = true;
            connection.enabled = true;
            connection.grant_pending = true;
            self.save_connection(&connection)?;
        }
        let mut sessions = self.sessions.lock().await;
        if sessions
            .get_mut(PROFILE)
            .is_some_and(|s| s.child.try_wait().ok().flatten().is_some())
        {
            sessions.remove(PROFILE);
        }
        if !sessions.contains_key(PROFILE) {
            if sessions.len() >= self.config.max_sessions {
                bail!(
                    "Browser session limit reached; close another profile before connecting WhatsApp"
                );
            }
            sessions.insert(PROFILE.into(), self.launch(PROFILE).await?);
        }
        let s = sessions.get_mut(PROFILE).unwrap();
        let targets = s.cdp.call(None, "Target.getTargets", json!({})).await?;
        let origin = url::Url::parse(url)?.origin().ascii_serialization();
        let existing = targets["targetInfos"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|t| {
                t["type"] == "page"
                    && t["url"]
                        .as_str()
                        .and_then(|u| url::Url::parse(u).ok())
                        .is_some_and(|u| u.origin().ascii_serialization() == origin)
            })
            .and_then(|t| t["targetId"].as_str())
            .map(str::to_owned);
        let tab = if let Some(tab) = existing {
            tab
        } else {
            // The profile belongs solely to this connection. Reuse its active tab;
            // repeated Connect clicks do not create duplicate sessions or QR pages.
            s.active = targets["targetInfos"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|t| t["type"] == "page")
                .and_then(|t| t["targetId"].as_str())
                .unwrap_or("")
                .to_owned();
            self.operation(s, PROFILE, "browser_open", &json!({"url":url}))
                .await?;
            s.active.clone()
        };
        self.attach(s, &tab).await?;
        s.last = Instant::now();
        connection.tab = Some(tab);
        self.save_connection(&connection)?;
        self.whatsapp_snapshot(s, &mut connection, &origin).await
    }
    pub async fn whatsapp_status(&self) -> Result<Value> {
        self.whatsapp_status_at("https://web.whatsapp.com").await
    }
    async fn whatsapp_status_at(&self, origin: &str) -> Result<Value> {
        let _guard = self.integration_lock.lock().await;
        let mut connection = self.connection()?;
        if !connection.enabled {
            return Ok(json!({"id":"whatsapp","state":"disconnected","agent_access":false}));
        }
        let mut sessions = self.sessions.lock().await;
        if sessions
            .get_mut(PROFILE)
            .is_some_and(|s| s.child.try_wait().ok().flatten().is_some())
        {
            sessions.remove(PROFILE);
        }
        if let Some(s) = sessions.get_mut(PROFILE) {
            self.whatsapp_snapshot(s, &mut connection, origin).await
        } else {
            let allowed = self
                .profiles
                .lock()
                .await
                .get(PROFILE)
                .is_some_and(|p| p.allow_agent);
            Ok(
                json!({"id":"whatsapp","state":if connection.linked {"saved"} else {"disconnected"},"profile":PROFILE,"agent_access":allowed,"session_saved":connection.linked}),
            )
        }
    }
    async fn whatsapp_snapshot(
        &self,
        s: &mut Session,
        connection: &mut Connection,
        origin: &str,
    ) -> Result<Value> {
        let Some(tab) = connection.tab.clone() else {
            return Ok(json!({"id":"whatsapp","state":"needs_open","agent_access":false}));
        };
        let targets = s.cdp.call(None, "Target.getTargets", json!({})).await?;
        if !targets["targetInfos"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|t| t["targetId"] == tab)
        {
            return Ok(
                json!({"id":"whatsapp","state":"needs_open","profile":PROFILE,"agent_access":false}),
            );
        }
        let session = self.attach(s, &tab).await?;
        let script = include_str!("whatsapp_probe.js")
            .replace("EXPECTED_ORIGIN", &json!(origin).to_string());
        let probe = self
            .eval(s, &session, None, script)
            .await
            .unwrap_or_else(|_| json!({"state":"loading"}));
        let state = probe["state"].as_str().unwrap_or("loading");
        if state == "ready" {
            let changed = !connection.linked || connection.grant_pending;
            connection.linked = true;
            if connection.grant_pending {
                self.profile_record(PROFILE, true).await?;
                connection.grant_pending = false;
            }
            if changed {
                self.save_connection(connection)?;
            }
        } else if state == "awaiting_qr" && connection.linked {
            connection.linked = false;
            self.profile_record(PROFILE, false).await?;
            self.save_connection(connection)?;
        }
        // Polling connection status never reads chats or restarts an idle browser.
        let allowed = self
            .profiles
            .lock()
            .await
            .get(PROFILE)
            .is_some_and(|p| p.allow_agent);
        Ok(
            json!({"id":"whatsapp","state":state,"profile":PROFILE,"tab":tab,"agent_access":allowed,"session_saved":connection.linked,"qr_available":probe["qr"] == true}),
        )
    }
    pub async fn whatsapp_qr(&self) -> Result<Vec<u8>> {
        self.whatsapp_qr_at("https://web.whatsapp.com").await
    }
    async fn whatsapp_qr_at(&self, origin: &str) -> Result<Vec<u8>> {
        let _guard = self.integration_lock.lock().await;
        let connection = self.connection()?;
        if !connection.enabled {
            bail!("Connect WhatsApp first");
        }
        let mut sessions = self.sessions.lock().await;
        let s = sessions
            .get_mut(PROFILE)
            .context("WhatsApp browser is closed")?;
        let tab = connection
            .tab
            .as_deref()
            .context("WhatsApp tab is unavailable")?;
        let session = self.attach(s, tab).await?;
        let script = format!(
            r#"(()=>{{if(location.origin!=={origin})return null;const q=[...document.querySelectorAll('[data-ref] canvas, [data-testid="qrcode"] canvas, canvas[aria-label*="QR"], canvas[aria-label*="qr"]')].find(e=>e.getClientRects().length&&getComputedStyle(e).visibility!=='hidden'&&e.width>=100&&e.height>=100&&e.width<=1024&&e.height<=1024);return q?q.toDataURL('image/png'):null;}})()"#,
            origin = json!(origin)
        );
        let result = self.eval(s, &session, None, script).await?;
        let encoded = result
            .as_str()
            .and_then(|s| s.strip_prefix("data:image/png;base64,"))
            .context("WhatsApp QR is unavailable or has already been scanned")?;
        if encoded.len() > 512 * 1024 {
            bail!("Unexpected QR image size");
        }
        s.last = Instant::now();
        Ok(STANDARD.decode(encoded)?)
    }
    pub async fn whatsapp_pause(&self) -> Result<Value> {
        let _guard = self.integration_lock.lock().await;
        let mut connection = self.connection()?;
        connection.grant_pending = false;
        self.save_connection(&connection)?;
        if let Some(mut s) = self.sessions.lock().await.remove(PROFILE) {
            s.close().await;
        }
        Ok(json!({"paused":true,"session_saved":connection.linked}))
    }
    pub async fn whatsapp_forget(&self) -> Result<Value> {
        let _guard = self.integration_lock.lock().await;
        let connection = self.connection()?;
        if !connection.created {
            return Ok(json!({"forgotten":true}));
        }
        self.profile_record(PROFILE, false).await?;
        self.save_connection(&Connection {
            created: true,
            ..Default::default()
        })?;
        if let Some(mut s) = self.sessions.lock().await.remove(PROFILE) {
            s.close().await;
        }
        let directory = self.directory(PROFILE);
        if directory.exists() {
            std::fs::remove_dir_all(directory)?;
        }
        Ok(json!({"forgotten":true,"remote_device_revoked":false}))
    }
    pub(crate) async fn whatsapp_agent(&self, action: &str) -> Result<Value> {
        match action {
            "status" => self.whatsapp_status().await,
            "open" => self.whatsapp_open_at(URL, false).await,
            _ => bail!(
                "Connection actions are status or open; pairing is owner-only in /integrations"
            ),
        }
    }
    pub(crate) async fn maintain_whatsapp(&self) {
        // A scanned QR can finish syncing after the owner leaves Integrations.
        // Never open a process automatically or override a manual owner takeover.
        if self
            .connection()
            .is_ok_and(|c| c.enabled && c.grant_pending)
        {
            let _ = self.whatsapp_status().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    struct Fixture {
        _data: tempfile::TempDir,
        browser: Arc<Browser>,
        origin: String,
        server: tokio::task::JoinHandle<()>,
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            self.server.abort();
        }
    }
    impl Fixture {
        async fn new() -> Self {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let origin = format!("http://{}", listener.local_addr().unwrap());
            let server = tokio::spawn(async move {
                loop {
                    let (mut stream, _) = listener.accept().await.unwrap();
                    tokio::spawn(async move {
                        let mut buffer = [0u8; 4096];
                        let _ = stream.read(&mut buffer).await;
                        let body = r#"<!doctype html><title>Pairing fixture</title><main></main><script>
                        function show(state) {
                          localStorage.phase=state;
                          document.querySelector('main').innerHTML=state==='ready'?'<div id="pane-side">PRIVATE CHAT SENTINEL</div>':state==='syncing'?'<progress></progress>':'<div data-ref="PRIVATE QR SENTINEL"><canvas width="228" height="228"></canvas></div>';
                          const canvas=document.querySelector('canvas');
                          if(canvas){const c=canvas.getContext('2d');c.fillStyle='black';c.fillRect(0,0,228,228);c.fillStyle='white';c.fillRect(12,12,40,40);}
                        }
                        show(localStorage.phase||'qr');
                        </script>"#;
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                            body.len()
                        );
                        let _ = stream.write_all(response.as_bytes()).await;
                    });
                }
            });
            let data = tempfile::tempdir().unwrap();
            Self {
                browser: Browser::open(data.path()).unwrap(),
                _data: data,
                origin,
                server,
            }
        }
        async fn oracle(&self, script: &str) -> Value {
            let sessions = self.browser.sessions.lock().await;
            let s = &sessions[PROFILE];
            let tab = self.browser.connection().unwrap().tab.unwrap();
            let result = s
                .cdp
                .call(
                    Some(&s.tabs[&tab]),
                    "Runtime.evaluate",
                    json!({"expression":script,"returnByValue":true}),
                )
                .await
                .unwrap();
            assert!(result.get("exceptionDetails").is_none(), "{result}");
            result["result"]["value"].clone()
        }
        async fn state(&self, expected: &str) -> Value {
            let start = Instant::now();
            loop {
                let value = self.browser.whatsapp_status_at(&self.origin).await.unwrap();
                assert!(!value.to_string().contains("PRIVATE"));
                if value["state"] == expected {
                    return value;
                }
                assert!(
                    start.elapsed() < Duration::from_secs(10),
                    "Expected {expected}, got {value}"
                );
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }

    #[tokio::test]
    async fn whatsapp_status_is_passive_and_pairing_is_owner_only() {
        let data = tempfile::tempdir().unwrap();
        let b = Browser::open(data.path()).unwrap();
        assert_eq!(b.whatsapp_status().await.unwrap()["state"], "disconnected");
        assert!(b.sessions.lock().await.is_empty());
        assert!(b.whatsapp_agent("open").await.is_err());
        for action in ["connect", "qr", "forget"] {
            assert!(b.whatsapp_agent(action).await.is_err());
        }
        assert!(b.whatsapp_qr().await.is_err());
        b.profile(PROFILE, false).await.unwrap();
        assert!(
            b.whatsapp_connect()
                .await
                .unwrap_err()
                .to_string()
                .contains("already in use")
        );
        assert!(b.sessions.lock().await.is_empty());
    }

    #[tokio::test]
    #[ignore = "Requires installed Chromium with its sandbox enabled"]
    async fn whatsapp_pairing_lifecycle_persistence_and_owner_control() {
        let f = Fixture::new().await;
        let b = &f.browser;
        b.whatsapp_open_at(&f.origin, true).await.unwrap();
        let first = f.state("awaiting_qr").await;
        assert_eq!(first["agent_access"], false);
        let png = b.whatsapp_qr_at(&f.origin).await.unwrap();
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert!(b.whatsapp_qr_at("https://wrong.example").await.is_err());
        assert_eq!(
            b.whatsapp_status_at("https://wrong.example").await.unwrap()["state"],
            "unexpected_page"
        );
        // Repeated owner requests reuse the same profile and tab.
        let again = b.whatsapp_open_at(&f.origin, true).await.unwrap();
        assert_eq!(first["tab"], again["tab"]);
        assert_eq!(b.sessions.lock().await.len(), 1);
        f.oracle("show('syncing')").await;
        assert_eq!(f.state("syncing").await["agent_access"], false);
        assert!(b.whatsapp_qr_at(&f.origin).await.is_err());
        f.oracle("show('ready')").await;
        assert_eq!(f.state("ready").await["agent_access"], true);
        let metadata = std::fs::read_to_string(b.root.join("whatsapp.json")).unwrap();
        assert!(!metadata.contains("PRIVATE"));
        assert!(!metadata.contains("data:image"));
        // Routine status polling does not extend process lifetime.
        let last = b.sessions.lock().await[PROFILE].last;
        f.state("ready").await;
        assert_eq!(b.sessions.lock().await[PROFILE].last, last);
        b.whatsapp_pause().await.unwrap();
        assert!(b.sessions.lock().await.is_empty());
        assert_eq!(f.state("saved").await["session_saved"], true);
        let restarted = Browser::open(f._data.path()).unwrap();
        assert_eq!(restarted.whatsapp_status().await.unwrap()["state"], "saved");
        assert!(restarted.sessions.lock().await.is_empty());
        drop(restarted);
        b.whatsapp_open_at(&f.origin, false).await.unwrap();
        f.state("ready").await;
        assert_eq!(f.oracle("localStorage.phase").await, "ready");
        assert!(
            !f.oracle("navigator.userAgent")
                .await
                .as_str()
                .unwrap()
                .contains("HeadlessChrome")
        );
        // Expired login revokes agent access and never masquerades as ready.
        f.oracle("show('qr')").await;
        assert_eq!(f.state("awaiting_qr").await["agent_access"], false);
        assert!(b.whatsapp_open_at(&f.origin, false).await.is_err());
        // Explicit owner takeover during pairing cancels automatic handoff.
        b.whatsapp_open_at(&f.origin, true).await.unwrap();
        b.profile(PROFILE, false).await.unwrap();
        f.oracle("show('ready')").await;
        assert_eq!(f.state("ready").await["agent_access"], false);
        b.whatsapp_open_at(&f.origin, true).await.unwrap();
        assert_eq!(f.state("ready").await["agent_access"], true);
        // Forget removes only the dedicated session, preserving unrelated profiles.
        let other = b.directory("university");
        private_dir(&other).unwrap();
        std::fs::write(other.join("marker"), "preserve").unwrap();
        b.whatsapp_forget().await.unwrap();
        assert!(!b.directory(PROFILE).exists());
        assert!(other.join("marker").exists());
        assert_eq!(f.state("disconnected").await["agent_access"], false);
        b.whatsapp_open_at(&f.origin, true).await.unwrap();
        f.state("awaiting_qr").await;
        b.whatsapp_pause().await.unwrap();
    }

    #[tokio::test]
    #[ignore = "Network smoke test; opens an unlinked WhatsApp page, never scans or sends messages"]
    async fn whatsapp_live_pairing_qr() {
        let data = tempfile::tempdir().unwrap();
        let b = Browser::open(data.path()).unwrap();
        b.whatsapp_connect().await.unwrap();
        let start = Instant::now();
        loop {
            let status = b.whatsapp_status().await.unwrap();
            if status["state"] == "awaiting_qr" {
                let png = b.whatsapp_qr().await.unwrap();
                assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
                assert_eq!(status["agent_access"], false);
                eprintln!(
                    "Real WhatsApp QR available after {:.1}s; image not logged or saved",
                    start.elapsed().as_secs_f32()
                );
                break;
            }
            assert!(
                start.elapsed() < Duration::from_secs(60),
                "WhatsApp did not expose QR: {status}"
            );
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        b.whatsapp_forget().await.unwrap();
    }
}
