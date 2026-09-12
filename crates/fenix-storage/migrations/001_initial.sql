PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS records (
    kind TEXT NOT NULL, id TEXT NOT NULL, data TEXT NOT NULL,
    updated_at TEXT NOT NULL, PRIMARY KEY (kind, id)
);
CREATE TABLE IF NOT EXISTS tasks (
    seq INTEGER PRIMARY KEY AUTOINCREMENT, id TEXT UNIQUE NOT NULL,
    session_id TEXT NOT NULL, request TEXT NOT NULL, origin TEXT NOT NULL,
    status TEXT NOT NULL, schedule_id TEXT, created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL, error TEXT
);
CREATE INDEX IF NOT EXISTS tasks_queue ON tasks(status, seq);
CREATE TABLE IF NOT EXISTS messages (
    seq INTEGER PRIMARY KEY AUTOINCREMENT, id TEXT UNIQUE NOT NULL,
    task_id TEXT, role TEXT NOT NULL, text TEXT NOT NULL, body TEXT NOT NULL,
    visible INTEGER NOT NULL, in_context INTEGER NOT NULL, context_order INTEGER,
    origin TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS messages_task ON messages(task_id, seq);
CREATE INDEX IF NOT EXISTS messages_context ON messages(context_order);
CREATE VIRTUAL TABLE IF NOT EXISTS history_fts USING fts5(id UNINDEXED, text, tokenize='unicode61 remove_diacritics 2');
CREATE TABLE IF NOT EXISTS events (
    seq INTEGER PRIMARY KEY AUTOINCREMENT, id TEXT UNIQUE NOT NULL,
    task_id TEXT NOT NULL, kind TEXT NOT NULL, data TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS events_task ON events(task_id, seq);
CREATE TABLE IF NOT EXISTS memories (
    id TEXT PRIMARY KEY, title TEXT NOT NULL, content TEXT NOT NULL,
    source TEXT NOT NULL, normalized TEXT UNIQUE NOT NULL,
    created_at TEXT NOT NULL, updated_at TEXT NOT NULL
);
CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(id UNINDEXED, title, content, tokenize='unicode61 remove_diacritics 2');
CREATE TABLE IF NOT EXISTS outbox (
    seq INTEGER PRIMARY KEY AUTOINCREMENT, text TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending', attempts INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);
PRAGMA user_version = 1;
