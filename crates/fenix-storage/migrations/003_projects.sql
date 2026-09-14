CREATE TABLE projects (
    id TEXT PRIMARY KEY, name TEXT NOT NULL, aliases TEXT NOT NULL,
    description TEXT NOT NULL, workspace TEXT, brief TEXT NOT NULL,
    context_version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL, updated_at TEXT NOT NULL
);
CREATE TABLE project_aliases (
    alias TEXT PRIMARY KEY, project_id TEXT NOT NULL REFERENCES projects(id)
);
CREATE TABLE task_bindings (
    task_id TEXT PRIMARY KEY REFERENCES tasks(id),
    project_id TEXT REFERENCES projects(id),
    mode TEXT NOT NULL DEFAULT 'general',
    bound INTEGER NOT NULL DEFAULT 0,
    data TEXT NOT NULL
);
CREATE INDEX task_bindings_scope ON task_bindings(project_id,mode,task_id);
-- Existing conversations predate project routing and retain their general
-- scope. New arrivals get an explicit unbound row until routing is resolved.
INSERT INTO task_bindings(task_id,project_id,mode,bound,data)
SELECT id,NULL,'general',1,'{"mode":"general","bound":true}' FROM tasks;

ALTER TABLE memories RENAME TO legacy_memories;
CREATE TABLE memories (
    id TEXT PRIMARY KEY, title TEXT NOT NULL, content TEXT NOT NULL,
    source TEXT NOT NULL, normalized TEXT NOT NULL,
    created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
    project_id TEXT REFERENCES projects(id), task_id TEXT REFERENCES tasks(id)
);
INSERT INTO memories(id,title,content,source,normalized,created_at,updated_at)
SELECT id,title,content,source,normalized,created_at,updated_at FROM legacy_memories;
DROP TABLE legacy_memories;
CREATE UNIQUE INDEX memories_fact_scope ON memories(COALESCE(project_id,''), normalized);
CREATE INDEX memories_scope_updated ON memories(project_id,updated_at);

ALTER TABLE subagents RENAME TO legacy_subagents;
CREATE TABLE subagents (
    id TEXT PRIMARY KEY REFERENCES tasks(id),
    parent_task_id TEXT NOT NULL REFERENCES tasks(id),
    spec TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts BETWEEN 0 AND 3),
    result TEXT, report_task_id TEXT UNIQUE REFERENCES tasks(id),
    project_id TEXT REFERENCES projects(id)
);
INSERT INTO subagents(id,parent_task_id,spec,attempts,result,report_task_id)
SELECT id,parent_task_id,spec,attempts,result,report_task_id FROM legacy_subagents;
DROP TABLE legacy_subagents;
CREATE INDEX subagents_parent ON subagents(parent_task_id);
CREATE UNIQUE INDEX subagents_scope_purpose ON subagents(parent_task_id,COALESCE(project_id,''),spec);
ALTER TABLE outbox ADD COLUMN task_id TEXT REFERENCES tasks(id);
PRAGMA user_version = 3;
