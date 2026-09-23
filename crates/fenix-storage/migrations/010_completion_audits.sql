CREATE TABLE completion_audits (
    id TEXT PRIMARY KEY,
    plan_id TEXT NOT NULL REFERENCES plans(id),
    version INTEGER NOT NULL,
    node_id TEXT NOT NULL,
    worker_id TEXT UNIQUE REFERENCES tasks(id),
    status TEXT NOT NULL,
    body TEXT NOT NULL,
    UNIQUE(plan_id, version, node_id)
);
CREATE TABLE plan_evidence (
    plan_id TEXT NOT NULL REFERENCES plans(id),
    version INTEGER NOT NULL,
    entry_id TEXT NOT NULL,
    body TEXT NOT NULL,
    PRIMARY KEY(plan_id, version, entry_id)
);
PRAGMA user_version = 10;
