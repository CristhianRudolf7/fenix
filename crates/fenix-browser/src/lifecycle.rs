//! Browser task ownership and durable suspend/resume. Never replay page actions.
use super::*;

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
    }

    pub(crate) async fn suspend_session(
        &self,
        _profile: &str,
        session: &mut Session,
    ) -> Result<()> {
        if !session.cdp.is_quiet() {
            return Ok(());
        }
        // No native full-process checkpoint exists. Never close an unfinished live form.
        Ok(())
    }

    pub(crate) async fn reconnect_session(&self, session: &mut Session) -> Result<()> {
        let endpoint = read_endpoint(&session.directory)?;
        session.cdp = cdp::Cdp::connect(&endpoint).await?;
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
