-- Disposable lexical index; the redacted event journal remains authoritative.
CREATE VIRTUAL TABLE tool_output_fts USING fts5(
    content, task_id UNINDEXED, event_seq UNINDEXED, call_id UNINDEXED,
    tool UNINDEXED, char_offset UNINDEXED,
    tokenize = 'unicode61 remove_diacritics 2'
);
CREATE TABLE tool_output_index_cursors (
    task_id TEXT PRIMARY KEY REFERENCES tasks(id) ON DELETE CASCADE,
    event_seq INTEGER NOT NULL
);
CREATE INDEX events_tool_output_call ON events(task_id, json_extract(data, '$.call_id'), seq)
    WHERE kind = 'tool_result';
CREATE TRIGGER tool_output_search_delete AFTER DELETE ON events BEGIN
    DELETE FROM tool_output_fts WHERE event_seq = old.seq;
END;
PRAGMA user_version = 12;
