//! Personal assistant runtime. The web and Telegram are adapters to this same service.

mod active_context;
mod coding;
pub mod config;
mod context;
mod context_budget;
mod effects;
mod engine;
pub mod evaluation;
mod execution_state;
mod files;
pub mod memory;
mod permissions;
mod plan_reviews;
mod planning;
mod processes;
pub mod progress;
mod project_tools;
mod projects;
pub mod read_only;
pub mod redaction;
pub mod schedules;
pub mod skills;
mod subagents;
pub mod telegram;
mod tool_discovery;
mod tool_receipt;
pub mod tools;

use anyhow::{Result, bail};
use config::ModelConnection;
use fenix_core::{Message, Role};
use fenix_credentials::ApiKey;
use fenix_storage::Store;
use fs2::FileExt;
use permissions::{ActiveTask, Pending};
use processes::Processes;
use redaction::Redactor;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::Write,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::{Mutex, Notify, RwLock};
use tokio_util::sync::CancellationToken;

pub struct Runtime {
    pub store: Arc<Store>,
    pub web: Arc<fenix_web::Web>,
    pub browser: Arc<fenix_browser::Browser>,
    pub data_dir: PathBuf,
    pub skills_dir: PathBuf,
    pub redactor: Arc<Redactor>,
    pub stop: CancellationToken,
    pub(crate) model: RwLock<Option<Arc<ModelConnection>>>,
    pub(crate) active: Mutex<BTreeMap<String, ActiveTask>>,
    pub(crate) plan_lock: Mutex<()>,
    pub(crate) context_lock: Mutex<()>,
    pub(crate) bus_wake: Notify,
    pub(crate) pending: Mutex<BTreeMap<String, Pending>>,
    pub(crate) subagents: subagents::SubagentManager,
    pub(crate) effects: Arc<Mutex<()>>,
    pub(crate) file_effects: Arc<effects::FileEffects>,
    pub(crate) configure_lock: Mutex<()>,
    pub(crate) schedule_lock: Mutex<()>,
    pub(crate) telegram_lock: Mutex<()>,
    pub(crate) wake: Notify,
    pub(crate) processes: Processes,
    pub(crate) language_servers: coding::lsp::LanguageServers,
    pub(crate) progress: tokio::sync::watch::Sender<progress::Progress>,
    _lock: File,
}

impl Runtime {
    pub async fn open(data_dir: &Path, skills_dir: Option<&Path>) -> Result<Arc<Self>> {
        std::fs::create_dir_all(data_dir)?;
        std::fs::set_permissions(data_dir, std::fs::Permissions::from_mode(0o700))?;
        let data_dir = std::fs::canonicalize(data_dir)?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(data_dir.join("instance.lock"))?;
        lock.try_lock_exclusive().map_err(|_| {
            anyhow::anyhow!("Another Fenix process is already using this data directory")
        })?;
        let database = data_dir.join("fenix.sqlite3");
        let store = Arc::new(Store::open(&database)?);
        std::fs::set_permissions(&database, std::fs::Permissions::from_mode(0o600))?;
        let skills_dir = match skills_dir {
            Some(path) => path.to_owned(),
            None => skills::default_directory()?,
        };
        std::fs::create_dir_all(&skills_dir)?;
        let skills_dir = std::fs::canonicalize(skills_dir)?;
        let redactor = Arc::new(Redactor::default());
        let stop = CancellationToken::new();
        let processes = Processes::new(data_dir.join("processes"), redactor.clone(), stop.clone())?;
        let runtime = Arc::new(Self {
            store,
            web: fenix_web::Web::open(&data_dir).await?,
            browser: fenix_browser::Browser::open(&data_dir)?,
            data_dir,
            skills_dir,
            redactor,
            stop,
            model: RwLock::new(None),
            active: Mutex::new(BTreeMap::new()),
            plan_lock: Mutex::new(()),
            context_lock: Mutex::new(()),
            bus_wake: Notify::new(),
            pending: Mutex::new(BTreeMap::new()),
            subagents: subagents::SubagentManager::default(),
            effects: Arc::new(Mutex::new(())),
            file_effects: Arc::new(effects::FileEffects::default()),
            configure_lock: Mutex::new(()),
            schedule_lock: Mutex::new(()),
            telegram_lock: Mutex::new(()),
            wake: Notify::new(),
            processes,
            language_servers: coding::lsp::LanguageServers::default(),
            progress: tokio::sync::watch::channel(progress::Progress::default()).0,
            _lock: lock,
        });
        runtime.install_resources()?;
        for key in runtime.store.records::<String>("credentials")? {
            runtime.redactor.add(&key);
        }
        if let Some(token) = runtime.store.get::<String>("telegram_secret", "token")? {
            runtime.redactor.add(&token);
        }
        if let Some(settings) = runtime.settings()?
            && let Some(key) = runtime
                .store
                .get::<String>("credentials", &settings.credential_id())?
        {
            *runtime.model.write().await = Some(Arc::new(
                runtime.model_connection(settings, ApiKey::new(key)?)?,
            ));
        }
        let interrupted = runtime.store.interrupt_unfinished()?;
        runtime.skip_missed_schedules().await?;
        if interrupted > 0 {
            runtime.notify(None,&format!("El servicio se reinició. {interrupted} tarea(s) quedaron interrumpidas; puedes consultar su historial. Sus acciones no se han reanudado."))?;
        }
        Ok(runtime)
    }

    pub fn start(self: &Arc<Self>) -> Vec<tokio::task::JoinHandle<()>> {
        let runtime = self.clone();
        let worker = tokio::spawn(async move {
            runtime.worker().await;
        });
        let runtime = self.clone();
        let scheduler = tokio::spawn(async move {
            runtime.scheduler().await;
        });
        let runtime = self.clone();
        let telegram = tokio::spawn(async move {
            runtime.telegram_loop().await;
        });
        let runtime = self.clone();
        let subagents = tokio::spawn(async move { runtime.subagent_worker().await });
        let runtime = self.clone();
        let planner = tokio::spawn(async move { runtime.planning_worker().await });
        let runtime = self.clone();
        let events = tokio::spawn(async move { runtime.event_worker().await });
        let runtime = self.clone();
        let memory = tokio::spawn(async move { runtime.memory_worker().await });
        let browser = self.browser.clone();
        let stop = self.stop.clone();
        let browser = tokio::spawn(async move { browser.maintain(stop).await });
        vec![
            worker, scheduler, telegram, subagents, planner, events, browser, memory,
        ]
    }

    /// Start public background crawling explicitly. Library callers and offline
    /// fixtures can start the conversation workers without contacting websites.
    pub fn start_web_search(self: &Arc<Self>) -> tokio::task::JoinHandle<()> {
        let web = self.web.clone();
        let stop = self.stop.clone();
        tokio::spawn(async move { web.run(stop).await })
    }

    pub async fn submit(&self, text: &str, origin: &str) -> Result<Value> {
        self.submit_referenced(text, origin, None).await
    }

    pub async fn submit_referenced(
        &self,
        text: &str,
        origin: &str,
        reply_to_task_id: Option<&str>,
    ) -> Result<Value> {
        let text = text.trim();
        if text.is_empty() || text.len() > 32_000 {
            bail!("Send 1–32000 bytes of text");
        }
        let text = self.redactor.text(text);
        if let Some(result) = self
            .handle_plan_control(&text, origin, reply_to_task_id)
            .await?
        {
            return Ok(result);
        }
        if let Some(result) = self
            .handle_referenced_control(&text, origin, reply_to_task_id)
            .await?
        {
            return Ok(result);
        }
        let _config = self.configure_lock.lock().await;
        if self.model.read().await.is_none() {
            bail!("Complete model setup first");
        }
        let task = self
            .store
            .enqueue_referenced(&text, origin, None, reply_to_task_id)?;
        self.pin_destination(&task.id, origin)?;
        self.wake.notify_one();
        Ok(json!({"queued":true,"task_id":task.id}))
    }

    /// Human-facing notices are history, but not unpaired tool messages in model context.
    pub fn notify(&self, task: Option<&str>, text: &str) -> Result<()> {
        let text = self.redactor.text(text);
        self.store
            .publish_message(task, &Message::text(Role::Assistant, &text), false, "agent")?;
        self.bus_wake.notify_waiters();
        Ok(())
    }

    fn publish_once(&self, event_id: &str, task: Option<&str>, text: &str) -> Result<()> {
        self.store.publish_message_once(
            &format!("notice:{event_id}"),
            task,
            &Message::text(Role::Assistant, self.redactor.text(text)),
            false,
            "agent",
        )?;
        self.bus_wake.notify_waiters();
        Ok(())
    }

    pub(crate) fn pin_destination(&self, task: &str, origin: &str) -> Result<()> {
        if origin == "telegram"
            && let Some(settings) = self.telegram_settings()?
            && let Some(owner) = settings.owner_id
        {
            return self.store.set_task_destination(
                task,
                "telegram",
                Some(&format!("{}:{owner}", settings.username)),
            );
        }
        self.store.set_task_destination(task, "web", None)
    }

    pub async fn status(&self) -> Result<Value> {
        let settings = self.settings()?;
        let pending_requests = self
            .pending
            .lock()
            .await
            .values()
            .map(|p| p.view.clone())
            .collect::<Vec<_>>();
        let pending = pending_requests.first();
        let active_tasks = self.active.lock().await.keys().cloned().collect::<Vec<_>>();
        let active = active_tasks.first();
        Ok(
            json!({"ready":self.model.read().await.is_some(),"session_id":self.store.session_id(),"settings":settings,"default_settings":config::Settings::initial(std::env::current_dir()?),"active_task":active,"active_tasks":active_tasks,"plans":self.plan_views()?,"pending":pending,"pending_requests":pending_requests,"subagents":self.subagent_views()?,"projects":self.project_catalog()?,"focus_project_id":self.store.conversation_focus()?,"timezone":"America/Lima","tool_count":tools::catalog().len(),"telegram":self.telegram_settings()?,"skills_directory":self.skills_dir}),
        )
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.stop.cancel();
        for active in self.active.lock().await.values() {
            active.cancel.cancel();
        }
        self.processes.cancel_all().await;
        self.language_servers.shutdown().await;
        self.browser.shutdown().await;
        self.file_effects.wait(None).await;
        self.wake.notify_waiters();
        self.subagents.wake.notify_waiters();
        Ok(())
    }
}

pub fn write_new(path: &Path, content: &str) -> Result<()> {
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
    {
        Ok(mut file) => {
            file.write_all(content.as_bytes())?;
            file.sync_all()?;
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error.into()),
    }
}
pub fn write_private(path: &Path, content: &str) -> Result<()> {
    let temporary = path.with_extension(format!("{}.tmp", fenix_storage::id()));
    write_new(&temporary, content)?;
    std::fs::rename(&temporary, path)?;
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}
