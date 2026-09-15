use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub workspace: Option<String>,
    pub brief: String,
    pub context_version: u64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectInput {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub brief: String,
}

/// Scope is captured once per task. Foreground focus is only a routing hint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TaskBinding {
    pub project_id: Option<String>,
    pub read_project_ids: Vec<String>,
    pub mode: String,
    pub reply_to_task_id: Option<String>,
    pub focus_project_id: Option<String>,
    pub workspace: Option<String>,
    pub context_version: u64,
    pub bound: bool,
}

impl Default for TaskBinding {
    fn default() -> Self {
        Self {
            project_id: None,
            read_project_ids: vec![],
            mode: "general".into(),
            reply_to_task_id: None,
            focus_project_id: None,
            workspace: None,
            context_version: 0,
            bound: false,
        }
    }
}

const PROJECT_COLUMNS: &str =
    "id,name,aliases,description,workspace,brief,context_version,created_at,updated_at";
// Legacy messages without a binding remain in the general scope. Unrouted,
// clarification and coordination turns never enter a project's history.
const MESSAGE_SCOPE: &str = "b.project_id IS ?1 AND COALESCE(b.mode,'general') IN ('general','project') AND (b.task_id IS NULL OR b.bound=1)";

impl Store {
    pub fn projects(&self) -> Result<Vec<Project>> {
        Ok(self
            .db()?
            .prepare(&format!(
                "SELECT {PROJECT_COLUMNS} FROM projects ORDER BY name COLLATE NOCASE,id"
            ))?
            .query_map([], project_row)?
            .collect::<rusqlite::Result<_>>()?)
    }

    pub fn project(&self, id: &str) -> Result<Project> {
        query_project(&*self.db()?, id)
    }

    pub fn save_project(&self, project_id: Option<&str>, input: &ProjectInput) -> Result<Project> {
        let name = input.name.trim();
        if name.is_empty()
            || name.len() > 200
            || input.description.len() > 8000
            || input.brief.len() > 32_000
        {
            bail!(
                "Project needs a name (1–200 bytes), description up to 8000 bytes and brief up to 32000 bytes"
            );
        }
        if input.aliases.len() > 30
            || input
                .aliases
                .iter()
                .any(|alias| alias.trim().is_empty() || alias.len() > 200)
        {
            bail!("Use at most 30 project aliases of 1–200 bytes");
        }
        let workspace = input
            .workspace
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned);
        if workspace
            .as_deref()
            .is_some_and(|path| !Path::new(path).is_absolute() || path.len() > 4096)
        {
            bail!("Project workspace must be an absolute path of at most 4096 bytes");
        }
        let mut aliases: Vec<String> = input
            .aliases
            .iter()
            .map(|alias| alias.trim().to_owned())
            .collect();
        aliases.sort_by_key(|alias| alias.to_lowercase());
        aliases.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let previous = project_id.map(|id| query_project(&tx, id)).transpose()?;
        let project = Project {
            id: project_id.map(str::to_owned).unwrap_or_else(id),
            name: name.into(),
            aliases,
            description: input.description.trim().into(),
            workspace,
            brief: input.brief.trim().into(),
            context_version: previous.as_ref().map_or(1, |p| p.context_version + 1),
            created_at: previous
                .as_ref()
                .map(|p| p.created_at.clone())
                .unwrap_or_else(now),
            updated_at: now(),
        };
        tx.execute(
            "DELETE FROM project_aliases WHERE project_id=?1",
            [&project.id],
        )?;
        tx.execute("INSERT INTO projects(id,name,aliases,description,workspace,brief,context_version,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9) ON CONFLICT(id) DO UPDATE SET name=excluded.name,aliases=excluded.aliases,description=excluded.description,workspace=excluded.workspace,brief=excluded.brief,context_version=excluded.context_version,updated_at=excluded.updated_at", params![project.id,project.name,serde_json::to_string(&project.aliases)?,project.description,project.workspace,project.brief,project.context_version,project.created_at,project.updated_at])?;
        for alias in std::iter::once(&project.name).chain(project.aliases.iter()) {
            let normalized = alias.to_lowercase();
            let owner: Option<String> = tx
                .query_row(
                    "SELECT project_id FROM project_aliases WHERE alias=?1",
                    [&normalized],
                    |row| row.get(0),
                )
                .optional()?;
            if owner.as_deref().is_some_and(|owner| owner != project.id) {
                bail!("A project already uses the name or alias '{alias}'");
            }
            tx.execute(
                "INSERT OR IGNORE INTO project_aliases(alias,project_id) VALUES (?1,?2)",
                params![normalized, project.id],
            )?;
        }
        tx.commit()?;
        Ok(project)
    }

    /// Compare-and-swap prevents late work from overwriting newer project notes.
    pub fn update_project_brief(
        &self,
        id: &str,
        brief: &str,
        expected_version: u64,
    ) -> Result<Project> {
        if brief.len() > 32_000 {
            bail!("Project brief exceeds 32000 bytes");
        }
        let mut db = self.db()?;
        let tx = db.transaction()?;
        if tx.execute("UPDATE projects SET brief=?2,context_version=context_version+1,updated_at=?3 WHERE id=?1 AND context_version=?4", params![id,brief.trim(),now(),expected_version])? != 1 {
            bail!("Project context changed or project was not found; reload it before updating");
        }
        let project = query_project(&tx, id)?;
        tx.commit()?;
        Ok(project)
    }

    pub fn conversation_focus(&self) -> Result<Option<String>> {
        Ok(self
            .db()?
            .query_row(
                "SELECT value FROM metadata WHERE key='conversation_focus'",
                [],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub fn set_conversation_focus(&self, project_id: Option<&str>) -> Result<()> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        if let Some(project_id) = project_id {
            query_project(&tx, project_id)?;
            tx.execute("INSERT INTO metadata(key,value) VALUES ('conversation_focus',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [project_id])?;
        } else {
            tx.execute("DELETE FROM metadata WHERE key='conversation_focus'", [])?;
        }
        tx.commit()?;
        Ok(())
    }

    /// A slow routing response cannot replace the focus selected by a newer turn.
    pub fn set_task_focus(&self, task: &str, project_id: Option<&str>) -> Result<()> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let seq: i64 = tx.query_row("SELECT seq FROM tasks WHERE id=?1", [task], |r| r.get(0))?;
        let last:i64=tx.query_row("SELECT CAST(value AS INTEGER) FROM metadata WHERE key='conversation_focus_task_seq'",[],|r|r.get(0)).optional()?.unwrap_or(0);
        if seq < last {
            return Ok(());
        }
        if let Some(project) = project_id {
            query_project(&tx, project)?;
            tx.execute("INSERT INTO metadata(key,value) VALUES ('conversation_focus',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[project])?;
        } else {
            tx.execute("DELETE FROM metadata WHERE key='conversation_focus'", [])?;
        }
        tx.execute("INSERT INTO metadata(key,value) VALUES ('conversation_focus_task_seq',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[seq.to_string()])?;
        tx.commit()?;
        Ok(())
    }

    pub fn task_binding(&self, task_id: &str) -> Result<TaskBinding> {
        read_binding(&*self.db()?, task_id)
    }

    /// Give a scheduled notice durable scope and reply provenance without ever
    /// putting it in the worker queue or creating an artificial user message.
    pub fn record_notification_task(&self, text: &str, binding: &TaskBinding) -> Result<Task> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        let timestamp = now();
        let task = Task {
            id: id(),
            session_id: self.session_id.clone(),
            request: text.to_owned(),
            origin: "reminder".into(),
            status: "completed".into(),
            schedule_id: None,
            created_at: timestamp.clone(),
            updated_at: timestamp,
            error: None,
        };
        tx.execute("INSERT INTO tasks(id,session_id,request,origin,status,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",params![task.id,task.session_id,task.request,task.origin,task.status,task.created_at,task.updated_at])?;
        bind_task_in_database(&tx, &task.id, binding)?;
        tx.commit()?;
        Ok(task)
    }

    pub fn bind_task(&self, task_id: &str, binding: &TaskBinding) -> Result<()> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        bind_task_in_database(&tx, task_id, binding)?;
        tx.commit()?;
        Ok(())
    }

    pub fn tasks_for_project(&self, id: &str, limit: usize) -> Result<Vec<Task>> {
        let db = self.db()?;
        query_project(&db, id)?;
        Ok(db.prepare("SELECT t.id,t.session_id,t.request,t.origin,t.status,t.schedule_id,t.created_at,t.updated_at,t.error FROM tasks t JOIN task_bindings b ON b.task_id=t.id WHERE b.project_id=?1 AND b.bound=1 ORDER BY t.seq DESC LIMIT ?2")?
            .query_map(params![id,limit.min(200)],task_row)?.collect::<rusqlite::Result<_>>()?)
    }

    pub fn context_scoped(
        &self,
        after: i64,
        project_id: Option<&str>,
    ) -> Result<Vec<StoredMessage>> {
        Ok(self.db()?.prepare(&format!("SELECT m.context_order,m.id,m.task_id,m.role,m.text,m.body,m.origin,m.created_at FROM messages m LEFT JOIN task_bindings b ON b.task_id=m.task_id WHERE {MESSAGE_SCOPE} AND m.in_context=1 AND m.context_order>?2 ORDER BY m.context_order"))?
            .query_map(params![project_id,after],message_row)?.collect::<rusqlite::Result<_>>()?)
    }

    pub fn recent_context_scoped(
        &self,
        project_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<StoredMessage>> {
        let db = self.db()?;
        let mut messages = db.prepare(&format!("SELECT m.context_order,m.id,m.task_id,m.role,m.text,m.body,m.origin,m.created_at FROM messages m LEFT JOIN task_bindings b ON b.task_id=m.task_id WHERE {MESSAGE_SCOPE} AND m.in_context=1 AND m.role IN ('user','assistant') AND trim(m.text)!='' ORDER BY m.context_order DESC LIMIT ?2"))?
            .query_map(params![project_id,limit.min(200)],message_row)?.collect::<rusqlite::Result<Vec<_>>>()?;
        messages.reverse();
        Ok(messages)
    }

    pub fn task_user_message(&self, task_id: &str) -> Result<Option<StoredMessage>> {
        Ok(self.db()?.query_row("SELECT seq,id,task_id,role,text,body,origin,created_at FROM messages WHERE task_id=?1 AND role='user' ORDER BY seq LIMIT 1",[task_id],message_row).optional()?)
    }

    pub fn history_search_scoped(
        &self,
        query: &str,
        limit: usize,
        project_id: Option<&str>,
    ) -> Result<Vec<Value>> {
        let Some(query) = lexical_query(query) else {
            return Ok(vec![]);
        };
        Ok(self.db()?.prepare(&format!("SELECT m.id,m.task_id,m.text,m.created_at FROM history_fts f JOIN messages m ON m.id=f.id LEFT JOIN task_bindings b ON b.task_id=m.task_id WHERE {MESSAGE_SCOPE} AND history_fts MATCH ?2 ORDER BY rank LIMIT ?3"))?
            .query_map(params![project_id,query,limit.min(30)], |r| Ok(json!({"id":r.get::<_,String>(0)?,"task_id":r.get::<_,Option<String>>(1)?,"text":r.get::<_,String>(2)?,"created_at":r.get::<_,String>(3)?,"project_id":project_id})))?.collect::<rusqlite::Result<_>>()?)
    }

    pub fn history_read_scoped(
        &self,
        message_id: &str,
        radius: usize,
        project_id: Option<&str>,
    ) -> Result<Vec<StoredMessage>> {
        let db = self.db()?;
        let seq: i64 = db.query_row(&format!("SELECT m.seq FROM messages m LEFT JOIN task_bindings b ON b.task_id=m.task_id WHERE {MESSAGE_SCOPE} AND m.id=?2"), params![project_id,message_id], |row| row.get(0)).optional()?.context("Message not found in this scope")?;
        Ok(db.prepare(&format!("SELECT m.seq,m.id,m.task_id,m.role,m.text,m.body,m.origin,m.created_at FROM messages m LEFT JOIN task_bindings b ON b.task_id=m.task_id WHERE {MESSAGE_SCOPE} AND m.seq BETWEEN ?2 AND ?3 ORDER BY m.seq"))?
            .query_map(params![project_id,seq-radius.min(15) as i64,seq+radius.min(15) as i64],message_row)?.collect::<rusqlite::Result<_>>()?)
    }
}

pub(crate) fn read_binding(db: &Connection, task_id: &str) -> Result<TaskBinding> {
    let exists: bool = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM tasks WHERE id=?1)",
        [task_id],
        |row| row.get(0),
    )?;
    if !exists {
        bail!("Task not found");
    }
    let data: Option<String> = db
        .query_row(
            "SELECT data FROM task_bindings WHERE task_id=?1",
            [task_id],
            |row| row.get(0),
        )
        .optional()?;
    data.map(|data| serde_json::from_str(&data).map_err(Into::into))
        .transpose()
        .map(|binding| binding.unwrap_or_default())
}

pub(crate) fn write_binding(db: &Connection, task_id: &str, binding: &TaskBinding) -> Result<()> {
    db.execute("INSERT INTO task_bindings(task_id,project_id,mode,bound,data) VALUES (?1,?2,?3,?4,?5) ON CONFLICT(task_id) DO UPDATE SET project_id=excluded.project_id,mode=excluded.mode,bound=excluded.bound,data=excluded.data",params![task_id,binding.project_id,binding.mode,binding.bound,serde_json::to_string(binding)?])?;
    Ok(())
}

pub(crate) fn query_project(db: &Connection, id: &str) -> Result<Project> {
    db.query_row(
        &format!("SELECT {PROJECT_COLUMNS} FROM projects WHERE id=?1"),
        [id],
        project_row,
    )
    .optional()?
    .context("Project not found")
}

fn project_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    let aliases = serde_json::from_str(&row.get::<_, String>(2)?).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(error))
    })?;
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        aliases,
        description: row.get(3)?,
        workspace: row.get(4)?,
        brief: row.get(5)?,
        context_version: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

pub(crate) fn bind_task_in_database(
    db: &Connection,
    task_id: &str,
    binding: &TaskBinding,
) -> Result<()> {
    let previous = read_binding(db, task_id)?;
    let mut binding = binding.clone();
    binding.bound = true;
    binding.reply_to_task_id = previous.reply_to_task_id.clone();
    binding.focus_project_id = previous.focus_project_id.clone();
    if !matches!(
        binding.mode.as_str(),
        "general" | "project" | "coordination" | "clarification"
    ) {
        bail!("Unknown task context mode");
    }
    if (binding.mode == "project") != binding.project_id.is_some() {
        bail!("Only a project task may have a primary project");
    }
    if let Some(project_id) = &binding.project_id {
        let project = query_project(db, project_id)?;
        if !binding.read_project_ids.contains(project_id) {
            binding.read_project_ids.push(project_id.clone());
        }
        if binding.workspace.is_none() {
            binding.workspace = if previous.bound {
                previous.workspace.clone()
            } else {
                project.workspace
            };
        }
        if binding.context_version == 0 {
            binding.context_version = if previous.bound {
                previous.context_version
            } else {
                project.context_version
            };
        }
    }
    binding.read_project_ids.sort();
    binding.read_project_ids.dedup();
    for project_id in &binding.read_project_ids {
        query_project(db, project_id)?;
    }
    if binding.mode == "general" && !binding.read_project_ids.is_empty() {
        bail!("General tasks cannot read project scopes");
    }
    if previous.bound {
        if previous != binding {
            bail!("Task scope is immutable once bound");
        }
        return Ok(());
    }
    write_binding(db, task_id, &binding)?;
    Ok(())
}
