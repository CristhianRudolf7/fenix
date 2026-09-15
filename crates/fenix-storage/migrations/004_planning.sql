CREATE TABLE plans (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL UNIQUE REFERENCES tasks(id),
    version INTEGER NOT NULL,
    status TEXT NOT NULL,
    body TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE TABLE plan_revisions (
    plan_id TEXT NOT NULL REFERENCES plans(id),
    version INTEGER NOT NULL,
    body TEXT NOT NULL,
    source_task_id TEXT NOT NULL REFERENCES tasks(id),
    reason TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY(plan_id, version)
);
CREATE TABLE plan_workers (
    task_id TEXT PRIMARY KEY REFERENCES tasks(id),
    plan_id TEXT NOT NULL REFERENCES plans(id),
    node_id TEXT NOT NULL,
    generation INTEGER NOT NULL,
    reconciled INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE bus_events (
    seq INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT NOT NULL UNIQUE,
    topic TEXT NOT NULL,
    task_id TEXT REFERENCES tasks(id),
    data TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE TABLE event_subscribers (
    name TEXT PRIMARY KEY,
    cursor INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE task_destinations (
    task_id TEXT PRIMARY KEY REFERENCES tasks(id),
    channel TEXT NOT NULL,
    address TEXT
);
CREATE TABLE questions (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    kind TEXT NOT NULL,
    status TEXT NOT NULL,
    body TEXT NOT NULL,
    answer TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
ALTER TABLE outbox ADD COLUMN destination TEXT;
ALTER TABLE outbox ADD COLUMN event_id TEXT;
ALTER TABLE outbox ADD COLUMN retry_at TEXT;
CREATE UNIQUE INDEX outbox_event ON outbox(event_id) WHERE event_id IS NOT NULL;
CREATE INDEX bus_topic_seq ON bus_events(topic, seq);
PRAGMA user_version = 4;
