use async_trait::async_trait;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::path::Path;

use crate::error::AppError;
use crate::models::{AttachmentDownloadMeta, AttachmentMeta, MessageDetail, MessageSummary, Tag};
use crate::repository::Repository;

/// SQL script that creates the initial database schema.
const MIGRATE_SQL: &str = include_str!("../sql/migrate.sql");

/// Filename for the SQLite database file within the data directory.
const DB_FILENAME: &str = "index.sqlite";

#[derive(Debug, Clone)]
pub struct SqlxRepository {
    pub(crate) pool: SqlitePool,
}

impl SqlxRepository {
    fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Initialize an on-disk SQLite connection pool, run migrations, and seed tags.
    #[tracing::instrument]
    pub async fn init_pool(data_dir: &Path) -> Result<Self, sqlx::Error> {
        let db_path = data_dir.join(DB_FILENAME);
        tracing::info!(db_path = %db_path.display(), "connecting to database");

        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options).await?;
        Self::migrate(&pool).await?;
        Self::seed_tags(&pool).await?;
        tracing::info!("database initialized");
        Ok(Self::new(pool))
    }

    /// Initialize an in-memory SQLite pool for tests.
    #[tracing::instrument]
    pub async fn init_pool_in_memory() -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        Self::migrate(&pool).await?;
        Self::seed_tags(&pool).await?;
        Ok(Self::new(pool))
    }

    /// Run the schema migration (CREATE TABLE IF NOT EXISTS).
    #[tracing::instrument(skip(pool))]
    async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(MIGRATE_SQL).execute(pool).await?;
        tracing::debug!("migration executed");
        Ok(())
    }

    /// No static seeding required — tags are created on-demand.
    async fn seed_tags(_pool: &SqlitePool) -> Result<(), sqlx::Error> {
        Ok(())
    }
}

#[async_trait]
impl Repository for SqlxRepository {
    #[tracing::instrument(skip(self))]
    async fn count_messages(&self, tag_id: Option<i64>) -> Result<i64, AppError> {
        let total = if let Some(tag_id) = tag_id {
            sqlx::query_scalar("SELECT COUNT(*) FROM message_tags WHERE tag_id = ?1")
                .bind(tag_id)
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM messages")
                .fetch_one(&self.pool)
                .await?
        };
        Ok(total)
    }

    #[tracing::instrument(skip(self))]
    async fn list_messages(
        &self,
        tag_id: Option<i64>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<MessageSummary>, AppError> {
        let items = if let Some(tag_id) = tag_id {
            sqlx::query_as::<_, MessageSummary>(
                "SELECT m.* FROM messages m
                 JOIN message_tags mt ON mt.message_id = m.id
                 WHERE mt.tag_id = ?1
                 ORDER BY m.created_at DESC
                 LIMIT ?2 OFFSET ?3",
            )
            .bind(tag_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, MessageSummary>(
                "SELECT * FROM messages ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };
        Ok(items)
    }

    #[tracing::instrument(skip(self))]
    async fn get_message_detail(&self, id: &str) -> Result<Option<MessageDetail>, AppError> {
        let message = sqlx::query_as::<_, MessageDetail>("SELECT * FROM messages WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(message)
    }

    #[tracing::instrument(skip(self))]
    async fn list_attachments_by_message(
        &self,
        message_id: &str,
    ) -> Result<Vec<AttachmentMeta>, AppError> {
        let attachments = sqlx::query_as::<_, AttachmentMeta>(
            "SELECT id, message_id, filename, content_type, size FROM attachments WHERE message_id = ?1",
        )
        .bind(message_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(attachments)
    }

    #[tracing::instrument(skip(self))]
    async fn get_attachment_meta(
        &self,
        id: &str,
    ) -> Result<Option<AttachmentDownloadMeta>, AppError> {
        let row = sqlx::query_as::<_, AttachmentDownloadMeta>(
            "SELECT path, filename, content_type FROM attachments WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    #[tracing::instrument(skip(self))]
    #[allow(clippy::too_many_arguments)]
    async fn create_message(
        &self,
        id: &str,
        from_address: &str,
        to_address: &str,
        subject: Option<&str>,
        date: Option<&str>,
        message_id: Option<&str>,
        raw_path: &str,
        body_text: Option<&str>,
        body_html: Option<&str>,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO messages (id, from_address, to_address, subject, date, message_id, raw_path, body_text, body_html)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .bind(id)
        .bind(from_address)
        .bind(to_address)
        .bind(subject)
        .bind(date)
        .bind(message_id)
        .bind(raw_path)
        .bind(body_text)
        .bind(body_html)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn create_attachment(
        &self,
        id: &str,
        message_id: &str,
        filename: Option<&str>,
        content_type: Option<&str>,
        size: i64,
        path: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO attachments (id, message_id, filename, content_type, size, path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(id)
        .bind(message_id)
        .bind(filename)
        .bind(content_type)
        .bind(size)
        .bind(path)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn ensure_tag(&self, kind: &str, value: &str, label: &str) -> Result<i64, AppError> {
        let id: Option<i64> =
            sqlx::query_scalar("SELECT id FROM tags WHERE kind = ?1 AND value = ?2")
                .bind(kind)
                .bind(value)
                .fetch_optional(&self.pool)
                .await?;

        if let Some(id) = id {
            tracing::debug!(tag_id = id, kind, value, "tag already exists");
            return Ok(id);
        }

        let id = sqlx::query_scalar(
            "INSERT INTO tags (kind, value, label, source) VALUES (?1, ?2, ?3, 'system') RETURNING id"
        )
        .bind(kind)
        .bind(value)
        .bind(label)
        .fetch_one(&self.pool)
        .await?;
        tracing::info!(tag_id = id, kind, value, "created new tag");

        Ok(id)
    }

    #[tracing::instrument(skip(self))]
    async fn link_message_tag(&self, message_id: &str, tag_id: i64) -> Result<(), AppError> {
        sqlx::query("INSERT OR IGNORE INTO message_tags (message_id, tag_id) VALUES (?1, ?2)")
            .bind(message_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await?;
        tracing::debug!(message_id, tag_id, "linked message to tag");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn list_tags(&self) -> Result<Vec<Tag>, AppError> {
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
        tracing::debug!(count = tags.len(), "listed tags");
        Ok(tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_migrate_creates_schema() {
        let repo = SqlxRepository::init_pool_in_memory().await.unwrap();
        let tables: Vec<String> =
            sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
                .fetch_all(&repo.pool)
                .await
                .unwrap();
        assert!(tables.contains(&"messages".to_string()));
        assert!(tables.contains(&"attachments".to_string()));
        assert!(tables.contains(&"tags".to_string()));
        assert!(tables.contains(&"message_tags".to_string()));
    }

    #[tokio::test]
    async fn test_seed_tags_idempotent() {
        let repo = SqlxRepository::init_pool_in_memory().await.unwrap();
        SqlxRepository::seed_tags(&repo.pool).await.unwrap();
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tags")
            .fetch_one(&repo.pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
}
