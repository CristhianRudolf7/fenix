CREATE TABLE subagents (
    id TEXT PRIMARY KEY REFERENCES tasks(id),
    parent_task_id TEXT NOT NULL REFERENCES tasks(id),
    spec TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts BETWEEN 0 AND 3),
    result TEXT,
    report_task_id TEXT UNIQUE REFERENCES tasks(id),
    UNIQUE (parent_task_id, spec)
);
CREATE INDEX subagents_parent ON subagents(parent_task_id);
PRAGMA user_version = 2;
