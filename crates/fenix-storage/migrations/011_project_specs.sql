CREATE TABLE project_specs (
    project_id TEXT PRIMARY KEY REFERENCES projects(id),
    version INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE project_requirements (
    project_id TEXT NOT NULL REFERENCES projects(id),
    id TEXT NOT NULL,
    body TEXT NOT NULL,
    PRIMARY KEY(project_id, id)
);
CREATE TABLE project_spec_changes (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    task_id TEXT NOT NULL REFERENCES tasks(id),
    status TEXT NOT NULL CHECK(status IN ('proposed', 'applied', 'discarded')),
    body TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX project_spec_changes_scope ON project_spec_changes(project_id, status, created_at, id);
CREATE INDEX project_spec_changes_task ON project_spec_changes(task_id, status);
PRAGMA user_version = 11;
