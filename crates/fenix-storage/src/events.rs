//! Persisted publication with cursor-based subscribers. Wakeups are only hints.
use crate::{Store, id, now, parse_json};
use anyhow::{Context, Result, bail};
use fenix_core::{Message, Role};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusEvent {
    pub seq: i64,
    pub id: String,
    pub topic: String,
    pub task_id: Option<String>,
    pub data: Value,
    pub created_at: String,
}

pub(crate) fn publish(
    db: &Connection,
    task: Option<&str>,
    topic: &str,
    data: &Value,
) -> Result<String> {
    let event_id = id();
    let mut data = data.clone();
    if let Some(object) = data.as_object_mut() {
        object.entry("correlation_id").or_insert(json!(task));
        if let Some(task) = task {
            let session: Option<String> = db
                .query_row("SELECT session_id FROM tasks WHERE id=?1", [task], |r| {
                    r.get(0)
                })
                .optional()?;
            let project: Option<String> = db
                .query_row(
                    "SELECT project_id FROM task_bindings WHERE task_id=?1",
                    [task],
                    |r| r.get(0),
                )
                .optional()?
                .flatten();
            let pin:Option<(String,u64)>=db.query_row("SELECT id,version FROM plans WHERE task_id=?1 UNION SELECT p.id,p.version FROM plans p JOIN plan_workers w ON p.id=w.plan_id WHERE w.task_id=?1 LIMIT 1",[task],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
            let destination: Option<(String, Option<String>)> = db
                .query_row(
                    "SELECT channel,address FROM task_destinations WHERE task_id=?1",
                    [task],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?;
            object.entry("session_id").or_insert(json!(session));
            object.entry("project_id").or_insert(json!(project));
            object.entry("destination").or_insert(json!(destination));
            if let Some((plan, version)) = pin {
                object.entry("plan_id").or_insert(json!(plan));
                object.entry("plan_version").or_insert(json!(version));
            }
        }
    }

    db.execute(
        "INSERT INTO bus_events(id,topic,task_id,data,created_at) VALUES (?1,?2,?3,?4,?5)",
        params![event_id, topic, task, data.to_string(), now()],
    )?;
    Ok(event_id)
}

impl Store {
    pub fn publish_event(&self, task: Option<&str>, topic: &str, data: &Value) -> Result<String> {
        publish(&*self.db()?, task, topic, data)
    }

    pub fn bus_events(&self, after: i64, limit: usize) -> Result<Vec<BusEvent>> {
        Ok(self.db()?.prepare("SELECT seq,id,topic,task_id,data,created_at FROM bus_events WHERE seq>?1 ORDER BY seq LIMIT ?2")?
            .query_map(params![after,limit.clamp(1,200)], |r| Ok(BusEvent {
                seq:r.get(0)?,id:r.get(1)?,topic:r.get(2)?,task_id:r.get(3)?,data:parse_json(r.get(4)?)?,created_at:r.get(5)?,
            }))?.collect::<rusqlite::Result<_>>()?)
    }

    pub fn subscriber_cursor(&self, name: &str) -> Result<i64> {
        Ok(self
            .db()?
            .query_row(
                "SELECT cursor FROM event_subscribers WHERE name=?1",
                [name],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or(0))
    }

    pub fn acknowledge_event(&self, name: &str, seq: i64) -> Result<()> {
        self.db()?.execute("INSERT INTO event_subscribers(name,cursor) VALUES (?1,?2) ON CONFLICT(name) DO UPDATE SET cursor=MAX(cursor,excluded.cursor)",params![name,seq])?;
        Ok(())
    }

    pub fn set_task_destination(
        &self,
        task: &str,
        channel: &str,
        address: Option<&str>,
    ) -> Result<()> {
        if !matches!(channel, "web" | "telegram") {
            bail!("Unknown delivery channel");
        }
        self.db()?.execute(
            "INSERT OR IGNORE INTO task_destinations(task_id,channel,address) VALUES (?1,?2,?3)",
            params![task, channel, address],
        )?;
        Ok(())
    }

    pub fn task_destination(&self, task: &str) -> Result<Option<(String, Option<String>)>> {
        Ok(self
            .db()?
            .query_row(
                "SELECT channel,address FROM task_destinations WHERE task_id=?1",
                [task],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?)
    }

    pub fn inherit_destination(&self, task: &str, parent: &str) -> Result<()> {
        self.db()?.execute("INSERT OR IGNORE INTO task_destinations(task_id,channel,address) SELECT ?1,channel,address FROM task_destinations WHERE task_id=?2",params![task,parent])?;
        Ok(())
    }

    /// Atomically append visible history and its delivery event. Each channel
    /// subscriber may reconnect without rerunning the model or any tool.
    pub fn publish_message(
        &self,
        task: Option<&str>,
        message: &Message,
        in_context: bool,
        origin: &str,
    ) -> Result<String> {
        self.publish_message_once(&id(), task, message, in_context, origin)
    }

    pub fn publish_message_once(
        &self,
        message_id: &str,
        task: Option<&str>,
        message: &Message,
        in_context: bool,
        origin: &str,
    ) -> Result<String> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        if tx
            .query_row("SELECT 1 FROM messages WHERE id=?1", [message_id], |_| {
                Ok(())
            })
            .optional()?
            .is_some()
        {
            return Ok(message_id.to_owned());
        }
        let text = message.text_content();
        let role = match message.role {
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::System => "system",
            Role::Tool => "tool",
        };
        tx.execute("INSERT INTO messages(id,task_id,role,text,body,visible,in_context,origin,created_at) VALUES (?1,?2,?3,?4,?5,?9,?6,?7,?8)",params![message_id,task,role,text,serde_json::to_string(message)?,in_context,origin,now(),!text.is_empty()])?;
        if in_context {
            tx.execute("UPDATE messages SET context_order=(SELECT COALESCE(MAX(context_order),0)+1 FROM messages) WHERE id=?1",[&message_id])?;
        }
        tx.execute(
            "INSERT INTO history_fts(id,text) VALUES (?1,?2)",
            params![message_id, text],
        )?;
        let destination: Option<(String, Option<String>)> = task
            .map(|task| {
                tx.query_row(
                    "SELECT channel,address FROM task_destinations WHERE task_id=?1",
                    [task],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()
            })
            .transpose()?
            .flatten();
        if !text.is_empty() {
            publish(
                &tx,
                task,
                "message.created",
                &json!({"message_id":message_id,"text":text,"role":role,"destination":destination,"session_id":self.session_id}),
            )?;
        }
        tx.commit()?;
        Ok(message_id.to_owned())
    }

    /// A subscriber can replay this operation safely after a crash before ACK.
    pub fn queue_event_delivery(
        &self,
        event: &BusEvent,
        destination: &str,
        text: &str,
    ) -> Result<()> {
        self.db()?.execute("INSERT OR IGNORE INTO outbox(text,created_at,task_id,destination,event_id) VALUES (?1,?2,?3,?4,?5)",params![text,now(),event.task_id,destination,event.id])?;
        Ok(())
    }

    pub fn deliveries_for(&self, destination: &str) -> Result<Vec<(i64, String)>> {
        Ok(self.db()?.prepare("SELECT seq,text FROM outbox WHERE status='pending' AND (destination=?1 OR destination IS NULL) AND (retry_at IS NULL OR retry_at<=?2) ORDER BY seq LIMIT 10")?.query_map(params![destination,now()],|r|Ok((r.get(0)?,r.get(1)?)))?.collect::<rusqlite::Result<_>>()?)
    }

    pub fn save_question(&self, question: &Value) -> Result<()> {
        let id = question["id"].as_str().context("Question ID required")?;
        let task = question["task_id"]
            .as_str()
            .context("Question task required")?;
        let mut db = self.db()?;
        let tx = db.transaction()?;
        tx.execute("INSERT INTO questions(id,task_id,kind,status,body,created_at,updated_at) VALUES (?1,?2,?3,'pending',?4,?5,?5)",params![id,task,question["kind"].as_str().unwrap_or("clarification"),question.to_string(),now()])?;
        publish(&tx, Some(task), "question.created", question)?;
        tx.commit()?;
        Ok(())
    }

    pub fn resolve_question(&self, id: &str, status: &str, answer: Option<&str>) -> Result<bool> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let changed=tx.execute("UPDATE questions SET status=?2,answer=?3,updated_at=?4 WHERE id=?1 AND status='pending'",params![id,status,answer,now()])?;
        if changed > 0 {
            let task: String =
                tx.query_row("SELECT task_id FROM questions WHERE id=?1", [id], |r| {
                    r.get(0)
                })?;
            publish(
                &tx,
                Some(&task),
                "question.resolved",
                &json!({"request_id":id,"status":status}),
            )?;
        }
        tx.commit()?;
        Ok(changed > 0)
    }

    pub fn saved_questions(&self, task: Option<&str>) -> Result<Vec<Value>> {
        Ok(self.db()?.prepare("SELECT body,status,answer FROM questions WHERE (?1 IS NULL OR task_id=?1) ORDER BY created_at DESC LIMIT 200")?.query_map([task],|r|{
            let mut value=parse_json(r.get(0)?)?;
            value["status"]=json!(r.get::<_,String>(1)?);
            value["answer"]=json!(r.get::<_,Option<String>>(2)?);
            Ok(value)
        })?.collect::<rusqlite::Result<_>>()?)
    }
}
