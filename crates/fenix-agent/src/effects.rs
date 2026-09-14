//! Completion barriers for filesystem work that outlives a cancelled async caller.
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::Notify;

#[derive(Default)]
pub(crate) struct FileEffects {
    active: Mutex<HashMap<String, usize>>,
    changed: Notify,
}

pub(crate) struct FileEffect {
    effects: Arc<FileEffects>,
    task: String,
}

impl FileEffects {
    pub(crate) fn begin(self: &Arc<Self>, task: &str) -> FileEffect {
        *self
            .active
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .entry(task.into())
            .or_default() += 1;
        FileEffect {
            effects: self.clone(),
            task: task.into(),
        }
    }

    pub(crate) async fn wait(&self, task: Option<&str>) {
        loop {
            let changed = self.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            let active = {
                let active = self.active.lock().unwrap_or_else(|e| e.into_inner());
                task.map_or(!active.is_empty(), |task| active.contains_key(task))
            };
            if !active {
                return;
            }
            changed.await;
        }
    }
}

impl Drop for FileEffect {
    fn drop(&mut self) {
        let mut active = self
            .effects
            .active
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(count) = active.get_mut(&self.task) {
            *count -= 1;
            if *count == 0 {
                active.remove(&self.task);
            }
        }
        drop(active);
        self.effects.changed.notify_waiters();
    }
}

impl crate::Runtime {
    pub(crate) async fn settle_task(&self, task: &str) {
        self.processes.cancel_task(task).await;
        self.file_effects.wait(Some(task)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn cancellation_waits_for_detached_filesystem_effects() {
        let effects = Arc::new(FileEffects::default());
        let task_effects = effects.clone();
        let (started, receive_started) = tokio::sync::oneshot::channel();
        let (release, receive_release) = std::sync::mpsc::channel();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("completed.txt");
        let output = path.clone();
        let caller = tokio::spawn(async move {
            let guard = task_effects.begin("child");
            tokio::task::spawn_blocking(move || {
                let _guard = guard;
                started.send(()).unwrap();
                receive_release.recv().unwrap();
                std::fs::write(output, "completed effect").unwrap();
            })
            .await
            .unwrap();
        });
        receive_started.await.unwrap();
        caller.abort();
        let _ = caller.await;
        assert!(
            tokio::time::timeout(Duration::from_millis(20), effects.wait(Some("child")))
                .await
                .is_err()
        );
        tokio::time::timeout(Duration::from_secs(1), effects.wait(Some("master")))
            .await
            .unwrap();
        release.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(2), effects.wait(Some("child")))
            .await
            .unwrap();
        assert_eq!(std::fs::read_to_string(path).unwrap(), "completed effect");
    }
}
