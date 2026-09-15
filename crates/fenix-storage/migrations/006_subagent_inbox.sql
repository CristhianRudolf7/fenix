CREATE TABLE subagent_inputs (
    seq INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT UNIQUE NOT NULL,
    subagent_id TEXT NOT NULL REFERENCES subagents(id),
    sender_task_id TEXT NOT NULL REFERENCES tasks(id),
    content TEXT NOT NULL,
    delivered INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    UNIQUE(subagent_id, sender_task_id, content)
);
CREATE INDEX subagent_inputs_pending ON subagent_inputs(subagent_id, delivered, seq);
PRAGMA user_version = 6;
