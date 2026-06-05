CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    from_address TEXT NOT NULL,
    to_address TEXT NOT NULL,
    subject TEXT,
    date TEXT,
    message_id TEXT UNIQUE,
    raw_path TEXT NOT NULL,
    body_text TEXT,
    body_html TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS attachments (
    id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    filename TEXT,
    content_type TEXT,
    size INTEGER,
    path TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    kind TEXT NOT NULL,
    value TEXT NOT NULL,
    label TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'system',
    UNIQUE(kind, value)
);

CREATE TABLE IF NOT EXISTS message_tags (
    message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (message_id, tag_id)
);
