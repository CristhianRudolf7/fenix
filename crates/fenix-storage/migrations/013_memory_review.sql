-- Assessments refer to exact revisions. They never mutate the underlying claims.
ALTER TABLE memory_review_pairs ADD COLUMN first_revision INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_review_pairs ADD COLUMN second_revision INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_review_pairs ADD COLUMN assessment TEXT;
ALTER TABLE memory_review_pairs ADD COLUMN attempts INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_review_pairs ADD COLUMN review_after TEXT;
CREATE INDEX memory_review_pending ON memory_review_pairs(status, attempts);
CREATE INDEX memory_revision_record ON memory_revisions(memory_id, seq);
UPDATE memory_review_pairs SET
 first_revision=(SELECT max(seq) FROM memory_revisions WHERE memory_id=first_id),
 second_revision=(SELECT max(seq) FROM memory_revisions WHERE memory_id=second_id);
-- Existing edges get a migration baseline, not a retrospective validity claim.
UPDATE knowledge_relations SET body=json_set(body,'$.evidence_revision',
 (SELECT max(seq) FROM memory_revisions WHERE memory_id=json_extract(knowledge_relations.body,'$.evidence_memory_id')));
PRAGMA user_version=13;
