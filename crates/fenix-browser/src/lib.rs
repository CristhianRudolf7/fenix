//! Local Chromium, persistent owner profiles, isolated task profiles and bounded CDP.
mod actions;
mod cdp;
mod downloads;
mod frames;
mod lifecycle;
mod tools;
mod vault;
mod whatsapp;
use anyhow::{Context, Result, bail};
use base64::{Engine, engine::general_purpose::STANDARD};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicU16, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{
    process::{Child, Command},
    sync::Mutex,
};
use tokio_util::sync::CancellationToken;
pub use tools::catalog;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub executable: String,
    pub max_sessions: usize,
    pub max_tabs: usize,
    pub idle_seconds: u64,
    #[serde(default = "default_hibernate_seconds")]
    pub hibernate_after_seconds: u64,
    pub memory_budget_mib: u64,
}
fn default_hibernate_seconds() -> u64 {
    60
}
impl Default for Config {
    fn default() -> Self {
        Self {
            executable: "/usr/bin/chromium".into(),
            max_sessions: 2,
            max_tabs: 4,
            idle_seconds: 600,
            hibernate_after_seconds: default_hibernate_seconds(),
            memory_budget_mib: 2048,
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct Profile {
    name: String,
    allow_agent: bool,
}
struct Session {
    child: Child,
    _group: ProcessGroup,
    execution: Option<fenix_execution::Lease>,
    directory: PathBuf,
    cdp: cdp::Cdp,
    _lock: File,
    tabs: BTreeMap<String, String>,
    frame_sessions: Mutex<BTreeMap<String, String>>,
    active: String,
    last: Instant,
    temporary: bool,
    width: u32,
    height: u32,
    dpr: f64,
    mobile: bool,
    touch: bool,
    tasks: BTreeMap<String, bool>,
    owner_controlled: bool,
    suspended: bool,
    needs_observation: bool,
    finished: bool,
}
pub struct Browser {
    root: PathBuf,
    config: Config,
    executor: Option<fenix_execution::Executor>,
    sessions: Mutex<BTreeMap<String, Session>>,
    profiles: Mutex<BTreeMap<String, Profile>>,
    control_port: AtomicU16,
    vault_lock: Mutex<()>,
    integration_lock: Mutex<()>,
    starting: Mutex<()>,
}
pub fn write_json(path: &Path, value: &Value) -> Result<()> {
    fenix_execution::fs::write(path, &serde_json::to_vec_pretty(value)?)
}
fn private_dir(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    Ok(())
}
fn identifier(name: &str) -> Result<()> {
    if name.is_empty()
        || name.len() > 80
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        bail!("Use 1–80 ASCII letters, digits, hyphens or underscores for an identifier");
    }
    Ok(())
}
fn text<'a>(v: &'a Value, key: &str, default: &'a str) -> &'a str {
    v[key].as_str().unwrap_or(default)
}
fn number(v: &Value, key: &str, default: f64) -> f64 {
    v[key].as_f64().filter(|n| n.is_finite()).unwrap_or(default)
}
impl Browser {
    pub fn open(data: &Path) -> Result<Arc<Self>> {
        let root = data.join("browser");
        private_dir(&root)?;
        private_dir(&root.join("profiles"))?;
        private_dir(&root.join("test"))?;
        private_dir(&root.join("artifacts"))?;
        private_dir(&root.join("hibernated"))?;
        let path = data.join("browser.json");
        let config: Config = if path.exists() {
            serde_json::from_slice(&std::fs::read(&path)?)?
        } else {
            let c = Config::default();
            write_json(&path, &serde_json::to_value(&c)?)?;
            c
        };
        if !(1..=4).contains(&config.max_sessions)
            || !(1..=10).contains(&config.max_tabs)
            || !(60..=86400).contains(&config.idle_seconds)
            || !(10..=86400).contains(&config.hibernate_after_seconds)
            || !(256..=65536).contains(&config.memory_budget_mib)
        {
            bail!("Invalid browser resource budgets");
        }
        let profiles = if root.join("profiles.json").exists() {
            serde_json::from_slice(&std::fs::read(root.join("profiles.json"))?)?
        } else {
            BTreeMap::new()
        };
        Ok(Arc::new(Self {
            root,
            config,
            executor: fenix_execution::Executor::from_environment()?,
            sessions: Mutex::new(BTreeMap::new()),
            profiles: Mutex::new(profiles),
            control_port: AtomicU16::new(4173),
            vault_lock: Mutex::new(()),
            integration_lock: Mutex::new(()),
            starting: Mutex::new(()),
        }))
    }
    pub fn set_control_port(&self, port: u16) {
        self.control_port.store(port, Ordering::Relaxed);
    }
    fn directory(&self, profile: &str) -> PathBuf {
        self.root
            .join(if profile.starts_with("test-") {
                "test"
            } else {
                "profiles"
            })
            .join(profile)
    }
    fn valid_url(&self, value: &str) -> Result<String> {
        if value == "about:blank" {
            return Ok(value.into());
        }
        if value.len() > 4096 {
            bail!("Browser URL too long");
        }
        let url = url::Url::parse(value).context("Use an absolute HTTP(S) URL")?;
        if !matches!(url.scheme(), "https" | "http")
            || !url.username().is_empty()
            || url.password().is_some()
        {
            bail!("Browser navigation requires HTTP(S) without URL credentials");
        }
        if url.port_or_known_default() == Some(self.control_port.load(Ordering::Relaxed))
            && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))
        {
            bail!("The automated browser cannot open Fenix's control interface");
        }
        Ok(url.to_string())
    }
    pub async fn status(&self) -> Result<Value> {
        let profiles = self.profiles.lock().await.clone();
        let sessions = self.sessions.lock().await;
        Ok(
            json!({"available":Path::new(&self.config.executable).is_file(),"config":self.config,
            "profiles":profiles.values().collect::<Vec<_>>(),"sessions":sessions.iter().map(|(name,s)|json!({"profile":name,"tab":s.active,"tabs":s.tabs.keys().collect::<Vec<_>>(),"temporary":s.temporary,"suspended":s.suspended,"width":s.width,"height":s.height,"dpr":s.dpr,"rss_sum_mib":if s.execution.is_none() {Some(memory_mib(s.child.id().unwrap_or(0)))} else {None}})).collect::<Vec<_>>(),"vault_exists":self.root.join("credentials.enc").exists()}),
        )
    }
    pub async fn profile(&self, name: &str, allow_agent: bool) -> Result<Value> {
        if name == whatsapp::PROFILE {
            let _guard = self.integration_lock.lock().await;
            self.cancel_whatsapp_handoff()?;
            return self.profile_record(name, allow_agent).await;
        }
        self.profile_record(name, allow_agent).await
    }
    async fn profile_record(&self, name: &str, allow_agent: bool) -> Result<Value> {
        identifier(name)?;
        if name.starts_with("test-") {
            bail!("The test- prefix is reserved for isolated test sessions");
        }
        let mut profiles = self.profiles.lock().await;
        if !profiles.contains_key(name) && profiles.len() >= 30 {
            bail!("Profile limit reached (30)");
        }
        profiles.insert(
            name.into(),
            Profile {
                name: name.into(),
                allow_agent,
            },
        );
        write_json(
            &self.root.join("profiles.json"),
            &serde_json::to_value(&*profiles)?,
        )?;
        Ok(json!({"profile":name,"allow_agent":allow_agent}))
    }
    async fn ensure_agent_profile(&self, name: &str) -> Result<()> {
        identifier(name)?;
        let mut profiles = self.profiles.lock().await;
        // Creation must never reauthorize a profile taken over by its owner.
        if profiles.contains_key(name) {
            return Ok(());
        }
        if name.starts_with("test-") || name.starts_with("integration-") {
            bail!("This profile namespace is reserved");
        }
        if profiles.len() >= 30 {
            bail!("Profile limit reached (30)");
        }
        let mut updated = profiles.clone();
        updated.insert(
            name.into(),
            Profile {
                name: name.into(),
                allow_agent: true,
            },
        );
        write_json(
            &self.root.join("profiles.json"),
            &serde_json::to_value(&updated)?,
        )?;
        *profiles = updated;
        Ok(())
    }
    pub async fn execute(
        &self,
        name: &str,
        args: &Value,
        owner: bool,
        task: &str,
        cancel: &CancellationToken,
    ) -> Result<Value> {
        if !catalog().iter().any(|t| t.name == name) {
            bail!("Unknown browser tool");
        }
        if name == "browser_connection" {
            if text(args, "connection", "whatsapp") != "whatsapp" {
                bail!("Unknown browser connection");
            }
            return tokio::select! { biased; _ = cancel.cancelled() => bail!("Browser connection operation cancelled"), result = self.whatsapp_agent(text(args, "action", "status")) => result };
        }
        if name == "browser_tabs" && text(args, "action", "") == "profiles" {
            let profiles = self.profiles.lock().await;
            return Ok(
                json!({"profiles":profiles.values().filter(|p|owner||p.allow_agent).collect::<Vec<_>>(),"default":"Omit profile for a task-isolated test browser"}),
            );
        }
        if cancel.is_cancelled() {
            bail!("Browser operation cancelled");
        }
        let fallback = format!("test-{task}");
        let profile = text(args, "profile", &fallback);
        identifier(profile)?;
        if !owner && name == "browser_open" && !profile.starts_with("test-") {
            self.ensure_agent_profile(profile).await?;
        }
        // Named sessions require the owner's explicit switch. Test sessions are
        // bound to the task; an agent cannot select another worker's test profile.
        if !owner {
            if profile.starts_with("test-") {
                if profile != fallback {
                    bail!("Other tasks' test profiles are not accessible");
                }
            } else if !self
                .profiles
                .lock()
                .await
                .get(profile)
                .is_some_and(|p| p.allow_agent)
            {
                bail!("Ask the owner to authorize agent access to this personal profile first");
            }
        } else if !profile.starts_with("test-") && !self.profiles.lock().await.contains_key(profile)
        {
            bail!("Create the personal profile first");
        }
        if owner
            && !profile.starts_with("test-")
            && (matches!(
                name,
                "browser_open" | "browser_act" | "browser_resize" | "browser_close"
            ) || name == "browser_tabs" && text(args, "action", "list") != "list")
        {
            self.profile(profile, false).await?;
        }
        let mut sessions = tokio::select! {biased; _ = cancel.cancelled()=>bail!("Browser operation cancelled"),s=self.sessions.lock()=>s};
        if !owner
            && !profile.starts_with("test-")
            && !self
                .profiles
                .lock()
                .await
                .get(profile)
                .is_some_and(|p| p.allow_agent)
        {
            bail!("Owner has taken control of this profile");
        }
        if name == "browser_close" && args.get("tab").is_none() {
            if let Some(mut s) = sessions.remove(profile) {
                if !s.close().await {
                    sessions.insert(profile.into(), s);
                    bail!("Browser cleanup is still pending; its files have been preserved");
                }
                if s.temporary {
                    let _ = std::fs::remove_dir_all(self.directory(profile));
                }
            }
            if self.saved_path(profile).exists() {
                let saved: Value = serde_json::from_slice(&fenix_execution::fs::read(
                    &self.saved_path(profile),
                    64 * 1024,
                )?)?;
                if let (Some(executor), Some(id)) = (&self.executor, saved["execution"].as_str()) {
                    executor.cancel_browser(id).await?;
                }
                std::fs::remove_file(self.saved_path(profile))?;
            }
            return Ok(
                json!({"closed":true,"profile":profile,"profile_preserved":!profile.starts_with("test-")}),
            );
        }
        if sessions
            .get_mut(profile)
            .is_some_and(|s| !s.suspended && s.child.try_wait().ok().flatten().is_some())
        {
            if self.saved_path(profile).exists() {
                bail!(
                    "Saved browser could not resume; inspect the recovery failure before starting a new page"
                );
            }
            sessions.remove(profile);
        }
        if !sessions.contains_key(profile) || sessions[profile].suspended {
            // Admission can wait for RAM. Existing sessions must remain available
            // for work and cleanup while another browser is waiting to start.
            drop(sessions);
            let _starting = tokio::select! {biased; _ = cancel.cancelled()=>bail!("Browser startup cancelled"), guard=self.starting.lock()=>guard};
            sessions = self.sessions.lock().await;
            if !sessions.contains_key(profile) {
                if name != "browser_open" && !self.saved_path(profile).exists() {
                    bail!("Open this browser profile first");
                }
                if sessions.values().filter(|s| !s.suspended).count() >= self.config.max_sessions {
                    bail!("Browser session limit reached; close another session first");
                }
                drop(sessions);
                let session = tokio::select! {biased; _ = cancel.cancelled()=>bail!("Browser startup cancelled"),s=async { if self.saved_path(profile).exists() { self.restore_saved(profile).await } else { self.launch(profile).await } }=>s?};
                sessions = self.sessions.lock().await;
                sessions.insert(profile.into(), session);
            } else if sessions[profile].suspended {
                if sessions.values().filter(|s| !s.suspended).count() >= self.config.max_sessions {
                    bail!("Browser session limit reached; close another session first");
                }
                let id = sessions[profile]
                    .execution
                    .as_ref()
                    .context("Missing saved execution")?
                    .id
                    .clone();
                drop(sessions);
                let executor = self
                    .executor
                    .as_ref()
                    .context("Browser restore requires isolation")?;
                let lease = tokio::select! {biased; _ = cancel.cancelled()=>bail!("Browser restore cancelled"), lease=executor.resume_browser(&id)=>lease?};
                sessions = self.sessions.lock().await;
                let session = sessions
                    .get_mut(profile)
                    .context("Browser was closed while waiting to resume")?;
                self.resume_session(profile, session, lease).await?;
            }
        }
        let s = sessions.get_mut(profile).unwrap();
        if !owner
            && !profile.starts_with("test-")
            && !self
                .profiles
                .lock()
                .await
                .get(profile)
                .is_some_and(|p| p.allow_agent)
        {
            bail!("Owner has taken control of this profile");
        }
        if s.needs_observation && s.cdp.is_disconnected() {
            self.reconnect_session(s).await?;
        }
        if owner {
            s.owner_controlled = true;
        } else {
            s.tasks.insert(task.into(), false);
        }
        s.finished = false;
        if s.needs_observation
            && !matches!(name, "browser_observe" | "browser_inspect" | "browser_tabs")
        {
            return Ok(
                json!({"restored":true,"requires_observation":true,"action_executed":false,
                "message":"Browser resumed with its saved page. Observe it before continuing; do not repeat prior submissions."}),
            );
        }
        s.last = Instant::now();
        // Do not automatically repeat a partially executed action on timeout.
        let result = tokio::select! {biased; _ = cancel.cancelled()=>Err(anyhow::anyhow!("Browser operation cancelled; inspect before repeating an action")),r=self.operation(s,profile,name,args)=>r};
        s.last = Instant::now();
        if result.is_ok() && name == "browser_observe" {
            s.needs_observation = false;
        }
        result
    }
    async fn launch(&self, profile: &str) -> Result<Session> {
        let directory = self.directory(profile);
        private_dir(&directory)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .mode(0o600)
            .open(directory.join("fenix.lock"))?;
        lock.try_lock_exclusive()
            .context("This profile is already open in another Fenix process")?;
        private_dir(&directory.join("Default"))?;
        // Native Chromium password storage can silently fall back to plaintext on
        // Linux. Use our encrypted owner vault instead; never change cookie state.
        let preferences = directory.join("Default/Preferences");
        let mut prefs: Value = if preferences.exists() {
            serde_json::from_slice(&fenix_execution::fs::read(&preferences, 16 * 1024 * 1024)?)?
        } else {
            json!({})
        };
        prefs["credentials_enable_service"] = json!(false);
        if !prefs["profile"].is_object() {
            prefs["profile"] = json!({});
        }
        prefs["profile"]["password_manager_enabled"] = json!(false);
        write_json(&preferences, &prefs)?;
        let active_file = directory.join("DevToolsActivePort");
        if active_file.exists() {
            std::fs::remove_file(&active_file)?;
        }
        let download = self.root.join("artifacts").join(profile).join("downloads");
        private_dir(&download)?;
        let mut argv = vec![
            self.config.executable.clone(),
            "--headless=new".into(),
            "--remote-debugging-port=0".into(),
            "--remote-debugging-address=127.0.0.1".into(),
            "--no-first-run".into(),
            "--no-default-browser-check".into(),
            "--disable-background-mode".into(),
            "--disable-sync".into(),
            "--window-size=1280,800".into(),
            "--disk-cache-size=67108864".into(),
            format!("--user-data-dir={}", directory.display()),
            "about:blank".into(),
        ];
        let execution = if let Some(executor) = &self.executor {
            Some(
                executor
                    .create(fenix_execution::Specification::new(
                        &format!("browser:{profile}"),
                        "browser",
                        argv.clone(),
                        &directory,
                        &directory,
                        86400,
                    ))
                    .await?,
            )
        } else {
            None
        };
        let mut command = if let Some(lease) = &execution {
            lease.command()?
        } else {
            let mut command = Command::new(argv.remove(0));
            command
                .args(argv)
                .env_clear()
                .envs(fenix_execution::local_environment(&directory));
            command
        };
        let child = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .process_group(0)
            .spawn()
            .context(
                "Could not start isolated Chromium; verify its executable and sandbox support",
            )?;
        let group = ProcessGroup(child.id().context("Missing browser PID")?);
        let mut child = child;
        let endpoint=tokio::time::timeout(Duration::from_secs(45),async {
            loop {
                if child.try_wait()?.is_some(){bail!("Chromium exited at startup. Its sandbox must be supported; Fenix does not disable it.");}
                if let Ok(value) = fenix_execution::fs::read(&active_file, 4096).and_then(|b| Ok(String::from_utf8(b)?)) {
                    let mut lines = value.lines();
                    if let (Some(port), Some(path)) = (lines.next(), lines.next())
                        && let Ok(port) = port.parse::<u16>()
                        && path.starts_with("/devtools/browser/")
                    { return Ok(format!("ws://127.0.0.1:{port}{path}")); }
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }).await.context("Browser startup timed out")??;
        let cdp = if let Some(lease) = &execution {
            let port = url::Url::parse(&endpoint)?
                .port()
                .context("Missing browser port")?;
            cdp::Cdp::connect_stream(&endpoint, lease.connect(port).await?).await?
        } else {
            cdp::Cdp::connect(&endpoint).await?
        };
        let targets = cdp.call(None, "Target.getTargets", json!({})).await?;
        let id = targets["targetInfos"]
            .as_array()
            .and_then(|a| a.iter().find(|t| t["type"] == "page"))
            .and_then(|t| t["targetId"].as_str())
            .context("Browser has no page")?
            .to_owned();
        let mut s = Session {
            child,
            _group: group,
            execution,
            directory: directory.clone(),
            cdp,
            _lock: lock,
            tabs: BTreeMap::new(),
            frame_sessions: Mutex::new(BTreeMap::new()),
            active: id.clone(),
            last: Instant::now(),
            temporary: profile.starts_with("test-"),
            width: 1280,
            height: 800,
            dpr: 1.0,
            mobile: false,
            touch: false,
            tasks: BTreeMap::new(),
            owner_controlled: false,
            suspended: false,
            needs_observation: false,
            finished: false,
        };
        self.attach(&mut s, &id).await?;
        let download = self.root.join("artifacts").join(profile).join("downloads");
        private_dir(&download)?;
        s.cdp
            .call(
                None,
                "Browser.setDownloadBehavior",
                json!({"behavior":"allowAndName","downloadPath":download,"eventsEnabled":true}),
            )
            .await?;
        Ok(s)
    }
    async fn attach(&self, s: &mut Session, id: &str) -> Result<String> {
        if let Some(session) = s.tabs.get(id) {
            return Ok(session.clone());
        }
        let result = s
            .cdp
            .call(
                None,
                "Target.attachToTarget",
                json!({"targetId":id,"flatten":true}),
            )
            .await?;
        let session = result["sessionId"]
            .as_str()
            .context("Missing page session")?
            .to_owned();
        for domain in ["Page.enable", "Runtime.enable", "Network.enable"] {
            s.cdp.call(Some(&session), domain, json!({})).await?;
        }
        if s.directory == self.directory(whatsapp::PROFILE) {
            // WhatsApp's compatibility gate rejects HeadlessChrome. Keep the actual
            // installed version/platform; only normalize the Chrome product name.
            let version = s.cdp.call(None, "Browser.getVersion", json!({})).await?;
            let user_agent = version["userAgent"]
                .as_str()
                .context("Missing browser user agent")?;
            s.cdp
                .call(
                    Some(&session),
                    "Emulation.setUserAgentOverride",
                    json!({
                        "userAgent": user_agent.replace("HeadlessChrome/", "Chrome/")
                    }),
                )
                .await?;
        }
        let port = self.control_port.load(Ordering::Relaxed);
        s.cdp.call(Some(&session),"Network.setBlockedURLs",json!({"urls":[format!("http://127.0.0.1:{port}/*"),format!("http://localhost:{port}/*"),format!("http://[::1]:{port}/*")]})).await?;
        s.cdp
            .call(
                Some(&session),
                "Emulation.setDeviceMetricsOverride",
                json!({"width":s.width,"height":s.height,"deviceScaleFactor":s.dpr,"mobile":s.mobile}),
            )
            .await?;
        s.cdp
            .call(
                Some(&session),
                "Emulation.setTouchEmulationEnabled",
                json!({"enabled":s.touch}),
            )
            .await?;
        s.tabs.insert(id.into(), session.clone());
        Ok(session)
    }
    async fn eval(
        &self,
        s: &Session,
        session: &str,
        frame: Option<&str>,
        expression: String,
    ) -> Result<Value> {
        let world = self.world(s, session, frame).await?;
        let expression = format!("{}\n{}", include_str!("dom.js"), expression);
        let result=s.cdp.call(Some(&world.session),"Runtime.evaluate",json!({"expression":expression,"contextId":world.id,"returnByValue":true,"awaitPromise":true,"timeout":5000})).await?;
        if result.get("exceptionDetails").is_some() {
            bail!("Page inspection failed; select a valid frame and observe again");
        }
        Ok(result["result"]["value"].clone())
    }
    async fn operation(
        &self,
        s: &mut Session,
        profile: &str,
        name: &str,
        args: &Value,
    ) -> Result<Value> {
        if s.active.is_empty() {
            let r = s
                .cdp
                .call(None, "Target.createTarget", json!({"url":"about:blank"}))
                .await?;
            s.active = r["targetId"].as_str().context("Missing tab")?.into();
        }
        let tab = text(args, "tab", &s.active).to_owned();
        let session = if name == "browser_tabs" && text(args, "action", "list") == "list" {
            String::new()
        } else {
            self.attach(s, &tab).await?
        };
        let frame = args["frame"].as_str();
        if let Some(dialog) = s.cdp.dialog(&session)
            && matches!(name, "browser_observe" | "browser_inspect")
        {
            return Ok(
                json!({"profile":profile,"tab":tab,"dialog":dialog,"next":"Handle the JavaScript dialog before observing the page"}),
            );
        }
        match name {
            "browser_open" => {
                let url = self.valid_url(text(args, "url", "about:blank"))?;
                let result = s
                    .cdp
                    .call(Some(&session), "Page.navigate", json!({"url":url}))
                    .await?;
                if result.get("errorText").is_some() {
                    bail!("Browser navigation failed");
                }
                Ok(json!({"profile":profile,"tab":tab,"opened":true}))
            }
            "browser_tabs" => {
                match text(args, "action", "list") {
                    "new" => {
                        if s.tabs.len() >= self.config.max_tabs {
                            bail!("Tab limit reached");
                        }
                        let url = self.valid_url(text(args, "url", "about:blank"))?;
                        let r = s
                            .cdp
                            .call(None, "Target.createTarget", json!({"url":url}))
                            .await?;
                        let id = r["targetId"].as_str().context("Missing new tab")?;
                        self.attach(s, id).await?;
                        s.active = id.into();
                    }
                    "select" => {
                        s.cdp
                            .call(None, "Target.activateTarget", json!({"targetId":tab}))
                            .await?;
                        s.active = tab.clone();
                    }
                    "list" => {}
                    _ => bail!("Unknown tab action"),
                }
                let targets = s.cdp.call(None, "Target.getTargets", json!({})).await?;
                let live = targets["targetInfos"]
                    .as_array()
                    .context("Invalid tab list")?
                    .iter()
                    .filter(|t| t["type"] == "page")
                    .filter_map(|t| t["targetId"].as_str())
                    .collect::<Vec<_>>();
                s.tabs.retain(|id, _| live.contains(&id.as_str()));
                let mut tabs = vec![];
                for target in targets["targetInfos"]
                    .as_array()
                    .context("Invalid tab list")?
                {
                    if target["type"] != "page" {
                        continue;
                    }
                    let id = target["targetId"].as_str().unwrap_or("");
                    if !s.tabs.contains_key(id) && s.tabs.len() >= self.config.max_tabs {
                        s.cdp
                            .call(None, "Target.closeTarget", json!({"targetId":id}))
                            .await?;
                        continue;
                    }
                    self.attach(s, id).await?;
                    tabs.push(json!({"tab":id,"title":target["title"],"url":clean_url(target["url"].as_str().unwrap_or(""))}));
                }
                let ids = tabs
                    .iter()
                    .filter_map(|t| t["tab"].as_str())
                    .collect::<Vec<_>>();
                s.tabs.retain(|id, _| ids.contains(&id.as_str()));
                if !s.tabs.contains_key(&s.active) {
                    s.active = ids.first().copied().unwrap_or("").to_owned();
                }
                Ok(json!({"tabs":tabs,"active":s.active}))
            }
            "browser_close" => {
                s.cdp
                    .call(None, "Target.closeTarget", json!({"targetId":tab}))
                    .await?;
                s.tabs.remove(&tab);
                if s.active == tab {
                    s.active = s.tabs.keys().next().cloned().unwrap_or_default();
                }
                Ok(json!({"closed":tab}))
            }
            "browser_resize" => {
                let width = number(args, "width", 1280.) as u32;
                let height = number(args, "height", 800.) as u32;
                let dpr = number(args, "dpr", 1.);
                if !(240..=3840).contains(&width)
                    || !(240..=2160).contains(&height)
                    || !(1.0..=3.0).contains(&dpr)
                {
                    bail!("Viewport or device scale factor exceeds supported limits");
                }
                s.cdp.call(Some(&session),"Emulation.setDeviceMetricsOverride",json!({"width":width,"height":height,"deviceScaleFactor":dpr,"mobile":args["mobile"].as_bool().unwrap_or(false)})).await?;
                s.cdp
                    .call(
                        Some(&session),
                        "Emulation.setTouchEmulationEnabled",
                        json!({"enabled":args["touch"].as_bool().unwrap_or(false)}),
                    )
                    .await?;
                for other in s.tabs.values().filter(|v| **v != session) {
                    s.cdp.call(Some(other),"Emulation.setDeviceMetricsOverride",json!({"width":width,"height":height,"deviceScaleFactor":dpr,"mobile":args["mobile"].as_bool().unwrap_or(false)})).await?;
                }
                for other in s.tabs.values().filter(|v| **v != session) {
                    s.cdp
                        .call(
                            Some(other),
                            "Emulation.setTouchEmulationEnabled",
                            json!({"enabled":args["touch"].as_bool().unwrap_or(false)}),
                        )
                        .await?;
                }
                s.mobile = args["mobile"].as_bool().unwrap_or(false);
                s.touch = args["touch"].as_bool().unwrap_or(false);
                s.width = width;
                s.height = height;
                s.dpr = dpr;
                Ok(json!({"width":width,"height":height,"dpr":dpr}))
            }
            "browser_observe" => {
                let expression = include_str!("observe.js").replace(
                    "TOKEN",
                    &json!(uuid::Uuid::new_v4().to_string()).to_string(),
                );
                let mut result = self.eval(s, &session, frame, expression).await?;
                if args["screenshot"] == true {
                    result["screenshot"] = self.capture(s, &session, profile, args, false).await?;
                }
                result["profile"] = json!(profile);
                result["tab"] = json!(tab);
                Ok(result)
            }
            "browser_inspect" => {
                if matches!(text(args, "action", "state"), "capture" | "baseline") {
                    return self
                        .capture(
                            s,
                            &session,
                            profile,
                            args,
                            text(args, "action", "") == "baseline",
                        )
                        .await;
                }
                let frames = json!({"frameTree":self.frame_set(s,&session).await?.tree});
                let layout = s
                    .cdp
                    .call(Some(&session), "Page.getLayoutMetrics", json!({}))
                    .await?;
                let events = s.cdp.events.lock().unwrap().clone();
                let mut downloads = self.downloads(profile)?;
                downloads["transfers"] = json!(s.cdp.downloads.lock().unwrap().values());
                let mut safe_frames = frames;
                strip_urls(&mut safe_frames);
                Ok(
                    json!({"frames":safe_frames,"layout":layout,"events":events,"profile":profile,"tab":tab,"downloads":downloads,"rss_sum_mib":if s.execution.is_none() {Some(memory_mib(s.child.id().unwrap_or(0)))} else {None},"warning":"Failure metadata omits sensitive payloads. RSS sums may double-count shared pages."}),
                )
            }
            "browser_act" => {
                if text(args, "action", "") == "dialog" {
                    self.act(s, &session, frame, args).await
                } else {
                    // Native input may not acknowledge until a modal closes. Release the
                    // browser lock promptly so the next command can handle that modal.
                    tokio::select! {
                        biased;
                        dialog = s.cdp.wait_dialog(&session) => Ok(json!({"performed":false,"interrupted_by_dialog":true,"dialog":dialog,"next":"Handle the dialog, then inspect; do not repeat the action automatically"})),
                        result = self.act(s, &session, frame, args) => result,
                    }
                }
            }
            "browser_check" | "browser_wait" => {
                if text(args, "condition", "") == "screenshot_matches" {
                    let name = text(args, "name", "");
                    identifier(name)?;
                    let baseline = self
                        .root
                        .join("artifacts")
                        .join(profile)
                        .join(format!("baseline-{name}.png"));
                    let before = std::fs::read(baseline)
                        .context("Create an explicitly approved baseline first")?;
                    let now = self.image(s, &session, false).await?;
                    return Ok(
                        json!({"passed":before==now,"comparison":"exact PNG equality; same viewport, fonts and animation state required"}),
                    );
                }
                let end = Instant::now()
                    + Duration::from_millis(
                        number(args, "timeout_ms", 10000.).clamp(0., 30000.) as u64
                    );
                loop {
                    let result = self.condition(s, profile, &session, frame, args).await?;
                    if name == "browser_check"
                        || result["passed"] == true
                        || result["terminal"] == true
                    {
                        return Ok(result);
                    }
                    if Instant::now() >= end {
                        if text(args, "condition", "") == "download" {
                            let mut pending = result;
                            pending["timed_out"] = json!(true);
                            pending["next"] = json!(
                                "Inspect transfer progress or wait again for this ID; do not click the download again. A pending transfer is not a completed file."
                            );
                            return Ok(pending);
                        }
                        bail!("Browser condition did not become true before timeout");
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
            _ => bail!("Unknown browser operation"),
        }
    }
    async fn condition(
        &self,
        s: &Session,
        profile: &str,
        session: &str,
        frame: Option<&str>,
        args: &Value,
    ) -> Result<Value> {
        let condition = text(args, "condition", "");
        if matches!(condition, "visible" | "hidden") && text(args, "selector", "").is_empty() {
            bail!("A selector is required");
        }
        if matches!(condition, "text" | "url") && text(args, "text", "").is_empty() {
            bail!("Nonempty text is required");
        }
        if condition == "download" {
            let guid = text(args, "download_id", "");
            uuid::Uuid::parse_str(guid).context("Invalid download ID")?;
            let mut transfer = s
                .cdp
                .downloads
                .lock()
                .unwrap()
                .get(guid)
                .unwrap_or_else(|| json!({"id":guid,"state":"unknown"}));
            let file = self
                .download_path(profile, guid)
                .ok()
                .and_then(|path| std::fs::metadata(path).ok());
            let passed = file.is_some()
                && matches!(transfer["state"].as_str(), Some("completed" | "unknown"));
            if let Some(file) = file.filter(|_| passed) {
                transfer["state"] = json!("completed");
                transfer["file"] = json!({"id":guid,"bytes":file.len(),"url":format!("/api/browser/downloads/{profile}/{guid}")});
            }
            return Ok(
                json!({"profile":profile,"passed":passed,"condition":condition,"terminal":passed || transfer["state"] == "canceled","download":transfer}),
            );
        }
        if !["visible", "hidden", "text", "url", "ready", "no_overflow"].contains(&condition) {
            bail!("Unknown browser condition");
        }
        let expression = include_str!("condition.js").replace("CONDITION", &args.to_string());
        self.eval(s, session, frame, expression).await
    }
    async fn image(&self, s: &Session, session: &str, full: bool) -> Result<Vec<u8>> {
        let mut args = json!({"format":"png","captureBeyondViewport":full});
        if full {
            let layout = s
                .cdp
                .call(Some(session), "Page.getLayoutMetrics", json!({}))
                .await?;
            let size = &layout["cssContentSize"];
            let w = number(size, "width", s.width as f64);
            let h = number(size, "height", s.height as f64);
            if w * h * s.dpr * s.dpr > 16_000_000. {
                bail!("Full-page screenshot exceeds 16 megapixels; capture the viewport instead");
            }
            args["clip"] = json!({"x":0,"y":0,"width":w,"height":h,"scale":1});
        }
        if s.width as f64 * s.height as f64 * s.dpr * s.dpr > 16_000_000. {
            bail!("Screenshot exceeds 16 megapixels");
        }
        let result = s
            .cdp
            .call(Some(session), "Page.captureScreenshot", args)
            .await?;
        let data = result["data"]
            .as_str()
            .context("Browser returned no screenshot")?;
        if data.len() > 12 * 1024 * 1024 {
            bail!("Screenshot exceeds size budget");
        }
        Ok(STANDARD.decode(data)?)
    }
    async fn capture(
        &self,
        s: &Session,
        session: &str,
        profile: &str,
        args: &Value,
        baseline: bool,
    ) -> Result<Value> {
        let name = if baseline {
            let n = text(args, "name", "");
            identifier(n)?;
            format!("baseline-{n}")
        } else {
            uuid::Uuid::new_v4().to_string()
        };
        let directory = self.root.join("artifacts").join(profile);
        private_dir(&directory)?;
        if std::fs::read_dir(&directory)?.count() >= 100 && !baseline {
            bail!("Screenshot artifact limit reached for this profile (100)");
        }
        let bytes = self.image(s, session, args["full_page"] == true).await?;
        let file = format!("{name}.png");
        fenix_execution::fs::write(&directory.join(&file), &bytes)?;
        Ok(
            json!({"artifact":file,"profile":profile,"url":format!("/api/browser/artifacts/{profile}/{file}"),"bytes":bytes.len()}),
        )
    }
    pub async fn screenshot(&self, profile: &str, tab: Option<&str>) -> Result<Vec<u8>> {
        let mut sessions = self.sessions.lock().await;
        let s = sessions
            .get_mut(profile)
            .context("Browser profile is closed")?;
        if s.suspended {
            bail!("The browser is saved to disk. Open it to continue.");
        }
        s.last = Instant::now();
        let tab = tab.unwrap_or(&s.active);
        let session = s.tabs.get(tab).context("Unknown tab")?;
        self.image(s, session, false).await
    }
    pub fn artifact(&self, profile: &str, name: &str) -> Result<Vec<u8>> {
        identifier(profile)?;
        let stem = name
            .strip_suffix(".png")
            .context("Only PNG artifacts can be read")?;
        identifier(stem)?;
        let path = self.root.join("artifacts").join(profile).join(name);
        if std::fs::metadata(&path)?.len() > 12 * 1024 * 1024 {
            bail!("Artifact too large");
        }
        fenix_execution::fs::read(&path, 32 * 1024 * 1024)
    }
    pub fn downloads(&self, profile: &str) -> Result<Value> {
        identifier(profile)?;
        let directory = self.root.join("artifacts").join(profile).join("downloads");
        let mut files = vec![];
        if directory.is_dir() {
            for entry in std::fs::read_dir(directory)?.take(100) {
                let entry = entry?;
                let name = entry.file_name().to_string_lossy().to_string();
                if uuid::Uuid::parse_str(&name).is_ok() && entry.file_type()?.is_file() {
                    files.push(json!({"id":name,"bytes":entry.metadata()?.len(),"url":format!("/api/browser/downloads/{profile}/{name}")}));
                }
            }
        }
        Ok(json!({"files":files}))
    }
    pub fn download_path(&self, profile: &str, id: &str) -> Result<PathBuf> {
        identifier(profile)?;
        uuid::Uuid::parse_str(id).context("Invalid download ID")?;
        let path = self
            .root
            .join("artifacts")
            .join(profile)
            .join("downloads")
            .join(id);
        if !std::fs::symlink_metadata(&path)?.file_type().is_file() {
            bail!("Invalid download file");
        }
        Ok(path)
    }
    pub async fn shutdown(&self) {
        let mut sessions = self.sessions.lock().await;
        for (name, mut s) in std::mem::take(&mut *sessions) {
            if s.suspended {
                continue;
            }
            let closed = s.close().await;
            if closed && s.temporary {
                let _ = std::fs::remove_dir_all(self.directory(&name));
            }
        }
    }
    pub async fn maintain(self: Arc<Self>, stop: CancellationToken) {
        loop {
            tokio::select! {_ = stop.cancelled()=>break,_ = tokio::time::sleep(Duration::from_secs(5))=>{}}
            let mut sessions = self.sessions.lock().await;
            // Native popups can create targets without going through browser_tabs.
            for s in sessions.values_mut() {
                if s.suspended {
                    continue;
                }
                if let Ok(targets) = s.cdp.call(None, "Target.getTargets", json!({})).await {
                    let mut pages = targets["targetInfos"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter(|t| t["type"] == "page")
                        .filter_map(|t| t["targetId"].as_str())
                        .collect::<Vec<_>>();
                    pages.sort_by_key(|id| (*id != s.active, !s.tabs.contains_key(*id)));
                    let mut kept = 0;
                    for id in &pages {
                        if kept >= self.config.max_tabs {
                            let _ = s
                                .cdp
                                .call(None, "Target.closeTarget", json!({"targetId":id}))
                                .await;
                            s.tabs.remove(*id);
                        } else {
                            kept += 1;
                        }
                    }
                    s.tabs.retain(|id, _| pages.contains(&id.as_str()));
                    if !pages.contains(&s.active.as_str()) {
                        s.active = pages.first().copied().unwrap_or("").to_owned();
                    }
                }
            }
            let mut close = vec![];
            for (name, session) in sessions.iter_mut() {
                if session.suspended || name.starts_with("integration-") {
                    continue;
                }
                if session.finished && !session.cdp.downloads.lock().unwrap().has_pending() {
                    close.push(name.clone());
                    continue;
                }
                let waiting =
                    !session.tasks.is_empty() && session.tasks.values().all(|value| *value);
                let eligible = waiting
                    && session.last.elapsed().as_secs() >= self.config.hibernate_after_seconds
                    || session.owner_controlled
                        && session.tasks.is_empty()
                        && session.last.elapsed().as_secs() >= self.config.idle_seconds;
                if eligible {
                    let _ = self.suspend_session(name, session).await;
                    // Back off unavailable snapshots without discarding live state.
                    session.last = Instant::now();
                }
            }
            for name in close {
                if let Some(mut session) = sessions.remove(&name) {
                    if !session.close().await {
                        sessions.insert(name, session);
                        continue;
                    }
                    let _ = std::fs::remove_file(self.saved_path(&name));
                    if session.temporary {
                        let _ = std::fs::remove_dir_all(self.directory(&name));
                    }
                }
            }
            drop(sessions);
            self.maintain_whatsapp().await;
        }
    }
}
impl Session {
    async fn close(&mut self) -> bool {
        if self.suspended {
            if let Some(lease) = &self.execution {
                return lease.cancel().await.is_ok();
            }
            return false;
        }
        let _ = self.cdp.call(None, "Browser.close", json!({})).await;
        if tokio::time::timeout(Duration::from_secs(5), self.child.wait())
            .await
            .is_err()
        {
            let _ = self.child.kill().await;
        }
        if let Some(lease) = &self.execution {
            return lease.cancel().await.is_ok();
        }
        true
    }
}
struct ProcessGroup(u32);
impl Drop for ProcessGroup {
    fn drop(&mut self) {
        if self.0 == 0 {
            return;
        }
        let _ = nix::sys::signal::killpg(
            nix::unistd::Pid::from_raw(self.0 as i32),
            nix::sys::signal::Signal::SIGKILL,
        );
    }
}
fn clean_url(value: &str) -> String {
    url::Url::parse(value)
        .map(|mut u| {
            u.set_query(None);
            u.set_fragment(None);
            let _ = u.set_username("");
            let _ = u.set_password(None);
            u.to_string()
        })
        .unwrap_or_default()
}
fn strip_urls(v: &mut Value) {
    match v {
        Value::Object(o) => {
            for (k, v) in o.iter_mut() {
                if k == "url" {
                    *v = json!(clean_url(v.as_str().unwrap_or("")));
                } else {
                    strip_urls(v);
                }
            }
        }
        Value::Array(a) => {
            for v in a {
                strip_urls(v);
            }
        }
        _ => {}
    }
}
fn memory_mib(root: u32) -> u64 {
    if root == 0 {
        return 0;
    }
    let mut rows = vec![];
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else {
                continue;
            };
            let Ok(status) = std::fs::read_to_string(entry.path().join("status")) else {
                continue;
            };
            let field = |name: &str| {
                status
                    .lines()
                    .find_map(|l| l.strip_prefix(name))
                    .and_then(|v| v.split_whitespace().next()?.parse::<u64>().ok())
                    .unwrap_or(0)
            };
            rows.push((pid, field("PPid:") as u32, field("VmRSS:")));
        }
    }
    let mut ids = vec![root];
    for _ in 0..8 {
        let more = rows
            .iter()
            .filter(|(pid, parent, _)| !ids.contains(pid) && ids.contains(parent))
            .map(|(pid, _, _)| *pid)
            .collect::<Vec<_>>();
        if more.is_empty() {
            break;
        }
        ids.extend(more);
    }
    rows.iter()
        .filter(|(pid, _, _)| ids.contains(pid))
        .map(|(_, _, rss)| rss)
        .sum::<u64>()
        / 1024
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn vault_is_encrypted_and_rejects_wrong_password() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault");
        let entries = vec![vault::Entry {
            profile: "personal".into(),
            origin: "https://example.com".into(),
            username: "alice".into(),
            password: "very-secret-value".into(),
        }];
        vault::save(&path, "correct-master-password", &entries).unwrap();
        let file = std::fs::read_to_string(&path).unwrap();
        assert!(!file.contains("very-secret-value"));
        assert!(!file.contains("alice"));
        assert!(vault::load(&path, "incorrect-password").is_err());
        assert_eq!(
            vault::load(&path, "correct-master-password").unwrap()[0].password,
            "very-secret-value"
        );
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    #[tokio::test]
    async fn profile_authorization_and_url_boundaries() {
        let dir = tempfile::tempdir().unwrap();
        let b = Browser::open(dir.path()).unwrap();
        let c = CancellationToken::new();
        assert!(b.profile("../bad", true).await.is_err());
        assert!(b.profile("test-task", true).await.is_err());
        b.profile("personal", false).await.unwrap();
        assert!(
            b.execute(
                "browser_open",
                &json!({"profile":"personal"}),
                false,
                "task",
                &c
            )
            .await
            .is_err()
        );
        assert!(
            b.execute(
                "browser_open",
                &json!({"profile":"test-other"}),
                false,
                "task",
                &c
            )
            .await
            .is_err()
        );
        assert_eq!(
            b.execute(
                "browser_tabs",
                &json!({"action":"profiles"}),
                false,
                "task",
                &c
            )
            .await
            .unwrap()["profiles"],
            json!([])
        );
        b.profile("personal", true).await.unwrap();
        assert_eq!(
            b.execute(
                "browser_tabs",
                &json!({"action":"profiles"}),
                false,
                "task",
                &c
            )
            .await
            .unwrap()["profiles"][0]["name"],
            "personal"
        );
        assert!(b.valid_url("file:///etc/passwd").is_err());
        assert!(b.valid_url("http://127.0.0.1:4173/api/status").is_err());
        assert!(b.valid_url("http://127.0.0.1:3000/").is_ok());
        assert!(b.artifact("personal", "../credentials.enc").is_err());
    }
    #[tokio::test]
    async fn agent_profile_creation_preserves_owner_control() {
        let dir = tempfile::tempdir().unwrap();
        let b = Browser::open(dir.path()).unwrap();
        b.ensure_agent_profile("shop").await.unwrap();
        assert!(b.profiles.lock().await["shop"].allow_agent);
        b.profile("shop", false).await.unwrap();
        b.ensure_agent_profile("shop").await.unwrap();
        assert!(!b.profiles.lock().await["shop"].allow_agent);
        assert!(b.ensure_agent_profile("../other-user").await.is_err());
        assert!(
            b.ensure_agent_profile("integration-unpaired")
                .await
                .is_err()
        );
        assert!(b.ensure_agent_profile("test-other-task").await.is_err());
    }

    #[tokio::test]
    #[ignore = "Requires sandbox-capable local Chromium"]
    async fn real_chromium_navigation_responsive_and_persistent_login() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            loop {
                let (mut stream, _) = listener.accept().await.unwrap();
                tokio::spawn(async move {
                    let mut buf = [0; 4096];
                    let n = stream.read(&mut buf).await.unwrap_or(0);
                    let restoring = String::from_utf8_lossy(&buf[..n]).starts_with("GET /restore ");
                    let body = r#"<!doctype html><meta name="viewport" content="width=device-width"><style>body{margin:0}main{max-width:100%;overflow-wrap:anywhere}#space{height:1800px}</style><main><h1>Browser fixture</h1><label>Name<input id="name" autocomplete="username"></label><input id="password" type="password"><button id="submit" onclick="document.querySelector('#result').textContent=document.querySelector('#name').value">Submit</button><p id="result"></p><input id="range" type="range" min="0" max="10"><button id="context" oncontextmenu="event.preventDefault();this.textContent='Context opened'">Context</button><input id="check" type="checkbox"><select id="choice"><option value="a">A</option><option value="b">B</option></select><iframe srcdoc="<button>Frame button</button>"></iframe><div id="space"></div><button id="bottom">Bottom</button></main><script>document.cookie='login=persisted; Max-Age=86400; SameSite=Lax';localStorage.setItem('login','persisted');indexedDB.open('session',1).onupgradeneeded=e=>e.target.result.createObjectStore('tokens');</script>"#;
                    let body = if restoring {
                        "<!doctype html><h1>Restored profile</h1>"
                    } else {
                        body
                    };
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                });
            }
        });
        let dir = tempfile::tempdir().unwrap();
        let b = Browser::open(dir.path()).unwrap();
        let c = CancellationToken::new();
        let run = |name: &'static str, mut args: Value| {
            let b = b.clone();
            let c = c.clone();
            async move {
                args["profile"] = json!("personal");
                b.execute(name, &args, false, "task", &c).await.unwrap()
            }
        };
        run("browser_open", json!({"url":url})).await;
        run("browser_wait", json!({"condition":"ready"})).await;
        let observation = run("browser_observe", json!({})).await;
        assert!(observation.to_string().contains("Browser fixture"));
        run(
            "browser_act",
            json!({"action":"fill","selector":"#name","text":"Alice"}),
        )
        .await;
        b.task_waiting("task", true).await;
        {
            let mut sessions = b.sessions.lock().await;
            let session = sessions.get_mut("personal").unwrap();
            // Standalone Chromium has no full-process snapshot backend. It must
            // retain the live form instead of closing it and losing its contents.
            b.suspend_session("personal", session).await.unwrap();
            assert!(!session.suspended);
            let value = b
                .eval(
                    session,
                    &session.tabs[&session.active],
                    None,
                    "document.querySelector('#name').value".into(),
                )
                .await
                .unwrap();
            assert_eq!(value, "Alice");
            // A successful restore sets this flag before accepting any action.
            session.needs_observation = true;
        }
        let blocked = run(
            "browser_act",
            json!({"action":"click","selector":"#submit"}),
        )
        .await;
        assert_eq!(blocked["action_executed"], false);
        assert!(
            run("browser_observe", json!({})).await["text"]
                .as_str()
                .is_some()
        );
        b.task_waiting("task", false).await;
        run(
            "browser_act",
            json!({"action":"click","selector":"#submit"}),
        )
        .await;
        assert_eq!(
            run("browser_check", json!({"condition":"text","text":"Alice"})).await["passed"],
            true
        );
        run(
            "browser_act",
            json!({"action":"fill","selector":"#password","text":"fixture-private-secret"}),
        )
        .await;
        run(
            "browser_act",
            json!({"action":"type","selector":"#password","text":"-suffix"}),
        )
        .await;
        {
            let sessions = b.sessions.lock().await;
            let session = &sessions["personal"];
            assert_eq!(
                b.eval(
                    session,
                    &session.tabs[&session.active],
                    None,
                    "document.querySelector('#password').value".into()
                )
                .await
                .unwrap(),
                "fixture-private-secret-suffix"
            );
        }
        assert!(
            !run("browser_observe", json!({}))
                .await
                .to_string()
                .contains("fixture-private-secret")
        );
        run(
            "browser_act",
            json!({"action":"key","selector":"#password","key":"Control+A"}),
        )
        .await;
        run(
            "browser_act",
            json!({"action":"key","selector":"#password","key":"Backspace"}),
        )
        .await;
        run(
            "browser_act",
            json!({"action":"fill","selector":"#range","text":"7"}),
        )
        .await;
        run(
            "browser_act",
            json!({"action":"right_click","selector":"#context"}),
        )
        .await;
        assert_eq!(
            run(
                "browser_check",
                json!({"condition":"text","text":"Context opened"})
            )
            .await["passed"],
            true
        );
        run("browser_act", json!({"action":"check","selector":"#check"})).await;
        run(
            "browser_act",
            json!({"action":"select","selector":"#choice","value":"b"}),
        )
        .await;
        for (width, height) in [(390, 844), (768, 1024), (1280, 800)] {
            run("browser_resize", json!({"width":width,"height":height})).await;
            assert_eq!(
                run("browser_check", json!({"condition":"no_overflow"})).await["passed"],
                true
            );
            let observed = run("browser_observe", json!({"screenshot":true})).await;
            assert_eq!(observed["viewport"]["width"], width);
            assert!(observed["screenshot"]["bytes"].as_u64().unwrap() > 100);
        }
        b.credentials(json!({"profile":"personal","action":"save","master":"a-long-master-password","username":"Alice","password":"fixture-private-secret"})).await.unwrap();
        b.credentials(
            json!({"profile":"personal","action":"fill","master":"a-long-master-password"}),
        )
        .await
        .unwrap();
        assert!(!b.profiles.lock().await["personal"].allow_agent);
        b.profile("personal", true).await.unwrap();
        let observation = run("browser_observe", json!({})).await.to_string();
        assert!(!observation.contains("fixture-private-secret"));
        let details = run("browser_inspect", json!({})).await;
        let frame = details["frames"]["frameTree"]["childFrames"][0]["frame"]["id"]
            .as_str()
            .unwrap();
        let framed = run("browser_observe", json!({"frame":frame})).await;
        assert!(framed["text"].as_str().unwrap().contains("Frame button"));
        run(
            "browser_act",
            json!({"action":"click","frame":frame,"selector":"button"}),
        )
        .await;
        run("browser_act", json!({"action":"scroll","dy":600})).await;
        run(
            "browser_wait",
            json!({"condition":"visible","selector":"#bottom"}),
        )
        .await;
        assert!(
            run(
                "browser_inspect",
                json!({"action":"capture","full_page":true})
            )
            .await["bytes"]
                .as_u64()
                .unwrap()
                > 100
        );
        run("browser_tabs", json!({"action":"new"})).await;
        assert_eq!(
            run("browser_tabs", json!({})).await["tabs"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        let before = b.status().await.unwrap();
        eprintln!("Browser test RSS: {}", before["sessions"][0]["rss_sum_mib"]);
        // A popup can close itself outside our browser_close tool.
        {
            let sessions = b.sessions.lock().await;
            let s = &sessions["personal"];
            s.cdp
                .call(None, "Target.closeTarget", json!({"targetId":s.active}))
                .await
                .unwrap();
        }
        // Chromium acknowledges closeTarget before removing the target.
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if run("browser_tabs", json!({})).await["tabs"]
                    .as_array()
                    .unwrap()
                    .len()
                    == 1
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        })
        .await
        .unwrap();
        assert!(
            run("browser_observe", json!({})).await["text"]
                .as_str()
                .unwrap()
                .contains("Browser fixture")
        );
        {
            let sessions = b.sessions.lock().await;
            let s = &sessions["personal"];
            let saved = b.eval(s, &s.tabs[&s.active], None, r#"new Promise((resolve,reject)=>{
                const open=indexedDB.open('session');
                open.onerror=()=>reject(Error('Database open failed'));
                open.onsuccess=()=>{const db=open.result;const tx=db.transaction('tokens','readwrite');
                    tx.objectStore('tokens').put('fixture-persisted-token','login');
                    tx.oncomplete=()=>{db.close();resolve(true)};tx.onerror=()=>reject(Error('Database write failed'));
                };
            })"#.into()).await.unwrap();
            assert_eq!(saved, true);
        }
        run("browser_close", json!({})).await;
        assert!(
            b.status().await.unwrap()["sessions"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        run("browser_open", json!({"url":format!("{url}restore")})).await;
        run("browser_wait", json!({"condition":"ready"})).await;
        {
            let sessions = b.sessions.lock().await;
            let s = &sessions["personal"];
            let session = &s.tabs[&s.active];
            let persisted=b.eval(s,session,None,"(async()=>({cookie:document.cookie,local:localStorage.getItem('login'),databases:await indexedDB.databases()}))()".into()).await.unwrap();
            assert!(
                persisted["cookie"]
                    .as_str()
                    .unwrap()
                    .contains("login=persisted")
            );
            assert_eq!(persisted["local"], "persisted");
            let token = b.eval(s, session, None, r#"new Promise((resolve,reject)=>{
                const open=indexedDB.open('session');
                open.onerror=()=>reject(Error('Database open failed'));
                open.onsuccess=()=>{const db=open.result;const request=db.transaction('tokens').objectStore('tokens').get('login');
                    request.onsuccess=()=>{db.close();resolve(request.result)};
                    request.onerror=()=>reject(Error('Database read failed'));
                };
            })"#.into()).await.unwrap();
            assert_eq!(token, "fixture-persisted-token");
            assert!(
                persisted["databases"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|db| db["name"] == "session")
            );
        }
        // Two tasks may share an authorized profile; completing one cannot close
        // a browser still needed by the other. The last completion frees it.
        b.execute(
            "browser_observe",
            &json!({"profile":"personal"}),
            false,
            "second-task",
            &c,
        )
        .await
        .unwrap();
        b.task_finished("task").await;
        assert_eq!(b.sessions.lock().await.len(), 1);
        b.task_finished("second-task").await;
        assert!(b.sessions.lock().await.is_empty());
        assert!(b.directory("personal").exists());
        b.shutdown().await;
        server.abort();
    }
}

#[cfg(test)]
mod adversarial_tests;
