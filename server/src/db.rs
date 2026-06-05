use std::path::Path;

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

use crate::models::Tag;

/// SQL script that creates the initial database schema.
const MIGRATE_SQL: &str = include_str!("sql/migrate.sql");

/// Initialize an on-disk SQLite connection pool, run migrations, and seed tags.
#[tracing::instrument]
pub async fn init_pool(data_dir: &Path) -> Result<SqlitePool, sqlx::Error> {
    let db_path = data_dir.join("db.sqlite");
    tracing::info!(db_path = %db_path.display(), "connecting to database");

    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;
    migrate(&pool).await?;
    seed_tags(&pool).await?;
    tracing::info!("database initialized");
    Ok(pool)
}

/// Initialize an in-memory SQLite pool for tests.
#[tracing::instrument]
pub async fn init_pool_in_memory() -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    migrate(&pool).await?;
    seed_tags(&pool).await?;
    Ok(pool)
}

/// Run the schema migration (CREATE TABLE IF NOT EXISTS).
#[tracing::instrument(skip(pool))]
async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(MIGRATE_SQL)
        .execute(pool)
        .await?;
    tracing::debug!("migration executed");
    Ok(())
}

/// No static seeding required — tags are created on-demand by `ensure_tag`.
async fn seed_tags(_pool: &SqlitePool) -> Result<(), sqlx::Error> {
    Ok(())
}

/// Ensure a tag with the given kind/value exists, creating it if necessary.
///
/// Returns the tag's database row ID.
#[tracing::instrument(skip(pool))]
pub async fn ensure_tag(pool: &SqlitePool, kind: &str, value: &str, label: &str) -> Result<i64, sqlx::Error> {
    let id: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM tags WHERE kind = ?1 AND value = ?2"
    )
    .bind(kind)
    .bind(value)
    .fetch_optional(pool)
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
    .fetch_one(pool)
    .await?;
    tracing::info!(tag_id = id, kind, value, "created new tag");

    Ok(id)
}

/// Link a message to a tag, silently ignoring duplicate associations.
#[tracing::instrument(skip(pool))]
pub async fn link_message_tag(pool: &SqlitePool, message_id: &str, tag_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR IGNORE INTO message_tags (message_id, tag_id) VALUES (?1, ?2)"
    )
    .bind(message_id)
    .bind(tag_id)
    .execute(pool)
    .await?;
    tracing::debug!(message_id, tag_id, "linked message to tag");
    Ok(())
}

/// List all tags with the count of associated messages.
#[tracing::instrument(skip(pool))]
pub async fn list_tags(pool: &SqlitePool) -> Result<Vec<Tag>, sqlx::Error> {
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
        "#
    )
    .fetch_all(pool)
    .await?;
    tracing::debug!(count = tags.len(), "listed tags");
    Ok(tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_migrate_creates_schema() {
        let pool = init_pool_in_memory().await.unwrap();
        let tables: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name"
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert!(tables.contains(&"messages".to_string()));
        assert!(tables.contains(&"attachments".to_string()));
        assert!(tables.contains(&"tags".to_string()));
        assert!(tables.contains(&"message_tags".to_string()));
    }

    #[tokio::test]
    async fn test_seed_tags_idempotent() {
        let pool = init_pool_in_memory().await.unwrap();
        seed_tags(&pool).await.unwrap();
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tags")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_ensure_tag_idempotent() {
        let pool = init_pool_in_memory().await.unwrap();
        let id1 = ensure_tag(&pool, "recipient_address", "a@b.com", "To: a@b.com").await.unwrap();
        let id2 = ensure_tag(&pool, "recipient_address", "a@b.com", "To: a@b.com").await.unwrap();
        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn test_link_message_tag() {
        let pool = init_pool_in_memory().await.unwrap();
        sqlx::query("INSERT INTO messages (id, from_address, to_address, raw_path) VALUES (?1, ?2, ?3, ?4)")
            .bind("msg-1")
            .bind("from@example.com")
            .bind("to@example.com")
            .bind("/tmp/raw.msg")
            .execute(&pool)
            .await
            .unwrap();

        let tag_id = ensure_tag(&pool, "recipient_address", "to@example.com", "To: to@example.com").await.unwrap();
        link_message_tag(&pool, "msg-1", tag_id).await.unwrap();

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM message_tags")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_list_tags_with_counts() {
        let pool = init_pool_in_memory().await.unwrap();
        let tag_id = ensure_tag(&pool, "recipient_address", "to@example.com", "To: to@example.com").await.unwrap();

        sqlx::query("INSERT INTO messages (id, from_address, to_address, raw_path) VALUES (?1, ?2, ?3, ?4)")
            .bind("msg-1")
            .bind("from@example.com")
            .bind("to@example.com")
            .bind("/tmp/raw.msg")
            .execute(&pool)
            .await
            .unwrap();

        link_message_tag(&pool, "msg-1", tag_id).await.unwrap();

        let tags = list_tags(&pool).await.unwrap();
        let tag = tags.iter().find(|t| t.value == "to@example.com").unwrap();
        assert_eq!(tag.message_count, Some(1));
    }
}
