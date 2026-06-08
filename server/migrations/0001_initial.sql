CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    message_id TEXT UNIQUE,
    subject TEXT NOT NULL DEFAULT '',
    from_name TEXT,
    from_address TEXT NOT NULL,
    to_name TEXT,
    to_address TEXT,
    envelope_to TEXT NOT NULL,
    date TEXT,
    raw_path TEXT NOT NULL,
    snapshot_version INTEGER NOT NULL,
    parsed_snapshot TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE attachments (
    id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    filename TEXT,
    content_type TEXT,
    size INTEGER NOT NULL,
    path TEXT NOT NULL
);

CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    kind TEXT NOT NULL,
    value TEXT NOT NULL,
    label TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'system',
    UNIQUE(kind, value)
);

CREATE TABLE message_tags (
    message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (message_id, tag_id)
);

CREATE INDEX idx_messages_created_at_id ON messages(created_at DESC, id DESC);
CREATE INDEX idx_attachments_message_id ON attachments(message_id);
CREATE INDEX idx_message_tags_tag_id_message_id ON message_tags(tag_id, message_id);
CREATE INDEX idx_message_tags_message_id ON message_tags(message_id);
