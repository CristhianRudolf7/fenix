use crate::Runtime;
use anyhow::{Context, Result, bail};
use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};
use chrono_tz::America::Lima;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::str::FromStr;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScheduleInput {
    pub name: String,
    pub content: String,
    pub mode: String,
    pub kind: String,
    #[serde(default)]
    pub at: Option<String>,
    #[serde(default)]
    pub interval_seconds: Option<u64>,
    #[serde(default)]
    pub cron: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub workflow_id: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: String,
    #[serde(flatten)]
    pub spec: ScheduleInput,
    pub status: String,
    pub next_run: Option<DateTime<Utc>>,
    pub created_at: String,
    pub last_run: Option<DateTime<Utc>>,
    pub last_result: Option<String>,
}

impl ScheduleInput {
    pub fn next(&self, after: DateTime<Utc>) -> Result<Option<DateTime<Utc>>> {
        match self.kind.as_str() {
            "once" => {
                let value = self
                    .at
                    .as_deref()
                    .context("One-time schedules require at")?;
                let date = if let Ok(date) = DateTime::parse_from_rfc3339(value) {
                    date.with_timezone(&Utc)
                } else {
                    let naive = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S")
                        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M"))?;
                    Lima.from_local_datetime(&naive)
                        .single()
                        .context("Invalid local date/time")?
                        .with_timezone(&Utc)
                };
                Ok((date > after).then_some(date))
            }
            "interval" => {
                let seconds = self
                    .interval_seconds
                    .context("Intervals require interval_seconds")?;
                if !(1..=31_536_000).contains(&seconds) {
                    bail!("Interval must be between one second and one year");
                }
                Ok(Some(after + Duration::seconds(seconds as i64)))
            }
            "cron" => {
                let text = self
                    .cron
                    .as_deref()
                    .context("Cron schedules require an expression")?;
                let text = if text.split_whitespace().count() == 5 {
                    format!("0 {text}")
                } else {
                    text.to_owned()
                };
                let schedule =
                    cron::Schedule::from_str(&text).context("Invalid cron expression")?;
                Ok(schedule
                    .after(&after.with_timezone(&Lima))
                    .next()
                    .map(|d| d.with_timezone(&Utc)))
            }
            _ => bail!("Schedule kind must be once, interval or cron"),
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty()
            || self.name.len() > 200
            || self.content.trim().is_empty()
            || self.content.len() > 16_000
        {
            bail!("Supply a schedule name and content within the size limits");
        }
        if !matches!(self.mode.as_str(), "reminder" | "task" | "workflow") {
            bail!("Schedule mode must be reminder, task or workflow");
        }
        anyhow::ensure!(
            self.mode == "workflow" || self.workflow_id.is_none(),
            "Only workflow schedules accept a workflow ID"
        );
        anyhow::ensure!(
            self.mode != "workflow" || self.workflow_id.as_deref().is_some_and(|id| !id.is_empty()),
            "Workflow ID required"
        );
        if self.next(Utc::now())?.is_none() {
            bail!("Schedule must have a future execution");
        }
        Ok(())
    }
}

impl Runtime {
    fn pin_schedule_destination(&self, task: &str, schedule: &str) -> Result<()> {
        if let Some((channel, address)) = self
            .store
            .get::<(String, Option<String>)>("schedule_destinations", schedule)?
        {
            self.store
                .set_task_destination(task, &channel, address.as_deref())
        } else {
            self.pin_destination(task, "telegram")
        }
    }
    pub async fn schedule_manage(&self, mut args: Value) -> Result<Value> {
        let _lock = self.schedule_lock.lock().await;
        let action = args["action"].as_str().unwrap_or("list").to_owned();
        let id = args["id"].as_str().map(str::to_owned);
        let scoped = args.get("project_id").is_some();
        let project_id = args["project_id"].as_str().map(str::to_owned);
        if let Some(project) = &project_id {
            self.store.project(project)?;
        }
        if action == "list" {
            return Ok(json!(
                self.store
                    .records::<Schedule>("schedules")?
                    .into_iter()
                    .filter(|s| !scoped || s.spec.project_id == project_id)
                    .collect::<Vec<_>>()
            ));
        }
        if action != "create" {
            let previous = self
                .store
                .get::<Schedule>("schedules", id.as_deref().context("Schedule ID required")?)?
                .context("Schedule not found")?;
            if scoped && previous.spec.project_id != project_id {
                bail!("Schedule not found in this scope");
            }
            if action == "update" && !scoped {
                args["project_id"] = json!(previous.spec.project_id);
            }
        }
        if action == "get" {
            return Ok(json!(
                self.store
                    .get::<Schedule>("schedules", id.as_deref().context("Schedule ID required")?)?
                    .context("Schedule not found")?
            ));
        }
        if matches!(action.as_str(), "create" | "update") {
            let object = args.as_object_mut().context("Expected schedule object")?;
            object.remove("action");
            object.remove("id");
            self.redactor.value(&mut args);
            let spec: ScheduleInput = serde_json::from_value(args)?;
            spec.validate()?;
            if let Some(id) = &spec.workflow_id {
                anyhow::ensure!(
                    self.workflow(id)?.spec.project_id == spec.project_id,
                    "Workflow and schedule must belong to the same project"
                );
            }
            let previous = if action == "update" {
                Some(
                    self.store
                        .get::<Schedule>(
                            "schedules",
                            id.as_deref().context("Schedule ID required")?,
                        )?
                        .context("Schedule not found")?,
                )
            } else {
                None
            };
            let schedule = Schedule {
                id: previous
                    .as_ref()
                    .map(|s| s.id.clone())
                    .unwrap_or_else(fenix_storage::id),
                status: "active".into(),
                next_run: spec.next(Utc::now())?,
                spec,
                created_at: previous
                    .as_ref()
                    .map(|s| s.created_at.clone())
                    .unwrap_or_else(fenix_storage::now),
                last_run: previous.as_ref().and_then(|s| s.last_run),
                last_result: previous.and_then(|s| s.last_result),
            };
            if action == "create"
                && self
                    .store
                    .records::<Schedule>("schedules")?
                    .iter()
                    .any(|s| {
                        s.status == "active"
                            && serde_json::to_value(&s.spec).ok()
                                == serde_json::to_value(&schedule.spec).ok()
                    })
            {
                bail!("An identical active schedule already exists");
            }
            self.store.put("schedules", &schedule.id, &schedule)?;
            if action == "create" {
                self.store.put(
                    "schedule_destinations",
                    &schedule.id,
                    &("web", None::<String>),
                )?;
            }
            return Ok(json!(schedule));
        }
        let mut schedule = self
            .store
            .get::<Schedule>("schedules", id.as_deref().context("Schedule ID required")?)?
            .context("Schedule not found")?;
        match action.as_str() {
            "pause" => {
                schedule.status = "paused".into();
                schedule.next_run = None;
            }
            "resume" => {
                schedule.next_run = schedule.spec.next(Utc::now())?;
                if schedule.next_run.is_none() {
                    bail!("This schedule has no future execution; edit it first");
                }
                schedule.status = "active".into();
            }
            "cancel" => {
                schedule.status = "cancelled".into();
                schedule.next_run = None;
            }
            _ => bail!("Unknown schedule action"),
        }
        self.store.put("schedules", &schedule.id, &schedule)?;
        Ok(json!(schedule))
    }

    pub(crate) async fn skip_missed_schedules(&self) -> Result<()> {
        let _lock = self.schedule_lock.lock().await;
        let now = Utc::now();
        for mut schedule in self.store.records::<Schedule>("schedules")? {
            if schedule.status == "active" && schedule.next_run.is_some_and(|t| t <= now) {
                schedule.next_run = schedule.spec.next(now)?;
                if schedule.next_run.is_none() {
                    schedule.status = "missed".into();
                }
                schedule.last_result = Some("Missed while offline; not replayed".into());
                self.store.put("schedules", &schedule.id, &schedule)?;
            }
        }
        Ok(())
    }

    pub async fn tick_schedules(&self) -> Result<()> {
        let _lock = self.schedule_lock.lock().await;
        let now = Utc::now();
        for mut schedule in self.store.records::<Schedule>("schedules")? {
            if schedule.status != "active" || !schedule.next_run.is_some_and(|t| t <= now) {
                continue;
            }
            schedule.next_run = if schedule.spec.kind == "once" {
                None
            } else {
                schedule.spec.next(now)?
            };
            if schedule.next_run.is_none() {
                schedule.status = "completed".into();
            }
            schedule.last_run = Some(now);
            // Advance durably before effects: no automatic replay after process failure.
            schedule.last_result = Some("Dispatching".into());
            self.store.put("schedules", &schedule.id, &schedule)?;
            let result = if schedule.spec.mode == "workflow" {
                match self
                    .enqueue_workflow(
                        schedule
                            .spec
                            .workflow_id
                            .as_deref()
                            .context("Workflow ID required")?,
                        false,
                        Some(&schedule.id),
                    )
                    .await
                {
                    Ok(run) => {
                        self.pin_schedule_destination(&run.task_id, &schedule.id)?;
                        Ok(format!("Queued workflow execution {}", run.id))
                    }
                    Err(error) => Err(error),
                }
            } else if schedule.spec.mode == "reminder" {
                let project = schedule
                    .spec
                    .project_id
                    .as_deref()
                    .map(|id| self.store.project(id))
                    .transpose()?;
                let label = project
                    .as_ref()
                    .map(|p| format!("Proyecto {} · ", p.name))
                    .unwrap_or_default();
                let notification = self.store.record_notification_task(
                    &schedule.spec.content,
                    &fenix_storage::TaskBinding {
                        project_id: schedule.spec.project_id.clone(),
                        read_project_ids: schedule.spec.project_id.iter().cloned().collect(),
                        mode: if project.is_some() {
                            "project"
                        } else {
                            "general"
                        }
                        .into(),
                        workspace: project.as_ref().and_then(|p| p.workspace.clone()),
                        context_version: project.as_ref().map_or(0, |p| p.context_version),
                        ..Default::default()
                    },
                )?;
                self.pin_schedule_destination(&notification.id, &schedule.id)?;
                self.notify(
                    Some(&notification.id),
                    &format!(
                        "{label}Recordatorio · {}\n\n{}",
                        schedule.spec.name, schedule.spec.content
                    ),
                )
                .map(|_| "Reminder delivered to conversation".into())
            } else if self.model.read().await.is_none() {
                Err(anyhow::anyhow!("Model is not configured"))
            } else {
                (|| -> Result<String> {
                    let project = schedule
                        .spec
                        .project_id
                        .as_deref()
                        .map(|id| self.store.project(id))
                        .transpose()?;
                    let binding = fenix_storage::TaskBinding {
                        project_id: schedule.spec.project_id.clone(),
                        read_project_ids: schedule.spec.project_id.iter().cloned().collect(),
                        mode: if project.is_some() {
                            "project"
                        } else {
                            "general"
                        }
                        .into(),
                        workspace: project.as_ref().and_then(|p| p.workspace.clone()).or_else(
                            || {
                                self.settings()
                                    .ok()
                                    .flatten()
                                    .map(|s| s.workspace.to_string_lossy().into_owned())
                            },
                        ),
                        context_version: project.as_ref().map_or(0, |p| p.context_version),
                        ..Default::default()
                    };
                    let task = self.store.enqueue_bound(
                        &schedule.spec.content,
                        "schedule",
                        Some(&schedule.id),
                        &binding,
                    )?;
                    self.pin_schedule_destination(&task.id, &schedule.id)?;
                    self.wake.notify_one();
                    Ok(format!("Queued task {}", task.id))
                })()
            };
            schedule.last_result = Some(match result {
                Ok(message) => message,
                Err(e) => format!("Skipped: {}", self.redactor.text(&e.to_string())),
            });
            self.store.put("schedules", &schedule.id, &schedule)?;
        }
        Ok(())
    }

    pub(crate) async fn scheduler(&self) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            tokio::select! {_=self.stop.cancelled()=>break,_=interval.tick()=>{
                if let Err(error)=self.tick_schedules().await {eprintln!("Schedule error: {}",self.redactor.text(&error.to_string()));}
            }}
        }
    }
}
