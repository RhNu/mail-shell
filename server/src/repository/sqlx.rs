use async_trait::async_trait;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::path::Path;

use crate::models::{AttachmentDownloadMeta, AttachmentMeta, MessageDetail, MessageSummary, Tag};
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
}

#[async_trait]
impl Repository for SqlxRepository {
    #[tracing::instrument(skip(self, record), fields(message_id = %record.id))]
    async fn ingest_message(&self, record: InboundMessageRecord) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            "INSERT INTO messages (id, from_address, to_address, subject, date, message_id, raw_path, body_text, body_html)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .bind(&record.id)
        .bind(&record.from_address)
        .bind(&record.to_address)
        .bind(&record.subject)
        .bind(&record.date)
        .bind(&record.message_id)
        .bind(&record.raw_path)
        .bind(&record.body_text)
        .bind(&record.body_html)
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
                "SELECT m.*
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
                "SELECT *
                 FROM messages
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?1 OFFSET ?2",
            )
            .bind(query.limit)
            .bind(query.offset)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(MessagePage { items, total })
    }

    #[tracing::instrument(skip(self))]
    async fn get_message(&self, id: &str) -> Result<Option<MessageRecord>, RepositoryError> {
        let message = sqlx::query_as::<_, MessageDetail>("SELECT * FROM messages WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        let Some(message) = message else {
            return Ok(None);
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

        Ok(Some(MessageRecord {
            message,
            attachments,
        }))
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
        Ok(tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::{InboundAttachmentRecord, InboundTagRecord};

    fn inbound_record(id: &str, attachment_id: &str, message_id: &str) -> InboundMessageRecord {
        InboundMessageRecord {
            id: id.to_string(),
            from_address: "from@example.com".to_string(),
            to_address: "to@example.com".to_string(),
            subject: Some("Subject".to_string()),
            date: None,
            message_id: Some(message_id.to_string()),
            raw_path: format!("/tmp/{id}.eml"),
            body_text: Some("Body".to_string()),
            body_html: None,
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
}
