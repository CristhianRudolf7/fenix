ALTER TABLE memories ADD COLUMN metadata TEXT NOT NULL DEFAULT '{}';
CREATE TABLE memory_revisions (
 seq INTEGER PRIMARY KEY AUTOINCREMENT, memory_id TEXT NOT NULL,
 project_id TEXT, snapshot TEXT NOT NULL, changed_at TEXT NOT NULL
);
CREATE TRIGGER memory_insert_audit AFTER INSERT ON memories BEGIN
 INSERT INTO memory_revisions(memory_id,project_id,snapshot,changed_at)
 VALUES(new.id,new.project_id,json_object('id',new.id,'title',new.title,'content',new.content,'source',new.source,'metadata',json(new.metadata)),new.updated_at);
END;
CREATE TRIGGER memory_update_audit AFTER UPDATE ON memories BEGIN
 INSERT INTO memory_revisions(memory_id,project_id,snapshot,changed_at)
 VALUES(new.id,new.project_id,json_object('id',new.id,'title',new.title,'content',new.content,'source',new.source,'metadata',json(new.metadata)),new.updated_at);
END;
INSERT INTO memory_revisions(memory_id,project_id,snapshot,changed_at)
 SELECT id,project_id,json_object('id',id,'title',title,'content',content,'source',source,'metadata',json(metadata)),updated_at FROM memories;
CREATE INDEX memory_revision_scope ON memory_revisions(project_id,seq);
CREATE TABLE memory_observations(memory_id TEXT NOT NULL REFERENCES memories(id), evidence_id TEXT NOT NULL, PRIMARY KEY(memory_id,evidence_id));
CREATE TABLE memory_withdrawals(scope TEXT NOT NULL, normalized TEXT NOT NULL, PRIMARY KEY(scope,normalized));
CREATE TABLE knowledge_entities(id TEXT PRIMARY KEY, scope TEXT NOT NULL, body TEXT NOT NULL);
CREATE TABLE knowledge_aliases(scope TEXT NOT NULL, alias TEXT NOT NULL, entity_id TEXT NOT NULL REFERENCES knowledge_entities(id), PRIMARY KEY(scope,alias));
CREATE TABLE knowledge_schemas(scope TEXT NOT NULL, name TEXT NOT NULL, version INTEGER NOT NULL, body TEXT NOT NULL, PRIMARY KEY(scope,name,version));
CREATE TABLE knowledge_relations(id TEXT PRIMARY KEY, scope TEXT NOT NULL, source_id TEXT NOT NULL REFERENCES knowledge_entities(id), target_id TEXT NOT NULL REFERENCES knowledge_entities(id), predicate TEXT NOT NULL, body TEXT NOT NULL);
CREATE INDEX knowledge_neighbors ON knowledge_relations(scope,source_id,target_id);
CREATE TABLE memory_jobs(id TEXT PRIMARY KEY, task_id TEXT NOT NULL REFERENCES tasks(id), body TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'queued', attempts INTEGER NOT NULL DEFAULT 0, error TEXT, created_at TEXT NOT NULL);
PRAGMA user_version=7;
CREATE TABLE memory_embeddings(memory_id TEXT PRIMARY KEY REFERENCES memories(id), model TEXT NOT NULL, content TEXT NOT NULL, vector TEXT NOT NULL);
CREATE TABLE memory_review_pairs(first_id TEXT NOT NULL REFERENCES memories(id), second_id TEXT NOT NULL REFERENCES memories(id), reason TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'pending', PRIMARY KEY(first_id,second_id));
CREATE TABLE memory_maintenance_cursor(id INTEGER PRIMARY KEY CHECK(id=1), seq INTEGER NOT NULL);
INSERT INTO memory_maintenance_cursor VALUES (1,0);
CREATE TABLE knowledge_revisions(seq INTEGER PRIMARY KEY AUTOINCREMENT, scope TEXT NOT NULL, record_id TEXT NOT NULL, category TEXT NOT NULL, body TEXT NOT NULL);
CREATE TRIGGER entity_created AFTER INSERT ON knowledge_entities BEGIN INSERT INTO knowledge_revisions(scope,record_id,category,body) VALUES(new.scope,new.id,'entity',new.body); END;
CREATE TRIGGER entity_updated AFTER UPDATE ON knowledge_entities BEGIN INSERT INTO knowledge_revisions(scope,record_id,category,body) VALUES(new.scope,new.id,'entity',new.body); END;
CREATE TRIGGER relation_created AFTER INSERT ON knowledge_relations BEGIN INSERT INTO knowledge_revisions(scope,record_id,category,body) VALUES(new.scope,new.id,'relation',new.body); END;
CREATE TRIGGER relation_updated AFTER UPDATE ON knowledge_relations BEGIN INSERT INTO knowledge_revisions(scope,record_id,category,body) VALUES(new.scope,new.id,'relation',new.body); END;
CREATE TABLE memory_changes(seq INTEGER PRIMARY KEY AUTOINCREMENT, project_id TEXT, record_id TEXT NOT NULL, category TEXT NOT NULL, body TEXT NOT NULL, changed_at TEXT NOT NULL);
CREATE INDEX memory_changes_scope ON memory_changes(project_id,seq);
INSERT INTO memory_changes(project_id,record_id,category,body,changed_at) SELECT project_id,memory_id,'memory',snapshot,changed_at FROM memory_revisions ORDER BY seq;
CREATE TRIGGER memory_changed AFTER INSERT ON memory_revisions BEGIN INSERT INTO memory_changes(project_id,record_id,category,body,changed_at) VALUES(new.project_id,new.memory_id,'memory',new.snapshot,new.changed_at); END;
CREATE TRIGGER knowledge_changed AFTER INSERT ON knowledge_revisions BEGIN INSERT INTO memory_changes(project_id,record_id,category,body,changed_at) VALUES(nullif(new.scope,''),new.record_id,new.category,new.body,strftime('%Y-%m-%dT%H:%M:%fZ','now')); END;
CREATE TRIGGER schema_changed AFTER INSERT ON knowledge_schemas BEGIN INSERT INTO memory_changes(project_id,record_id,category,body,changed_at) VALUES(nullif(new.scope,''),new.name,'schema',json_object('version',new.version,'schema',json(new.body)),strftime('%Y-%m-%dT%H:%M:%fZ','now')); END;
