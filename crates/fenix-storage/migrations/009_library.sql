CREATE TABLE library_documents (
 id TEXT PRIMARY KEY, project_id TEXT REFERENCES projects(id), enabled INTEGER NOT NULL,
 revision INTEGER NOT NULL, data TEXT NOT NULL
);
CREATE INDEX library_scope ON library_documents(project_id, enabled);
CREATE TABLE library_revisions (
 document_id TEXT NOT NULL REFERENCES library_documents(id) ON DELETE CASCADE,
 revision INTEGER NOT NULL, status TEXT NOT NULL, embedding_status TEXT NOT NULL, data TEXT NOT NULL,
 PRIMARY KEY(document_id, revision)
);
CREATE TABLE library_chunks (
 id TEXT PRIMARY KEY, document_id TEXT NOT NULL, revision INTEGER NOT NULL,
 ordinal INTEGER NOT NULL, text TEXT NOT NULL, location TEXT NOT NULL, hash TEXT NOT NULL,
 embedding TEXT, model TEXT,
 FOREIGN KEY(document_id, revision) REFERENCES library_revisions(document_id, revision) ON DELETE CASCADE,
 UNIQUE(document_id, revision, ordinal)
);
CREATE INDEX library_chunk_version ON library_chunks(document_id, revision, ordinal);
CREATE INDEX library_chunk_hash ON library_chunks(document_id, hash, model);
CREATE VIRTUAL TABLE library_fts USING fts5(id UNINDEXED, title, text, tokenize='unicode61 remove_diacritics 2');
CREATE TRIGGER library_chunk_delete AFTER DELETE ON library_chunks BEGIN
 DELETE FROM library_fts WHERE id=OLD.id;
END;
PRAGMA user_version = 9;
