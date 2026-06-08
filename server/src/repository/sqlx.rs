use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::path::Path;

use crate::mime_parser::{ParsedMailSnapshotV1, SNAPSHOT_VERSION};
use crate::models::{
    AttachmentDownloadMeta, AttachmentMeta, HeaderEntry, MessageDetail, MessageRawMeta,
    MessageSummary, Tag,
};
use crate::repository::{
    InboundMessageRecord, ListMessagesQuery, MessagePage, MessageRecord, Repository,
    RepositoryError,
};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

const DB_FILENAME: &str = "index.sqlite";

#[derive(Debug, Clone)]
pub struct SqlxRepository {
    pub(crate) pool: SqlitePool,
}

#[derive(Debug, sqlx::FromRow)]
struct StoredMessage {
    id: String,
    message_id: Option<String>,
    subject: String,
    from_name: Option<String>,
    from_address: String,
    to_name: Option<String>,
    to_address: Option<String>,
    envelope_to: String,
    date: Option<String>,
    snapshot_version: i64,
    parsed_snapshot: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct StoredSnapshot {
    id: String,
    snapshot_version: i64,
    parsed_snapshot: String,
}

impl SqlxRepository {
    fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    #[tracing::instrument]
    pub async fn init_pool(data_dir: &Path) -> Result<Self, sqlx::Error> {
        let db_path = data_dir.join(DB_FILENAME);
        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new().connect_with(options).await?;
        Self::migrate(&pool).await?;
        Ok(Self::new(pool))
    }

    #[tracing::instrument]
    pub async fn init_pool_in_memory() -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::new()
            .in_memory(true)
            .shared_cache(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;
        Self::migrate(&pool).await?;
        Ok(Self::new(pool))
    }

    #[tracing::instrument(skip(pool))]
    async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        MIGRATOR.run(pool).await?;
        Ok(())
    }

    fn decode_snapshot(
        message_id: &str,
        snapshot_version: i64,
        parsed_snapshot: &str,
    ) -> Result<ParsedMailSnapshotV1, RepositoryError> {
        if snapshot_version != SNAPSHOT_VERSION {
            return Err(RepositoryError::UnsupportedSnapshotVersion {
                message_id: message_id.to_string(),
                version: snapshot_version,
            });
        }

        let snapshot: ParsedMailSnapshotV1 =
            serde_json::from_str(parsed_snapshot).map_err(|source| {
                RepositoryError::InvalidSnapshot {
                    message_id: message_id.to_string(),
                    source,
                }
            })?;
        if snapshot.version != snapshot_version {
            return Err(RepositoryError::InvalidSnapshotData {
                message_id: message_id.to_string(),
                reason: format!(
                    "row version {snapshot_version} does not match payload version {}",
                    snapshot.version
                ),
            });
        }
        snapshot
            .validate_for_storage()
            .map_err(|error| RepositoryError::InvalidSnapshotData {
                message_id: message_id.to_string(),
                reason: error.to_string(),
            })?;
        Ok(snapshot)
    }
}

#[async_trait]
impl Repository for SqlxRepository {
    #[tracing::instrument(skip(self, record), fields(message_id = %record.id))]
    async fn ingest_message(&self, record: InboundMessageRecord) -> Result<(), RepositoryError> {
        record.snapshot.validate_for_storage().map_err(|error| {
            RepositoryError::InvalidSnapshotData {
                message_id: record.id.clone(),
                reason: error.to_string(),
            }
        })?;
        let parsed_snapshot = serde_json::to_string(&record.snapshot).map_err(|source| {
            RepositoryError::InvalidSnapshot {
                message_id: record.id.clone(),
                source,
            }
        })?;
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            "INSERT INTO messages (id, message_id, subject, from_name, from_address, to_name, to_address, envelope_to, date, raw_path, snapshot_version, parsed_snapshot)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        )
        .bind(&record.id)
        .bind(&record.message_id)
        .bind(&record.subject)
        .bind(&record.from_name)
        .bind(&record.from_address)
        .bind(&record.to_name)
        .bind(&record.to_address)
        .bind(&record.envelope_to)
        .bind(&record.date)
        .bind(&record.raw_path)
        .bind(SNAPSHOT_VERSION)
        .bind(&parsed_snapshot)
        .execute(&mut *tx)
        .await?;

        for attachment in &record.attachments {
            sqlx::query(
                "INSERT INTO attachments (id, message_id, filename, content_type, size, path)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(&attachment.id)
            .bind(&record.id)
            .bind(&attachment.filename)
            .bind(&attachment.content_type)
            .bind(attachment.size)
            .bind(&attachment.path)
            .execute(&mut *tx)
            .await?;
        }

        for tag in &record.tags {
            let tag_id: i64 = sqlx::query_scalar(
                "INSERT INTO tags (kind, value, label, source)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(kind, value) DO UPDATE SET
                     label = excluded.label,
                     source = excluded.source
                 RETURNING id",
            )
            .bind(&tag.kind)
            .bind(&tag.value)
            .bind(&tag.label)
            .bind(&tag.source)
            .fetch_one(&mut *tx)
            .await?;

            sqlx::query("INSERT OR IGNORE INTO message_tags (message_id, tag_id) VALUES (?1, ?2)")
                .bind(&record.id)
                .bind(tag_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        tracing::debug!(
            message_id = %record.id,
            attachment_count = record.attachments.len(),
            tag_count = record.tags.len(),
            "ingested message"
        );
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn list_messages(
        &self,
        query: ListMessagesQuery,
    ) -> Result<MessagePage<MessageSummary>, RepositoryError> {
        let total = if let Some(tag_id) = query.tag_id {
            sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM messages m
                 JOIN message_tags mt ON mt.message_id = m.id
                 WHERE mt.tag_id = ?1",
            )
            .bind(tag_id)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM messages")
                .fetch_one(&self.pool)
                .await?
        };

        let items = if let Some(tag_id) = query.tag_id {
            sqlx::query_as::<_, MessageSummary>(
                "SELECT m.id, m.from_name, m.from_address, m.to_name, m.to_address,
                        m.envelope_to, m.subject, m.date, m.message_id, m.created_at
                 FROM messages m
                 JOIN message_tags mt ON mt.message_id = m.id
                 WHERE mt.tag_id = ?1
                 ORDER BY m.created_at DESC, m.id DESC
                 LIMIT ?2 OFFSET ?3",
            )
            .bind(tag_id)
            .bind(query.limit)
            .bind(query.offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, MessageSummary>(
                "SELECT id, from_name, from_address, to_name, to_address,
                        envelope_to, subject, date, message_id, created_at
                 FROM messages
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?1 OFFSET ?2",
            )
            .bind(query.limit)
            .bind(query.offset)
            .fetch_all(&self.pool)
            .await?
        };

        tracing::debug!(
            total,
            returned_count = items.len(),
            tag_filter = ?query.tag_id,
            limit = query.limit,
            offset = query.offset,
            "listed messages"
        );
        Ok(MessagePage { items, total })
    }

    #[tracing::instrument(skip(self))]
    async fn get_message(&self, id: &str) -> Result<Option<MessageRecord>, RepositoryError> {
        let stored = sqlx::query_as::<_, StoredMessage>(
            "SELECT id, message_id, subject, from_name, from_address, to_name, to_address,
                    envelope_to, date, snapshot_version, parsed_snapshot, created_at
             FROM messages
             WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(stored) = stored else {
            return Ok(None);
        };
        let snapshot =
            Self::decode_snapshot(&stored.id, stored.snapshot_version, &stored.parsed_snapshot)?;
        let message = MessageDetail {
            id: stored.id,
            from_name: stored.from_name,
            from_address: stored.from_address,
            to_name: stored.to_name,
            to_address: stored.to_address,
            envelope_to: stored.envelope_to,
            cc: addresses_to_json(&snapshot.cc),
            reply_to: addresses_to_json(&snapshot.reply_to),
            in_reply_to: snapshot.in_reply_to.first().cloned(),
            subject: stored.subject,
            date: stored.date,
            message_id: stored.message_id,
            body_text: snapshot.primary_body_text,
            body_html: snapshot.primary_body_html,
            created_at: stored.created_at,
        };

        let attachments = sqlx::query_as::<_, AttachmentMeta>(
            "SELECT id, message_id, filename, content_type, size
             FROM attachments
             WHERE message_id = ?1
             ORDER BY id",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;

        tracing::debug!(message_id = %id, found = true, "retrieved message detail");
        Ok(Some(MessageRecord {
            message,
            attachments,
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn get_message_headers(
        &self,
        id: &str,
    ) -> Result<Option<Vec<HeaderEntry>>, RepositoryError> {
        let stored = sqlx::query_as::<_, StoredSnapshot>(
            "SELECT id, snapshot_version, parsed_snapshot FROM messages WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(stored) = stored else {
            return Ok(None);
        };
        let snapshot =
            Self::decode_snapshot(&stored.id, stored.snapshot_version, &stored.parsed_snapshot)?;
        Ok(Some(
            snapshot
                .headers
                .into_iter()
                .map(|header| HeaderEntry {
                    name: header.name,
                    value: header.value,
                })
                .collect(),
        ))
    }

    #[tracing::instrument(skip(self))]
    async fn get_attachment_download(
        &self,
        id: &str,
    ) -> Result<Option<AttachmentDownloadMeta>, RepositoryError> {
        let row = sqlx::query_as::<_, AttachmentDownloadMeta>(
            "SELECT path, filename, content_type FROM attachments WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        tracing::debug!(attachment_id = %id, found = row.is_some(), "retrieved attachment download meta");
        Ok(row)
    }

    #[tracing::instrument(skip(self))]
    async fn get_message_raw(&self, id: &str) -> Result<Option<MessageRawMeta>, RepositoryError> {
        let row = sqlx::query_as::<_, MessageRawMeta>(
            "SELECT raw_path, subject FROM messages WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        tracing::debug!(message_id = %id, found = row.is_some(), "retrieved message raw meta");
        Ok(row)
    }

    #[tracing::instrument(skip(self))]
    async fn list_tags(&self) -> Result<Vec<Tag>, RepositoryError> {
        let tags = sqlx::query_as::<_, Tag>(
            r#"
            SELECT
                t.id,
                t.kind,
                t.value,
                t.label,
                t.source,
                COUNT(mt.message_id) AS message_count
            FROM tags t
            LEFT JOIN message_tags mt ON mt.tag_id = t.id
            GROUP BY t.id
            ORDER BY t.kind, t.value
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        tracing::debug!(tag_count = tags.len(), "listed tags");
        Ok(tags)
    }
}

fn addresses_to_json(addresses: &[crate::mime_parser::MailAddress]) -> Option<String> {
    if addresses.is_empty() {
        None
    } else {
        serde_json::to_string(addresses).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::{InboundAttachmentRecord, InboundTagRecord};

    fn inbound_record(id: &str, attachment_id: &str, message_id: &str) -> InboundMessageRecord {
        let raw = format!(
            "From: From <from@example.com>\r\nTo: to@example.com\r\nSubject: Subject\r\nMessage-ID: {message_id}\r\nContent-Type: multipart/mixed; boundary=\"boundary123\"\r\n\r\n--boundary123\r\nContent-Type: text/plain\r\n\r\nBody\r\n--boundary123\r\nContent-Type: text/plain\r\nContent-Disposition: attachment; filename=\"hello.txt\"\r\n\r\ntext\r\n--boundary123--"
        );
        let mut parsed = crate::mime_parser::parse_message(raw.as_bytes()).unwrap();
        let part_id = parsed.attachments[0].part_id;
        parsed
            .snapshot
            .bind_attachment_id(part_id, attachment_id.to_string())
            .unwrap();

        InboundMessageRecord {
            id: id.to_string(),
            message_id: Some(message_id.to_string()),
            subject: "Subject".to_string(),
            from_name: Some("From".to_string()),
            from_address: "from@example.com".to_string(),
            to_name: None,
            to_address: Some("to@example.com".to_string()),
            envelope_to: "to@example.com".to_string(),
            date: Some("2024-01-01T00:00:00+00:00".to_string()),
            raw_path: format!("/tmp/{id}.eml"),
            snapshot: parsed.snapshot,
            attachments: vec![InboundAttachmentRecord {
                id: attachment_id.to_string(),
                filename: Some("hello.txt".to_string()),
                content_type: Some("text/plain".to_string()),
                size: 4,
                path: format!("/tmp/{attachment_id}.txt"),
            }],
            tags: vec![InboundTagRecord {
                kind: "recipient_address".to_string(),
                value: "to@example.com".to_string(),
                label: "To: to@example.com".to_string(),
                source: "system".to_string(),
            }],
        }
    }

    #[tokio::test]
    async fn migrate_creates_schema() {
        let repo = SqlxRepository::init_pool_in_memory().await.unwrap();
        let tables: Vec<String> =
            sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
                .fetch_all(&repo.pool)
                .await
                .unwrap();
        assert!(tables.contains(&"_sqlx_migrations".to_string()));
        assert!(tables.contains(&"messages".to_string()));
        assert!(tables.contains(&"attachments".to_string()));
        assert!(tables.contains(&"tags".to_string()));
        assert!(tables.contains(&"message_tags".to_string()));

        let columns: Vec<String> =
            sqlx::query_scalar("SELECT name FROM pragma_table_info('messages')")
                .fetch_all(&repo.pool)
                .await
                .unwrap();
        assert!(columns.contains(&"snapshot_version".to_string()));
        assert!(columns.contains(&"parsed_snapshot".to_string()));
        assert!(!columns.contains(&"body_text".to_string()));
        assert!(!columns.contains(&"body_html".to_string()));
        assert!(!columns.contains(&"cc".to_string()));
        assert!(!columns.contains(&"reply_to".to_string()));
        assert!(!columns.contains(&"in_reply_to".to_string()));
    }

    #[tokio::test]
    async fn ingest_message_is_transactional() {
        let repo = SqlxRepository::init_pool_in_memory().await.unwrap();

        repo.ingest_message(inbound_record("msg-1", "att-1", "<dup>"))
            .await
            .unwrap();

        let duplicate = repo
            .ingest_message(inbound_record("msg-2", "att-2", "<dup>"))
            .await;
        assert!(duplicate.is_err());

        let page = repo
            .list_messages(ListMessagesQuery {
                tag_id: None,
                limit: 20,
                offset: 0,
            })
            .await
            .unwrap();
        assert_eq!(page.total, 1);

        let attachment_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM attachments")
            .fetch_one(&repo.pool)
            .await
            .unwrap();
        assert_eq!(attachment_count, 1);
    }

    #[tokio::test]
    async fn detail_and_headers_are_rebuilt_from_the_persisted_snapshot() {
        let repo = SqlxRepository::init_pool_in_memory().await.unwrap();
        let raw = b"From: Sender <from@example.com>\r\nTo: to@example.com\r\nCc: copy@example.com\r\nReply-To: reply@example.com\r\nIn-Reply-To: <parent@example.com>\r\nSubject: Snapshot Subject\r\nContent-Type: text/plain\r\n\r\nSnapshot body";
        let mut parsed = crate::mime_parser::parse_message(raw).unwrap();
        let attachment_ids: Vec<(u32, String)> = parsed
            .attachments
            .iter()
            .enumerate()
            .map(|(index, attachment)| (attachment.part_id, format!("att-{index}")))
            .collect();
        for (part_id, attachment_id) in attachment_ids {
            parsed
                .snapshot
                .bind_attachment_id(part_id, attachment_id)
                .unwrap();
        }

        let mut record = inbound_record("msg-1", "att-1", "<msg-1>");
        record.subject = parsed.subject.clone();
        record.snapshot = parsed.snapshot;
        repo.ingest_message(record).await.unwrap();

        let detail = repo.get_message("msg-1").await.unwrap().unwrap();
        assert_eq!(detail.message.subject, "Snapshot Subject");
        assert_eq!(detail.message.body_text.as_deref(), Some("Snapshot body"));
        assert_eq!(
            detail.message.cc.as_deref(),
            Some(r#"[{"name":null,"address":"copy@example.com"}]"#)
        );
        assert_eq!(
            detail.message.reply_to.as_deref(),
            Some(r#"[{"name":null,"address":"reply@example.com"}]"#)
        );
        assert_eq!(
            detail.message.in_reply_to.as_deref(),
            Some("parent@example.com")
        );

        let headers = repo.get_message_headers("msg-1").await.unwrap().unwrap();
        assert_eq!(
            headers
                .iter()
                .find(|header| header.name == "Subject")
                .unwrap()
                .value,
            "Snapshot Subject"
        );
    }

    #[tokio::test]
    async fn invalid_snapshot_json_and_version_are_repository_errors() {
        let repo = SqlxRepository::init_pool_in_memory().await.unwrap();
        repo.ingest_message(inbound_record("msg-1", "att-1", "<msg-1>"))
            .await
            .unwrap();

        sqlx::query("UPDATE messages SET parsed_snapshot = 'not-json' WHERE id = 'msg-1'")
            .execute(&repo.pool)
            .await
            .unwrap();
        assert!(matches!(
            repo.get_message("msg-1").await,
            Err(RepositoryError::InvalidSnapshot { .. })
        ));

        sqlx::query("UPDATE messages SET snapshot_version = 99 WHERE id = 'msg-1'")
            .execute(&repo.pool)
            .await
            .unwrap();
        assert!(matches!(
            repo.get_message("msg-1").await,
            Err(RepositoryError::UnsupportedSnapshotVersion { version: 99, .. })
        ));
    }
}
