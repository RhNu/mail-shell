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
    ingest_fingerprint TEXT,
    snapshot_version INTEGER NOT NULL,
    parsed_snapshot TEXT NOT NULL,
    mailbox TEXT NOT NULL DEFAULT 'inbox' CHECK (mailbox IN ('inbox', 'archive')),
    read_at DATETIME,
    is_starred INTEGER NOT NULL DEFAULT 0 CHECK (is_starred IN (0, 1)),
    snoozed_until DATETIME,
    trashed_at DATETIME,
    received_at DATETIME,
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

CREATE TABLE message_addresses (
    message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('from', 'to', 'cc', 'bcc', 'reply_to', 'sender', 'envelope_to')),
    position INTEGER NOT NULL,
    name TEXT,
    address TEXT NOT NULL,
    domain TEXT,
    PRIMARY KEY (message_id, role, position)
);

CREATE INDEX idx_messages_created_at_id ON messages(created_at DESC, id DESC);
CREATE INDEX idx_attachments_message_id ON attachments(message_id);
CREATE UNIQUE INDEX idx_messages_ingest_fingerprint
ON messages(ingest_fingerprint)
WHERE ingest_fingerprint IS NOT NULL;
CREATE INDEX idx_messages_mailbox_created_at_id
ON messages(mailbox, created_at DESC, id DESC);
CREATE INDEX idx_messages_trash_mailbox_created
ON messages(trashed_at, mailbox, created_at DESC, id DESC);
CREATE INDEX idx_messages_unread
ON messages(read_at, trashed_at, created_at DESC, id DESC);
CREATE INDEX idx_messages_starred
ON messages(is_starred, trashed_at, created_at DESC, id DESC);
CREATE INDEX idx_message_addresses_address_message
ON message_addresses(address, message_id);
CREATE INDEX idx_message_addresses_domain_message
ON message_addresses(domain, message_id);

CREATE VIRTUAL TABLE message_fts USING fts5(
    message_id UNINDEXED,
    subject,
    sender,
    recipients,
    body
);

CREATE TABLE labels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL COLLATE NOCASE UNIQUE,
    color TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE message_labels (
    message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    label_id INTEGER NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
    origin TEXT NOT NULL DEFAULT 'manual' CHECK (origin IN ('manual', 'rule')),
    rule_id INTEGER,
    PRIMARY KEY (message_id, label_id)
);

CREATE INDEX idx_message_labels_label_message ON message_labels(label_id, message_id);

CREATE TABLE rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    priority INTEGER NOT NULL DEFAULT 0,
    stop_processing INTEGER NOT NULL DEFAULT 0 CHECK (stop_processing IN (0, 1)),
    conditions_json TEXT NOT NULL,
    actions_json TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_rules_enabled_priority ON rules(enabled, priority DESC, id);

CREATE TABLE saved_views (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    query_json TEXT NOT NULL,
    pinned INTEGER NOT NULL DEFAULT 0 CHECK (pinned IN (0, 1)),
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
