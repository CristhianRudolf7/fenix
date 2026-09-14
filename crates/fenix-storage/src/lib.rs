//! Durable conversation, task journal and lexical memory. No model or channel dependencies.

mod memory;
mod projects;
mod subagents;
pub use memory::Memory;
pub use projects::{Project, ProjectInput, TaskBinding};
pub use subagents::{Subagent, SubagentSpec};

use anyhow::{Context, Result, bail};
use fenix_core::{Message, Role};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::{
    path::Path,
    sync::{Mutex, MutexGuard},
    time::Duration,
};

pub fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}
pub fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub struct Store {
    connection: Mutex<Connection>,
    session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub session_id: String,
    pub request: String,
    pub origin: String,
    pub status: String,
    pub schedule_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub seq: i64,
    pub id: String,
    pub task_id: Option<String>,
    pub role: String,
    pub text: String,
    pub message: Message,
    pub origin: String,
    pub created_at: String,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        let mut connection = Connection::open(path)?;
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version > 3 {
            bail!("Database was created by a newer Fenix version");
        }
        let migration = connection.transaction()?;
        if version < 1 {
            migration.execute_batch(include_str!("../migrations/001_initial.sql"))?;
        }
        if version < 2 {
            migration.execute_batch(include_str!("../migrations/002_subagents.sql"))?;
        }
        if version < 3 {
            migration.execute_batch(include_str!("../migrations/003_projects.sql"))?;
        }
        migration.commit()?;
        connection.execute(
            "INSERT OR IGNORE INTO metadata VALUES ('session_id', ?1)",
            [id()],
        )?;
        let session_id = connection.query_row(
            "SELECT value FROM metadata WHERE key='session_id'",
            [],
            |r| r.get(0),
        )?;
        Ok(Self {
            connection: Mutex::new(connection),
            session_id,
        })
    }

    fn db(&self) -> Result<MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|_| anyhow::anyhow!("Database lock unavailable"))
    }
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Do not resume queued, running or waiting work from a previous process.
    pub fn interrupt_unfinished(&self) -> Result<usize> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        tx.execute("UPDATE subagents SET result=?1 WHERE id IN (SELECT id FROM tasks WHERE status IN ('queued','running','waiting'))", [json!({"status":"interrupted","error":"Service stopped before completion; actions were not resumed.","pending":"Inspect saved task events before repeating actions."}).to_string()])?;
        let count = tx.execute("UPDATE tasks SET status='interrupted', error='Service stopped before completion; actions were not resumed.', updated_at=?1 WHERE status IN ('queued','running','waiting')", [now()])?;
        // Conversation history remains recoverable, but incomplete tool sequences never enter model context.
        tx.execute("UPDATE messages SET in_context=0 WHERE task_id IN (SELECT id FROM tasks WHERE status='interrupted')", [])?;
        tx.commit()?;
        Ok(count)
    }

    pub fn enqueue(&self, text: &str, origin: &str, schedule_id: Option<&str>) -> Result<Task> {
        self.enqueue_referenced(text, origin, schedule_id, None)
    }

    pub fn enqueue_referenced(
        &self,
        text: &str,
        origin: &str,
        schedule_id: Option<&str>,
        reply_to_task_id: Option<&str>,
    ) -> Result<Task> {
        self.enqueue_with_binding(text, origin, schedule_id, reply_to_task_id, None)
    }

    /// Scheduled work becomes visible to workers only after its scope is pinned.
    pub fn enqueue_bound(
        &self,
        text: &str,
        origin: &str,
        schedule_id: Option<&str>,
        binding: &TaskBinding,
    ) -> Result<Task> {
        self.enqueue_with_binding(
            text,
            origin,
            schedule_id,
            binding.reply_to_task_id.as_deref(),
            Some(binding),
        )
    }

    fn enqueue_with_binding(
        &self,
        text: &str,
        origin: &str,
        schedule_id: Option<&str>,
        reply_to_task_id: Option<&str>,
        binding: Option<&TaskBinding>,
    ) -> Result<Task> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        if let Some(reference) = reply_to_task_id {
            let exists: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM tasks WHERE id=?1)",
                [reference],
                |row| row.get(0),
            )?;
            if !exists {
                bail!("Referenced task not found");
            }
        }
        let focus: Option<String> = tx
            .query_row(
                "SELECT value FROM metadata WHERE key='conversation_focus'",
                [],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(schedule) = schedule_id {
            let busy: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM tasks WHERE schedule_id=?1 AND status IN ('queued','running','waiting'))", [schedule], |r| r.get(0))?;
            if busy {
                bail!("This schedule already has a pending execution");
            }
        }
        let task = Task {
            id: id(),
            session_id: self.session_id.clone(),
            request: text.to_owned(),
            origin: origin.to_owned(),
            status: "queued".into(),
            schedule_id: schedule_id.map(str::to_owned),
            created_at: now(),
            updated_at: now(),
            error: None,
        };
        tx.execute("INSERT INTO tasks(id,session_id,request,origin,status,schedule_id,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)", params![task.id,task.session_id,task.request,task.origin,task.status,task.schedule_id,task.created_at,task.updated_at])?;
        projects::write_binding(
            &tx,
            &task.id,
            &TaskBinding {
                reply_to_task_id: reply_to_task_id.map(str::to_owned),
                focus_project_id: focus,
                ..TaskBinding::default()
            },
        )?;
        if let Some(binding) = binding {
            projects::bind_task_in_database(&tx, &task.id, binding)?;
        }
        let message_id = id();
        tx.execute("INSERT INTO messages(id,task_id,role,text,body,visible,in_context,origin,created_at) VALUES (?1,?2,'user',?3,?4,1,0,?5,?6)", params![message_id,task.id,text,serde_json::to_string(&Message::user(text))?,origin,now()])?;
        tx.execute(
            "INSERT INTO history_fts(id,text) VALUES (?1,?2)",
            params![message_id, text],
        )?;
        tx.commit()?;
        Ok(task)
    }

    pub fn claim_next(&self) -> Result<Option<Task>> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let task = tx.query_row("SELECT id,session_id,request,origin,status,schedule_id,created_at,updated_at,error FROM tasks WHERE status='queued' AND origin!='subagent' ORDER BY CASE WHEN origin='subagent_report' THEN 1 ELSE 0 END,seq LIMIT 1", [], task_row).optional()?;
        if let Some(mut task) = task {
            task.status = "running".into();
            task.updated_at = now();
            tx.execute(
                "UPDATE tasks SET status='running',updated_at=?2 WHERE id=?1",
                params![task.id, task.updated_at],
            )?;
            tx.execute("UPDATE messages SET in_context=1,context_order=(SELECT COALESCE(MAX(context_order),0)+1 FROM messages) WHERE task_id=?1 AND role='user'", [&task.id])?;
            tx.commit()?;
            Ok(Some(task))
        } else {
            Ok(None)
        }
    }

    pub fn set_task_status(&self, task: &str, status: &str, error: Option<&str>) -> Result<()> {
        self.db()?.execute(
            "UPDATE tasks SET status=?2,error=?3,updated_at=?4 WHERE id=?1 AND (origin!='subagent' OR status NOT IN ('completed','blocked','failed','cancelled','interrupted'))",
            params![task, status, error, now()],
        )?;
        Ok(())
    }

    pub fn discard_context(&self, task: &str) -> Result<()> {
        self.db()?
            .execute("UPDATE messages SET in_context=0 WHERE task_id=?1", [task])?;
        Ok(())
    }

    pub fn tasks(&self, limit: usize) -> Result<Vec<Task>> {
        Ok(self.db()?.prepare("SELECT id,session_id,request,origin,status,schedule_id,created_at,updated_at,error FROM tasks ORDER BY seq DESC LIMIT ?1")?.query_map([limit.min(200)], task_row)?.collect::<rusqlite::Result<_>>()?)
    }

    pub fn task(&self, id: &str) -> Result<Task> {
        self.db()?.query_row("SELECT id,session_id,request,origin,status,schedule_id,created_at,updated_at,error FROM tasks WHERE id=?1", [id], task_row).optional()?.context("Task not found")
    }

    pub fn append(
        &self,
        task: Option<&str>,
        message: &Message,
        visible: bool,
        context: bool,
        origin: &str,
    ) -> Result<String> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let id = id();
        let text = message.text_content();
        let role = match message.role {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Tool => "tool",
        };
        tx.execute("INSERT INTO messages(id,task_id,role,text,body,visible,in_context,origin,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)", params![id,task,role,text,serde_json::to_string(message)?,visible,context,origin,now()])?;
        if context {
            tx.execute("UPDATE messages SET context_order=(SELECT COALESCE(MAX(context_order),0)+1 FROM messages) WHERE id=?1",[&id])?;
        }
        tx.execute(
            "INSERT INTO history_fts(id,text) VALUES (?1,?2)",
            params![id, text],
        )?;
        tx.commit()?;
        Ok(id)
    }

    pub fn conversation(&self, before: Option<i64>, limit: usize) -> Result<Vec<StoredMessage>> {
        let db = self.db()?;
        let mut rows = db.prepare("SELECT seq,id,task_id,role,text,body,origin,created_at FROM messages WHERE visible=1 AND seq<?1 ORDER BY seq DESC LIMIT ?2")?.query_map(params![before.unwrap_or(i64::MAX),limit.min(200)], message_row)?.collect::<rusqlite::Result<Vec<_>>>()?;
        rows.reverse();
        Ok(rows)
    }

    pub fn context(&self, after: i64) -> Result<Vec<StoredMessage>> {
        Ok(self.db()?.prepare("SELECT context_order,id,task_id,role,text,body,origin,created_at FROM messages WHERE in_context=1 AND context_order>?1 ORDER BY context_order")?.query_map([after], message_row)?.collect::<rusqlite::Result<_>>()?)
    }

    pub fn history_search(&self, query: &str, limit: usize) -> Result<Vec<Value>> {
        let Some(query) = lexical_query(query) else {
            return Ok(vec![]);
        };
        Ok(self.db()?.prepare("SELECT m.id,m.task_id,m.text,m.created_at FROM history_fts f JOIN messages m ON m.id=f.id WHERE history_fts MATCH ?1 ORDER BY rank LIMIT ?2")?.query_map(params![query,limit.min(30)], |r| Ok(json!({"id":r.get::<_,String>(0)?,"task_id":r.get::<_,Option<String>>(1)?,"text":r.get::<_,String>(2)?,"created_at":r.get::<_,String>(3)?})))?.collect::<rusqlite::Result<_>>()?)
    }

    pub fn history_read(&self, message_id: &str, radius: usize) -> Result<Vec<StoredMessage>> {
        let db = self.db()?;
        let seq: i64 = db
            .query_row("SELECT seq FROM messages WHERE id=?1", [message_id], |r| {
                r.get(0)
            })
            .optional()?
            .context("Message not found")?;
        Ok(db.prepare("SELECT seq,id,task_id,role,text,body,origin,created_at FROM messages WHERE seq BETWEEN ?1 AND ?2 ORDER BY seq")?.query_map(params![seq-radius.min(15) as i64,seq+radius.min(15) as i64],message_row)?.collect::<rusqlite::Result<_>>()?)
    }

    pub fn event(&self, task: &str, kind: &str, data: &Value) -> Result<()> {
        self.db()?.execute(
            "INSERT INTO events(id,task_id,kind,data,created_at) VALUES (?1,?2,?3,?4,?5)",
            params![id(), task, kind, data.to_string(), now()],
        )?;
        Ok(())
    }
    pub fn events(&self, task: &str, after: i64, limit: usize) -> Result<Vec<Value>> {
        Ok(self.db()?.prepare("SELECT seq,id,kind,data,created_at FROM events WHERE task_id=?1 AND seq>?2 ORDER BY seq LIMIT ?3")?.query_map(params![task,after,limit.min(100)],|r|Ok(json!({"seq":r.get::<_,i64>(0)?,"id":r.get::<_,String>(1)?,"kind":r.get::<_,String>(2)?,"data":parse_json(r.get(3)?)?,"created_at":r.get::<_,String>(4)?})))?.collect::<rusqlite::Result<_>>()?)
    }

    pub fn tool_output(
        &self,
        task: &str,
        call_id: &str,
        offset: usize,
        length: usize,
    ) -> Result<Value> {
        let text:String=self.db()?.query_row("SELECT json_extract(data,'$.result') FROM events WHERE task_id=?1 AND kind='tool_result' AND json_extract(data,'$.call_id')=?2 ORDER BY seq DESC LIMIT 1",params![task,call_id],|r|r.get(0)).optional()?.context("Tool output not found")?;
        let page = text
            .chars()
            .skip(offset)
            .take(length.clamp(1, 12_000))
            .collect::<String>();
        let next = offset + page.chars().count();
        Ok(
            json!({"call_id":call_id,"offset":offset,"next_offset":next,"total_characters":text.chars().count(),"content":page}),
        )
    }

    pub fn get<T: DeserializeOwned>(&self, kind: &str, id: &str) -> Result<Option<T>> {
        let data: Option<String> = self
            .db()?
            .query_row(
                "SELECT data FROM records WHERE kind=?1 AND id=?2",
                params![kind, id],
                |r| r.get(0),
            )
            .optional()?;
        data.map(|s| serde_json::from_str(&s).map_err(Into::into))
            .transpose()
    }
    pub fn put<T: Serialize>(&self, kind: &str, id: &str, data: &T) -> Result<()> {
        self.db()?.execute("INSERT INTO records(kind,id,data,updated_at) VALUES (?1,?2,?3,?4) ON CONFLICT(kind,id) DO UPDATE SET data=excluded.data,updated_at=excluded.updated_at",params![kind,id,serde_json::to_string(data)?,now()])?;
        Ok(())
    }
    pub fn records<T: DeserializeOwned>(&self, kind: &str) -> Result<Vec<T>> {
        self.db()?
            .prepare("SELECT data FROM records WHERE kind=?1 ORDER BY updated_at DESC")?
            .query_map([kind], |r| r.get::<_, String>(0))?
            .map(|r| Ok(serde_json::from_str(&r?)?))
            .collect()
    }
    pub fn delete(&self, kind: &str, id: &str) -> Result<()> {
        self.db()?.execute(
            "DELETE FROM records WHERE kind=?1 AND id=?2",
            params![kind, id],
        )?;
        Ok(())
    }

    pub fn queue_delivery(&self, text: &str) -> Result<()> {
        self.queue_delivery_for_task(text, None)
    }
    pub fn queue_delivery_for_task(&self, text: &str, task_id: Option<&str>) -> Result<()> {
        self.db()?.execute(
            "INSERT INTO outbox(text,created_at,task_id) VALUES (?1,?2,?3)",
            params![text, now(), task_id],
        )?;
        Ok(())
    }
    pub fn delivery_task(&self, seq: i64) -> Result<Option<String>> {
        Ok(self
            .db()?
            .query_row("SELECT task_id FROM outbox WHERE seq=?1", [seq], |row| {
                row.get(0)
            })
            .optional()?
            .flatten())
    }
    pub fn deliveries(&self) -> Result<Vec<(i64, String)>> {
        Ok(self.db()?.prepare("SELECT seq,text FROM outbox WHERE status='pending' AND attempts<3 ORDER BY seq LIMIT 10")?.query_map([],|r|Ok((r.get(0)?,r.get(1)?)))?.collect::<rusqlite::Result<_>>()?)
    }
    pub fn delivery_result(&self, seq: i64, delivered: bool) -> Result<()> {
        self.db()?.execute("UPDATE outbox SET status=CASE WHEN ?2 THEN 'sent' WHEN attempts>=2 THEN 'failed' ELSE 'pending' END,attempts=attempts+1 WHERE seq=?1",params![seq,delivered])?;
        Ok(())
    }
}

fn task_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    Ok(Task {
        id: r.get(0)?,
        session_id: r.get(1)?,
        request: r.get(2)?,
        origin: r.get(3)?,
        status: r.get(4)?,
        schedule_id: r.get(5)?,
        created_at: r.get(6)?,
        updated_at: r.get(7)?,
        error: r.get(8)?,
    })
}
fn message_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<StoredMessage> {
    Ok(StoredMessage {
        seq: r.get(0)?,
        id: r.get(1)?,
        task_id: r.get(2)?,
        role: r.get(3)?,
        text: r.get(4)?,
        message: serde_json::from_value(parse_json(r.get(5)?)?).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e))
        })?,
        origin: r.get(6)?,
        created_at: r.get(7)?,
    })
}
fn parse_json(text: String) -> rusqlite::Result<Value> {
    serde_json::from_str(&text).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}

/// Escape all user syntax; Unicode tokens, OR retrieval and FTS5 ranking only.
pub fn lexical_query(query: &str) -> Option<String> {
    let words = query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .take(20)
        .map(|s| format!("\"{s}\""))
        .collect::<Vec<_>>();
    (!words.is_empty()).then(|| words.join(" OR "))
}
