CREATE TABLE plan_reviews (
    id TEXT PRIMARY KEY,
    plan_id TEXT NOT NULL REFERENCES plans(id),
    version INTEGER NOT NULL,
    status TEXT NOT NULL,
    body TEXT NOT NULL
);
CREATE INDEX plan_review_version ON plan_reviews(plan_id, version);
CREATE TABLE plan_review_workers (
    task_id TEXT PRIMARY KEY REFERENCES tasks(id),
    review_id TEXT NOT NULL REFERENCES plan_reviews(id)
);
PRAGMA user_version = 5;
