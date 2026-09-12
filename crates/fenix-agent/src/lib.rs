//! Personal assistant runtime. The web and Telegram are adapters to this same service.

pub mod config;
mod context;
mod engine;
mod files;
mod permissions;
mod processes;
pub mod redaction;
pub mod schedules;
pub mod skills;
pub mod telegram;
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
    pub data_dir: PathBuf,
    pub skills_dir: PathBuf,
    pub redactor: Arc<Redactor>,
    pub stop: CancellationToken,
    pub(crate) model: RwLock<Option<Arc<ModelConnection>>>,
    pub(crate) active: Mutex<Option<ActiveTask>>,
    pub(crate) pending: Mutex<Option<Pending>>,
    pub(crate) configure_lock: Mutex<()>,
    pub(crate) schedule_lock: Mutex<()>,
    pub(crate) telegram_lock: Mutex<()>,
    pub(crate) wake: Notify,
    pub(crate) processes: Processes,
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
        let skills_dir = skills_dir
            .map(Path::to_owned)
            .unwrap_or_else(|| data_dir.join("skills"));
        std::fs::create_dir_all(&skills_dir)?;
        let skills_dir = std::fs::canonicalize(skills_dir)?;
        let runtime = Arc::new(Self {
            store,
            data_dir,
            skills_dir,
            redactor: Arc::new(Redactor::default()),
            stop: CancellationToken::new(),
            model: RwLock::new(None),
            active: Mutex::new(None),
            pending: Mutex::new(None),
            configure_lock: Mutex::new(()),
            schedule_lock: Mutex::new(()),
            telegram_lock: Mutex::new(()),
            wake: Notify::new(),
            processes: Processes::default(),
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
        vec![worker, scheduler, telegram]
    }

    pub async fn submit(&self, text: &str, origin: &str) -> Result<Value> {
        let text = text.trim();
        if text.is_empty() || text.len() > 32_000 {
            bail!("Send 1–32000 bytes of text");
        }
        let text = self.redactor.text(text);
        if let Some(result) = self.handle_control(&text, origin).await? {
            return Ok(result);
        }
        let _config = self.configure_lock.lock().await;
        if self.model.read().await.is_none() {
            bail!("Complete model setup first");
        }
        let task = self.store.enqueue(&text, origin, None)?;
        self.wake.notify_one();
        Ok(json!({"queued":true,"task_id":task.id}))
    }

    /// Human-facing notices are history, but not unpaired tool messages in model context.
    pub fn notify(&self, task: Option<&str>, text: &str) -> Result<()> {
        let text = self.redactor.text(text);
        self.store.append(
            task,
            &Message::text(Role::Assistant, &text),
            true,
            false,
            "agent",
        )?;
        if self
            .telegram_settings()?
            .is_some_and(|s| s.enabled && s.owner_id.is_some())
        {
            self.store.queue_delivery(&text)?;
        }
        Ok(())
    }

    pub async fn status(&self) -> Result<Value> {
        let settings = self.settings()?;
        let pending = self.pending.lock().await.as_ref().map(|p| p.view.clone());
        let active = self.active.lock().await.as_ref().map(|a| a.task_id.clone());
        Ok(
            json!({"ready":self.model.read().await.is_some(),"session_id":self.store.session_id(),"settings":settings,"default_settings":config::Settings::initial(std::env::current_dir()?),"active_task":active,"pending":pending,"timezone":"America/Lima","tool_count":tools::catalog().len(),"telegram":self.telegram_settings()?,"skills_directory":self.skills_dir}),
        )
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.stop.cancel();
        if let Some(active) = self.active.lock().await.as_ref() {
            active.cancel.cancel();
        }
        self.processes.cancel_all().await;
        self.wake.notify_waiters();
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
