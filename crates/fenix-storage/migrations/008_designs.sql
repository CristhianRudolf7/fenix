CREATE TABLE designs (
 id TEXT PRIMARY KEY,
 project_id TEXT REFERENCES projects(id),
 task_id TEXT REFERENCES tasks(id),
 title TEXT NOT NULL,
 kind TEXT NOT NULL,
 revision INTEGER NOT NULL,
 data TEXT NOT NULL,
 created_at TEXT NOT NULL,
 updated_at TEXT NOT NULL
);
CREATE INDEX designs_project_updated ON designs(project_id, updated_at DESC);
CREATE INDEX designs_task ON designs(task_id);
CREATE TABLE design_revisions (
 design_id TEXT NOT NULL REFERENCES designs(id),
 revision INTEGER NOT NULL,
 data TEXT NOT NULL,
 PRIMARY KEY(design_id, revision)
);
PRAGMA user_version = 8;
