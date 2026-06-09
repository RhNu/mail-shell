ALTER TABLE messages
ADD COLUMN mailbox TEXT NOT NULL DEFAULT 'inbox' CHECK (mailbox IN ('inbox', 'archive'));

CREATE INDEX idx_messages_mailbox_created_at_id
ON messages(mailbox, created_at DESC, id DESC);
