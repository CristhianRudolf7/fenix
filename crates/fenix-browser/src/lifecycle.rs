//! Browser task ownership and durable suspend/resume. Never replay page actions.
use super::*;

#[derive(Serialize, Deserialize)]
struct SavedSession {
    execution: String,
    active: String,
    width: u32,
    height: u32,
    dpr: f64,
    mobile: bool,
    touch: bool,
    tasks: BTreeMap<String, bool>,
    owner_controlled: bool,
}

impl Browser {
    pub(crate) fn saved_path(&self, profile: &str) -> PathBuf {
        self.root.join("hibernated").join(format!("{profile}.json"))
    }

    pub async fn task_waiting(&self, task: &str, waiting: bool) {
        for session in self.sessions.lock().await.values_mut() {
            if let Some(value) = session.tasks.get_mut(task) {
                *value = waiting;
                session.last = Instant::now();
            }
        }
    }

    pub async fn task_finished(&self, task: &str) {
        let mut sessions = self.sessions.lock().await;
        let mut close = vec![];
        for (profile, session) in sessions.iter_mut() {
            if session.tasks.remove(task).is_some()
                && session.tasks.is_empty()
                && !session.owner_controlled
                && !profile.starts_with("integration-")
            {
                session.finished = true;
                if !session.cdp.downloads.lock().unwrap().has_pending() {
                    close.push(profile.clone());
                }
            }
        }
        for profile in close {
            if let Some(mut session) = sessions.remove(&profile) {
                if !session.close().await {
                    sessions.insert(profile, session);
                    continue;
                }
                let _ = std::fs::remove_file(self.saved_path(&profile));
                if session.temporary {
                    let _ = std::fs::remove_dir_all(&session.directory);
                }
            }
        }
        // The controller may have restarted while the browser was on disk.
        // Finishing its task must not leave that saved execution occupying storage.
        if let (Some(executor), Ok(entries)) = (
            &self.executor,
            std::fs::read_dir(self.root.join("hibernated")),
        ) {
            for entry in entries.flatten() {
                let path = entry.path();
                let Some(profile) = path.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };
                if identifier(profile).is_err()
                    || sessions.contains_key(profile)
                    || profile.starts_with("integration-")
                {
                    continue;
                }
                let Ok(bytes) = fenix_execution::fs::read(&path, 64 * 1024) else {
                    continue;
                };
                let Ok(mut saved) = serde_json::from_slice::<SavedSession>(&bytes) else {
                    continue;
                };
                if saved.tasks.remove(task).is_none() {
                    continue;
                }
                if saved.tasks.is_empty() && !saved.owner_controlled {
                    if executor.cancel_browser(&saved.execution).await.is_ok() {
                        let _ = std::fs::remove_file(&path);
                        if profile.starts_with("test-") {
                            let _ = std::fs::remove_dir_all(self.directory(profile));
                        }
                    }
                } else if let Ok(value) = serde_json::to_value(saved) {
                    let _ = write_json(&path, &value);
                }
            }
        }
    }

    pub(crate) async fn suspend_session(&self, profile: &str, session: &mut Session) -> Result<()> {
        if session.suspended || profile.starts_with("integration-") || !session.cdp.is_quiet() {
            return Ok(());
        }
        let Some(executor) = &self.executor else {
            return Ok(());
        };
        if executor.health().await?["ram_snapshots"] != true {
            return Ok(());
        }
        let lease = session
            .execution
            .as_ref()
            .context("Browser has no execution lease")?;
        let saved = SavedSession {
            execution: lease.id.clone(),
            active: session.active.clone(),
            width: session.width,
            height: session.height,
            dpr: session.dpr,
            mobile: session.mobile,
            touch: session.touch,
            tasks: session.tasks.clone(),
            owner_controlled: session.owner_controlled,
        };
        // Persist controller metadata before requesting the atomic backend checkpoint.
        write_json(&self.saved_path(profile), &serde_json::to_value(&saved)?)?;
        session.cdp.disconnect().await;
        let result = lease.suspend_browser().await;
        if lease.is_suspended() {
            session.suspended = true;
            let _ = tokio::time::timeout(Duration::from_secs(10), session.child.wait()).await;
            session._group.0 = 0;
            session.tabs.clear();
            session.frame_sessions.lock().await.clear();
        } else {
            let _ = std::fs::remove_file(self.saved_path(profile));
            // A failed checkpoint keeps the original page, not a new navigation.
            self.reconnect_session(session).await?;
        }
        result
    }

    pub(crate) async fn reconnect_session(&self, session: &mut Session) -> Result<()> {
        let endpoint = read_endpoint(&session.directory)?;
        let lease = session
            .execution
            .as_ref()
            .context("Missing browser execution")?;
        let port = url::Url::parse(&endpoint)?
            .port()
            .context("Missing browser port")?;
        session.cdp = cdp::Cdp::connect_stream(&endpoint, lease.connect(port).await?).await?;
        session.tabs.clear();
        session.frame_sessions.lock().await.clear();
        let targets = session
            .cdp
            .call(None, "Target.getTargets", json!({}))
            .await?;
        let ids: Vec<String> = targets["targetInfos"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|t| t["type"] == "page")
            .filter_map(|t| t["targetId"].as_str().map(str::to_owned))
            .collect();
        for id in &ids {
            self.attach(session, id).await?;
        }
        if !ids.contains(&session.active) {
            session.active = ids.first().context("Restored browser has no page")?.clone();
        }
        let download = self
            .root
            .join("artifacts")
            .join(session.directory.file_name().context("Missing profile")?)
            .join("downloads");
        session
            .cdp
            .call(
                None,
                "Browser.setDownloadBehavior",
                json!({"behavior":"allowAndName","downloadPath":download,"eventsEnabled":true}),
            )
            .await?;
        // Stale references are invalid after reconnecting. Require a fresh observation.
        session.needs_observation = true;
        Ok(())
    }

    pub(crate) async fn resume_session(
        &self,
        profile: &str,
        session: &mut Session,
        lease: fenix_execution::Lease,
    ) -> Result<()> {
        let child = lease
            .command()?
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .process_group(0)
            .spawn()?;
        session.child = child;
        session._group = ProcessGroup(session.child.id().context("Missing restore PID")?);
        session.execution = Some(lease);
        session.suspended = false;
        session.needs_observation = true;
        // attach completes asynchronously in the internal runner.
        let mut connected = false;
        for _ in 0..120 {
            if session.child.try_wait()?.is_some() {
                bail!("The saved browser could not be restored; do not repeat submissions");
            }
            if self.reconnect_session(session).await.is_ok() {
                connected = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        if !connected {
            bail!("Browser restore connection failed; reconnect before continuing the task");
        }
        session.suspended = false;
        session.last = Instant::now();
        std::fs::remove_file(self.saved_path(profile)).ok();
        Ok(())
    }

    pub(crate) async fn restore_saved(&self, profile: &str) -> Result<Session> {
        let saved: SavedSession = serde_json::from_slice(&fenix_execution::fs::read(
            &self.saved_path(profile),
            64 * 1024,
        )?)?;
        let executor = self
            .executor
            .as_ref()
            .context("Browser restore requires isolation")?;
        let directory = self.directory(profile);
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .mode(0o600)
            .open(directory.join("fenix.lock"))?;
        lock.try_lock_exclusive()
            .context("This browser profile is already in use")?;
        let lease = executor.resume_browser(&saved.execution).await?;
        let child = lease
            .command()?
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .process_group(0)
            .spawn()?;
        let mut child = child;
        let mut connected = None;
        for _ in 0..120 {
            if child.try_wait()?.is_some() {
                bail!("Saved browser restore failed");
            }
            if let Ok(endpoint) = read_endpoint(&directory) {
                let port = url::Url::parse(&endpoint)?
                    .port()
                    .context("Missing browser port")?;
                if let Ok(stream) = lease.connect(port).await {
                    if let Ok(cdp) = cdp::Cdp::connect_stream(&endpoint, stream).await {
                        connected = Some(cdp);
                        break;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        let group = ProcessGroup(child.id().context("Missing browser PID")?);
        let mut session = Session {
            child,
            _group: group,
            execution: Some(lease),
            directory,
            _lock: lock,
            cdp: connected.context("Saved browser could not reconnect")?,
            tabs: BTreeMap::new(),
            frame_sessions: Mutex::new(BTreeMap::new()),
            active: saved.active,
            last: Instant::now(),
            temporary: profile.starts_with("test-"),
            width: saved.width,
            height: saved.height,
            dpr: saved.dpr,
            mobile: saved.mobile,
            touch: saved.touch,
            tasks: saved.tasks,
            owner_controlled: saved.owner_controlled,
            suspended: false,
            needs_observation: true,
            finished: false,
        };
        session.cdp.disconnect().await;
        self.reconnect_session(&mut session).await?;
        std::fs::remove_file(self.saved_path(profile)).ok();
        Ok(session)
    }
}

fn read_endpoint(directory: &Path) -> Result<String> {
    let data = String::from_utf8(fenix_execution::fs::read(
        &directory.join("DevToolsActivePort"),
        4096,
    )?)?;
    let mut lines = data.lines();
    let port: u16 = lines.next().context("Missing browser port")?.parse()?;
    let path = lines.next().context("Missing browser endpoint")?;
    anyhow::ensure!(
        port >= 1024 && path.starts_with("/devtools/browser/"),
        "Invalid browser endpoint"
    );
    Ok(format!("ws://127.0.0.1:{port}{path}"))
}
