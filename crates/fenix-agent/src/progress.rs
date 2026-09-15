//! Replaceable UI snapshots: no token journal, prompts, arguments or reasoning.
use crate::Runtime;
use serde::Serialize;

#[derive(Clone, Debug, Default, Serialize)]
pub struct Progress {
    pub task_id: Option<String>,
    pub response_id: Option<String>,
    pub message_id: Option<String>,
    pub phase: Phase,
    pub tool: Option<String>,
    pub preview: String,
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[default]
    Idle,
    Preparing,
    Generating,
    Tool,
    Waiting,
    Retrying,
    Saved,
}

impl Runtime {
    pub fn subscribe_progress(&self) -> tokio::sync::watch::Receiver<Progress> {
        self.progress.subscribe()
    }

    pub(crate) fn report_progress(&self, task_id: &str, phase: Phase, tool: Option<&str>) {
        let _ = self.store.publish_event(
            Some(task_id),
            "task.progress",
            &serde_json::json!({"phase":phase,"tool":tool}),
        );
        self.bus_wake.notify_waiters();
        if self
            .store
            .task(task_id)
            .is_ok_and(|task| task.origin == "subagent")
        {
            return;
        }
        self.progress.send_replace(Progress {
            task_id: Some(task_id.into()),
            phase,
            tool: tool
                .filter(|name| crate::tools::catalog().iter().any(|t| t.name == *name))
                .map(str::to_owned),
            ..Progress::default()
        });
    }
}
