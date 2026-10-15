//! Personal assistant runtime. The web and Telegram are adapters to this same service.

mod active_context;
mod autonomy;
mod coding;
mod command_safety;
mod completion_audits;
mod completion_gate;
pub mod config;
pub mod connectors;
mod content_safety;
mod context;
mod context_budget;
mod conversation;
pub mod decisions;
pub mod designs;
mod documents;
mod effects;
mod engine;
pub mod evaluation;
mod execution_state;
mod file_history;
mod files;
#[cfg(test)]
mod harness_improvements_tests;
pub mod interlocutor;
pub mod library;
pub mod memory;
mod model_health;
pub mod model_roles;
mod operational_context;
mod permissions;
mod plan_reviews;
mod planning;
mod processes;
pub mod progress;
mod project_specs;
mod project_tools;
mod projects;
pub mod redaction;
mod restart_recovery;
pub mod schedules;
pub mod skill_categories;
mod skill_learning;
pub mod skill_localization;
pub mod skills;
mod stop_gate;
mod subagents;
pub mod telegram;
mod tool_discovery;
mod tool_program;
mod tool_receipt;
pub mod tools;
mod usage_tracking;
pub mod workflows;
mod workspace_view;
mod worktrees;

use anyhow::{Context, Result, bail};
use config::ModelConnection;
use fenix_core::{Message, Role};
use fenix_credentials::ApiKey;
use fenix_storage::Store;
use permissions::{ActiveTask, Pending};
use processes::Processes;
use redaction::Redactor;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::{Mutex, Notify, RwLock};
use tokio_util::sync::CancellationToken;

pub struct Runtime {
    pub(crate) health_check: Mutex<()>,
    pub(crate) decisions: decisions::Decisions,
    pub(crate) worktrees: worktrees::Worktrees,
    pub connectors: Arc<fenix_connectors::Connectors>,
    pub(crate) workflow_lock: Mutex<()>,
    pub(crate) workflow_active: Mutex<BTreeMap<String, CancellationToken>>,
    pub(crate) workflow_wake: Notify,
    pub store: Arc<Store>,
    pub web: Arc<fenix_web::Web>,
    pub browser: Arc<fenix_browser::Browser>,
    pub data_dir: PathBuf,
    pub skills_dir: PathBuf,
    pub(crate) translation_notify: tokio::sync::Notify,
    pub(crate) skill_index: std::sync::Mutex<skills::SkillIndex>,
    pub tenant_root: Option<PathBuf>,
    pub redactor: Arc<Redactor>,
    pub stop: CancellationToken,
    pub(crate) interlocutor: interlocutor::Interlocutor,
    pub(crate) library_write: Mutex<()>,
    pub(crate) library_wake: Notify,
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
    _lock: fenix_files::private::Lock,
}

impl Runtime {
    pub async fn open(data_dir: &Path, skills_dir: Option<&Path>) -> Result<Arc<Self>> {
        let cwd = std::env::current_dir()?;
        let selected = fenix_files::resolve(&cwd, data_dir, true)?;
        let data_dir = fenix_files::private::create_private_dir(&selected)?;
        let lock = fenix_files::private::Lock::try_exclusive(&data_dir.join("instance.lock"))
            .context(
                "Another Fenix process is using this directory, or its lock is inaccessible",
            )?;
        let database = data_dir.join("fenix.sqlite3");
        let store = Arc::new(Store::open(&database)?);
        store.migrate_voice_model_records()?;
        let skills_dir = match skills_dir {
            Some(path) => path.to_owned(),
            None if cfg!(target_os = "ios") => data_dir.join("skills"),
            None => skills::default_directory()?,
        };
        let selected = fenix_files::resolve(&cwd, &skills_dir, true)?;
        let skills_dir = fenix_files::private::create_private_dir(&selected)?;
        let tenant_root = None;
        fenix_execution::validate_local_environment()?;
        let redactor = Arc::new(Redactor::default());
        let stop = CancellationToken::new();
        let processes = Processes::new(data_dir.join("processes"), redactor.clone(), stop.clone())?;
        let runtime = Arc::new(Self {
            health_check: Mutex::new(()),
            worktrees: worktrees::Worktrees::default(),
            connectors: Arc::new(fenix_connectors::Connectors::open(&data_dir)?),
            workflow_lock: Mutex::new(()),
            workflow_active: Mutex::new(BTreeMap::new()),
            workflow_wake: Notify::new(),
            decisions: decisions::Decisions::default(),
            store,
            web: fenix_web::Web::open(&data_dir).await?,
            browser: fenix_browser::Browser::open(&data_dir)?,
            data_dir,
            skills_dir,
            translation_notify: tokio::sync::Notify::new(),
            skill_index: std::sync::Mutex::new(skills::SkillIndex::default()),
            tenant_root,
            redactor,
            stop,
            interlocutor: interlocutor::Interlocutor::default(),
            library_write: Mutex::new(()),
            library_wake: Notify::new(),
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
        let (interrupted, recovered) = runtime.recover_restart()?;
        runtime.skip_missed_schedules().await?;
        runtime.recover_workflow_runs()?;
        if interrupted > 0 {
            runtime.notify(None,&format!("El servicio se reinició. {recovered} tarea(s) continúan desde un punto guardado sin repetir acciones. {} tarea(s) requieren revisión de su historial.", interrupted.saturating_sub(recovered)))?;
        }
        Ok(runtime)
    }

    pub(crate) fn validate_user_workspace(&self, path: &Path) -> Result<()> {
        if let Some(root) = &self.tenant_root {
            anyhow::ensure!(
                path.starts_with(root.join("workspace")) || path.starts_with(root.join("projects")),
                "Workspace must belong to the authenticated user"
            );
        }
        Ok(())
    }

    pub fn start(self: &Arc<Self>) -> Vec<tokio::task::JoinHandle<()>> {
        let runtime = self.clone();
        let workflows = tokio::spawn(async move {
            runtime.workflow_worker().await;
        });
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
        let learning = tokio::spawn(async move { runtime.skill_learning_worker().await });
        let runtime = self.clone();
        let memory = tokio::spawn(async move { runtime.memory_worker().await });
        let runtime = self.clone();
        let narrator = tokio::spawn(async move { runtime.narration_worker().await });
        let runtime = self.clone();
        let library = tokio::spawn(async move { runtime.library_worker().await });
        let browser = self.browser.clone();
        let stop = self.stop.clone();
        let browser = tokio::spawn(async move { browser.maintain(stop).await });
        vec![
            worker, scheduler, telegram, subagents, planner, events, browser, memory, library,
            workflows, learning, narrator,
        ]
    }

    pub fn start_model_health(self: &Arc<Self>) -> tokio::task::JoinHandle<()> {
        let runtime = self.clone();
        tokio::spawn(async move { runtime.model_health_worker().await })
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
        let text = self.conversation_text(text);
        self.store
            .publish_message(task, &Message::text(Role::Assistant, &text), false, "agent")?;
        self.bus_wake.notify_waiters();
        Ok(())
    }

    fn publish_once(&self, event_id: &str, task: Option<&str>, text: &str) -> Result<()> {
        self.store.publish_message_once(
            &format!("notice:{event_id}"),
            task,
            &Message::text(Role::Assistant, self.conversation_text(text)),
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
            json!({"ready":self.model.read().await.is_some(),"session_id":self.store.session_id(),"event_cursor":self.store.recent_bus_events(1)?.last().map(|e| e.seq).unwrap_or(0),"settings":settings,"default_settings":config::Settings::initial(std::env::current_dir()?),"active_task":active,"active_tasks":active_tasks,"plans":self.plan_views()?,"pending":pending,"pending_requests":pending_requests,"subagents":self.subagent_views()?,"projects":self.project_catalog()?,"focus_project_id":self.store.conversation_focus()?,"timezone":"America/Lima","tool_count":tools::catalog().len(),"telegram":self.telegram_settings()?,"skills_directory":self.skills_dir}),
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
    let directory =
        fenix_files::private::PrivateDirectory::open(path.parent().context("Parent required")?)?;
    let name = Path::new(path.file_name().context("File name required")?);
    match directory.create(name, content.as_bytes(), false) {
        Ok(()) => Ok(()),
        Err(error)
            if error
                .downcast_ref::<std::io::Error>()
                .is_some_and(|e| e.kind() == std::io::ErrorKind::AlreadyExists) =>
        {
            Ok(())
        }
        Err(error) => Err(error),
    }
}
pub fn write_private(path: &Path, content: &str) -> Result<()> {
    let directory =
        fenix_files::private::PrivateDirectory::open(path.parent().context("Parent required")?)?;
    let name = Path::new(path.file_name().context("File name required")?);
    if directory.exists(name)? {
        let before = directory.read(name, 128 * 1024 * 1024)?;
        directory.replace(name, &before, content.as_bytes())
    } else {
        directory.create(name, content.as_bytes(), false)
    }
}

mod security;
