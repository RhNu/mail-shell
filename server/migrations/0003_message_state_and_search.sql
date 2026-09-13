ALTER TABLE messages ADD COLUMN read_at DATETIME;
ALTER TABLE messages ADD COLUMN is_starred INTEGER NOT NULL DEFAULT 0 CHECK (is_starred IN (0, 1));
ALTER TABLE messages ADD COLUMN snoozed_until DATETIME;
ALTER TABLE messages ADD COLUMN trashed_at DATETIME;
ALTER TABLE messages ADD COLUMN received_at DATETIME;
ALTER TABLE messages ADD COLUMN ingest_fingerprint TEXT;

UPDATE messages SET received_at = created_at WHERE received_at IS NULL;

CREATE UNIQUE INDEX idx_messages_ingest_fingerprint
ON messages(ingest_fingerprint)
WHERE ingest_fingerprint IS NOT NULL;

CREATE INDEX idx_messages_trash_mailbox_created
ON messages(trashed_at, mailbox, created_at DESC, id DESC);

CREATE INDEX idx_messages_unread
ON messages(read_at, trashed_at, created_at DESC, id DESC);

CREATE INDEX idx_messages_starred
ON messages(is_starred, trashed_at, created_at DESC, id DESC);

CREATE TABLE message_addresses (
    message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('from', 'to', 'cc', 'bcc', 'reply_to', 'sender', 'envelope_to')),
    position INTEGER NOT NULL,
    name TEXT,
    address TEXT NOT NULL,
    domain TEXT,
    PRIMARY KEY (message_id, role, position)
);

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

CREATE TABLE app_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT INTO app_metadata(key, value) VALUES ('derived_index_version', '0');
