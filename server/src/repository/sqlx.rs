use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{
    QueryBuilder, Sqlite, SqlitePool, Transaction,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::path::Path;

use crate::mime_parser::{ParsedMailSnapshotV1, SNAPSHOT_VERSION};
use crate::models::{
    AttachmentDownloadMeta, AttachmentMeta, HeaderEntry, Mailbox, MessageDetail, MessageRawMeta,
    MessageSummary, Tag,
};
use crate::repository::{
    DeletedMessageFiles, InboundMessageRecord, ListMessagesQuery, MessagePage, MessageRecord,
    Repository, RepositoryError,
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
    mailbox: Mailbox,
    read_at: Option<DateTime<Utc>>,
    is_starred: bool,
    trashed_at: Option<DateTime<Utc>>,
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
        let backup_path = data_dir.join("index.pre-v3.sqlite");
        if db_path.exists() && !backup_path.exists() {
            std::fs::copy(&db_path, &backup_path).map_err(sqlx::Error::Io)?;
            tracing::info!(path = %backup_path.display(), "created pre-v3 database backup");
        }
        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new().connect_with(options).await?;
        Self::migrate(&pool).await?;
        let repo = Self::new(pool);
        repo.rebuild_derived_indexes().await?;
        Ok(repo)
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
        let repo = Self::new(pool);
        repo.rebuild_derived_indexes().await?;
        Ok(repo)
    }

    #[tracing::instrument(skip(pool))]
    async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        MIGRATOR.run(pool).await?;
        Ok(())
    }

    async fn rebuild_derived_indexes(&self) -> Result<(), sqlx::Error> {
        let version: Option<String> = sqlx::query_scalar(
            "SELECT value FROM app_metadata WHERE key = 'derived_index_version'",
        )
        .fetch_optional(&self.pool)
        .await?;
        if version.as_deref() == Some("1") {
            return Ok(());
        }

        let rows = sqlx::query_as::<_, StoredSnapshot>(
            "SELECT id, snapshot_version, parsed_snapshot FROM messages ORDER BY created_at, id",
        )
        .fetch_all(&self.pool)
        .await?;
        let envelopes: Vec<(String, String)> =
            sqlx::query_as("SELECT id, envelope_to FROM messages ORDER BY created_at, id")
                .fetch_all(&self.pool)
                .await?;
        let envelope_by_id = envelopes
            .into_iter()
            .collect::<std::collections::HashMap<_, _>>();
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM message_addresses")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM message_fts")
            .execute(&mut *tx)
            .await?;
        for row in rows {
            let snapshot =
                Self::decode_snapshot(&row.id, row.snapshot_version, &row.parsed_snapshot)
                    .map_err(|error| sqlx::Error::Protocol(error.to_string()))?;
            let envelope = envelope_by_id
                .get(&row.id)
                .map(String::as_str)
                .unwrap_or_default();
            insert_snapshot_values(&mut tx, &row.id, envelope, &snapshot).await?;
        }
        sqlx::query(
            "INSERT INTO app_metadata(key, value) VALUES ('derived_index_version', '1')
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
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
    async fn find_message_by_fingerprint(
        &self,
        fingerprint: &str,
    ) -> Result<Option<String>, RepositoryError> {
        Ok(
            sqlx::query_scalar("SELECT id FROM messages WHERE ingest_fingerprint = ?1")
                .bind(fingerprint)
                .fetch_optional(&self.pool)
                .await?,
        )
    }

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
            "INSERT INTO messages (id, message_id, subject, from_name, from_address, to_name, to_address, envelope_to, date, raw_path, ingest_fingerprint, snapshot_version, parsed_snapshot, mailbox, read_at, is_starred, trashed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
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
        .bind(&record.ingest_fingerprint)
        .bind(SNAPSHOT_VERSION)
        .bind(&parsed_snapshot)
        .bind(record.initial_state.mailbox.unwrap_or_default().as_str())
        .bind(record.initial_state.read.filter(|read| *read).map(|_| Utc::now()))
        .bind(record.initial_state.starred.unwrap_or(false))
        .bind(record.initial_state.trashed.filter(|trashed| *trashed).map(|_| Utc::now()))
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

        for label_id in &record.label_ids {
            sqlx::query(
                "INSERT OR IGNORE INTO message_labels(message_id, label_id, origin)
                 VALUES (?1, ?2, 'rule')",
            )
            .bind(&record.id)
            .bind(label_id)
            .execute(&mut *tx)
            .await?;
        }

        insert_snapshot_indexes(&mut tx, &record).await?;

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
        let search = query.search.as_deref().and_then(fts_query);
        let mut count = QueryBuilder::<Sqlite>::new("SELECT COUNT(DISTINCT m.id) FROM messages m");
        append_list_joins(&mut count, query.tag_id, query.label_id, search.as_deref());
        append_list_filters(&mut count, &query, search.as_deref());
        let total: i64 = count.build_query_scalar().fetch_one(&self.pool).await?;

        let mut items = QueryBuilder::<Sqlite>::new(
            "SELECT DISTINCT m.id, m.from_name, m.from_address, m.to_name, m.to_address, \
             m.envelope_to, m.subject, m.date, m.message_id, m.mailbox, \
             (m.read_at IS NOT NULL) AS is_read, m.is_starred, \
             (SELECT COUNT(*) FROM attachments a WHERE a.message_id = m.id) AS attachment_count, \
             m.created_at FROM messages m",
        );
        append_list_joins(&mut items, query.tag_id, query.label_id, search.as_deref());
        append_list_filters(&mut items, &query, search.as_deref());
        items
            .push(" ORDER BY m.created_at DESC, m.id DESC LIMIT ")
            .push_bind(query.limit)
            .push(" OFFSET ")
            .push_bind(query.offset);
        let mut items = items
            .build_query_as::<MessageSummary>()
            .fetch_all(&self.pool)
            .await?;
        for item in &mut items {
            item.labels = self.get_message_labels(&item.id).await?;
        }

        tracing::debug!(
            total,
            returned_count = items.len(),
            tag_filter = ?query.tag_id,
            mailbox = %query.mailbox,
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
                    envelope_to, date, mailbox, read_at, is_starred, trashed_at,
                    snapshot_version, parsed_snapshot, created_at
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
        let labels = self.get_message_labels(&stored.id).await?;
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
            mailbox: stored.mailbox,
            is_read: stored.read_at.is_some(),
            is_starred: stored.is_starred,
            trashed_at: stored.trashed_at,
            labels,
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
    async fn update_message_mailbox(
        &self,
        id: &str,
        mailbox: Mailbox,
    ) -> Result<bool, RepositoryError> {
        let result = sqlx::query("UPDATE messages SET mailbox = ?1 WHERE id = ?2")
            .bind(mailbox.as_str())
            .bind(id)
            .execute(&self.pool)
            .await?;

        let updated = result.rows_affected() > 0;
        tracing::debug!(message_id = %id, mailbox = %mailbox, updated, "updated message mailbox");
        Ok(updated)
    }

    async fn update_message_state(
        &self,
        ids: &[String],
        state: &crate::models::MessageStateUpdateRequest,
    ) -> Result<u64, RepositoryError> {
        if ids.is_empty() {
            return Ok(0);
        }
        let mut query = QueryBuilder::<Sqlite>::new("UPDATE messages SET ");
        let mut separated = query.separated(", ");
        if let Some(mailbox) = state.mailbox {
            separated
                .push("mailbox = ")
                .push_bind_unseparated(mailbox.as_str());
        }
        if let Some(read) = state.read {
            if read {
                separated.push("read_at = COALESCE(read_at, CURRENT_TIMESTAMP)");
            } else {
                separated.push("read_at = NULL");
            }
        }
        if let Some(starred) = state.starred {
            separated
                .push("is_starred = ")
                .push_bind_unseparated(starred);
        }
        if let Some(trashed) = state.trashed {
            if trashed {
                separated.push("trashed_at = COALESCE(trashed_at, CURRENT_TIMESTAMP)");
            } else {
                separated.push("trashed_at = NULL");
            }
        }
        if state.mailbox.is_none()
            && state.read.is_none()
            && state.starred.is_none()
            && state.trashed.is_none()
        {
            return Ok(0);
        }
        drop(separated);
        query.push(" WHERE id IN (");
        let mut ids_builder = query.separated(", ");
        for id in ids {
            ids_builder.push_bind(id);
        }
        ids_builder.push_unseparated(")");
        Ok(query.build().execute(&self.pool).await?.rows_affected())
    }

    #[tracing::instrument(skip(self))]
    async fn delete_message(
        &self,
        id: &str,
    ) -> Result<Option<DeletedMessageFiles>, RepositoryError> {
        let mut tx = self.pool.begin().await?;

        let raw_path: Option<String> =
            sqlx::query_scalar("SELECT raw_path FROM messages WHERE id = ?1")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?;

        let Some(raw_path) = raw_path else {
            return Ok(None);
        };

        let attachment_paths: Vec<String> =
            sqlx::query_scalar("SELECT path FROM attachments WHERE message_id = ?1 ORDER BY id")
                .bind(id)
                .fetch_all(&mut *tx)
                .await?;

        sqlx::query("DELETE FROM message_tags WHERE message_id = ?1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM attachments WHERE message_id = ?1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM messages WHERE id = ?1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM message_fts WHERE message_id = ?1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        tracing::debug!(
            message_id = %id,
            attachment_count = attachment_paths.len(),
            "deleted message"
        );
        Ok(Some(DeletedMessageFiles {
            raw_path,
            attachment_paths,
        }))
    }

    async fn list_trashed_before(
        &self,
        cutoff: Option<DateTime<Utc>>,
    ) -> Result<Vec<String>, RepositoryError> {
        let ids = if let Some(cutoff) = cutoff {
            sqlx::query_scalar(
                "SELECT id FROM messages WHERE trashed_at IS NOT NULL AND trashed_at <= ?1",
            )
            .bind(cutoff)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_scalar("SELECT id FROM messages WHERE trashed_at IS NOT NULL")
                .fetch_all(&self.pool)
                .await?
        };
        Ok(ids)
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
                COUNT(m.id) AS message_count
            FROM tags t
            LEFT JOIN message_tags mt ON mt.tag_id = t.id
            LEFT JOIN messages m ON m.id = mt.message_id AND m.mailbox = ?1
            GROUP BY t.id
            HAVING COUNT(m.id) > 0
            ORDER BY t.kind, t.value
            "#,
        )
        .bind(Mailbox::Inbox.as_str())
        .fetch_all(&self.pool)
        .await?;
        tracing::debug!(tag_count = tags.len(), "listed tags");
        Ok(tags)
    }

    async fn list_facets(
        &self,
        kind: Option<&str>,
    ) -> Result<Vec<crate::models::FacetValue>, RepositoryError> {
        let role = match kind.unwrap_or("recipient") {
            "sender" => "from",
            "recipient" => "envelope_to",
            "domain" => "domain",
            _ => "envelope_to",
        };
        let rows = if role == "domain" {
            sqlx::query_as::<_, FacetRow>(
                "SELECT 'domain' AS kind, ma.domain AS value, ma.domain AS label, COUNT(DISTINCT m.id) AS message_count
                 FROM message_addresses ma JOIN messages m ON m.id = ma.message_id
                 WHERE m.trashed_at IS NULL AND ma.domain IS NOT NULL
                 GROUP BY ma.domain ORDER BY message_count DESC, value LIMIT 200",
            ).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as::<_, FacetRow>(
                "SELECT ?1 AS kind, ma.address AS value, COALESCE(ma.name, ma.address) AS label, COUNT(DISTINCT m.id) AS message_count
                 FROM message_addresses ma JOIN messages m ON m.id = ma.message_id
                 WHERE m.trashed_at IS NULL AND ma.role = ?2
                 GROUP BY ma.address ORDER BY message_count DESC, value LIMIT 200",
            ).bind(kind.unwrap_or("recipient")).bind(role).fetch_all(&self.pool).await?
        };
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn list_labels(&self) -> Result<Vec<crate::models::Label>, RepositoryError> {
        Ok(sqlx::query_as(
            "SELECT l.id, l.name, l.color, l.sort_order, COUNT(m.id) AS message_count
             FROM labels l
             LEFT JOIN message_labels ml ON ml.label_id = l.id
             LEFT JOIN messages m ON m.id = ml.message_id AND m.trashed_at IS NULL
             GROUP BY l.id ORDER BY l.sort_order, l.name",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn create_label(
        &self,
        request: &crate::models::LabelWriteRequest,
    ) -> Result<crate::models::Label, RepositoryError> {
        let name = request.name.trim();
        if name.is_empty() {
            return Err(RepositoryError::InvalidClassification(
                "label name is empty".into(),
            ));
        }
        let mut label: crate::models::Label = sqlx::query_as(
            "INSERT INTO labels(name, color) VALUES (?1, ?2)
             RETURNING id, name, color, sort_order, NULL AS message_count",
        )
        .bind(name)
        .bind(&request.color)
        .fetch_one(&self.pool)
        .await?;
        label.message_count = Some(0);
        Ok(label)
    }

    async fn update_label(
        &self,
        id: i64,
        request: &crate::models::LabelWriteRequest,
    ) -> Result<bool, RepositoryError> {
        let name = request.name.trim();
        if name.is_empty() {
            return Err(RepositoryError::InvalidClassification(
                "label name is empty".into(),
            ));
        }
        Ok(
            sqlx::query("UPDATE labels SET name = ?1, color = ?2 WHERE id = ?3")
                .bind(name)
                .bind(&request.color)
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected()
                > 0,
        )
    }

    async fn delete_label(&self, id: i64) -> Result<bool, RepositoryError> {
        Ok(sqlx::query("DELETE FROM labels WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected()
            > 0)
    }

    async fn set_message_labels(
        &self,
        message_id: &str,
        label_ids: &[i64],
    ) -> Result<bool, RepositoryError> {
        let mut tx = self.pool.begin().await?;
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM messages WHERE id = ?1)")
                .bind(message_id)
                .fetch_one(&mut *tx)
                .await?;
        if !exists {
            return Ok(false);
        }
        sqlx::query("DELETE FROM message_labels WHERE message_id = ?1 AND origin = 'manual'")
            .bind(message_id)
            .execute(&mut *tx)
            .await?;
        for label_id in label_ids {
            sqlx::query(
                "INSERT OR IGNORE INTO message_labels(message_id, label_id, origin)
                 VALUES (?1, ?2, 'manual')",
            )
            .bind(message_id)
            .bind(label_id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(true)
    }

    async fn get_message_labels(
        &self,
        message_id: &str,
    ) -> Result<Vec<crate::models::MessageLabel>, RepositoryError> {
        Ok(sqlx::query_as(
            "SELECT l.id, l.name, l.color FROM labels l
             JOIN message_labels ml ON ml.label_id = l.id
             WHERE ml.message_id = ?1 ORDER BY l.sort_order, l.name",
        )
        .bind(message_id)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn list_rules(
        &self,
        enabled_only: bool,
    ) -> Result<Vec<crate::models::ClassificationRule>, RepositoryError> {
        let rows = sqlx::query_as::<_, StoredRule>(
            "SELECT id, name, enabled, priority, stop_processing, conditions_json, actions_json
             FROM rules WHERE (?1 = 0 OR enabled = 1) ORDER BY priority DESC, id",
        )
        .bind(enabled_only)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_rule).collect()
    }

    async fn create_rule(
        &self,
        request: &crate::models::RuleWriteRequest,
    ) -> Result<crate::models::ClassificationRule, RepositoryError> {
        let conditions = serde_json::to_string(&request.conditions)
            .map_err(|error| RepositoryError::InvalidClassification(error.to_string()))?;
        let actions = serde_json::to_string(&request.actions)
            .map_err(|error| RepositoryError::InvalidClassification(error.to_string()))?;
        let row = sqlx::query_as::<_, StoredRule>(
            "INSERT INTO rules(name, enabled, priority, stop_processing, conditions_json, actions_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             RETURNING id, name, enabled, priority, stop_processing, conditions_json, actions_json",
        )
        .bind(request.name.trim())
        .bind(request.enabled)
        .bind(request.priority)
        .bind(request.stop_processing)
        .bind(conditions)
        .bind(actions)
        .fetch_one(&self.pool)
        .await?;
        decode_rule(row)
    }

    async fn update_rule(
        &self,
        id: i64,
        request: &crate::models::RuleWriteRequest,
    ) -> Result<bool, RepositoryError> {
        let conditions = serde_json::to_string(&request.conditions)
            .map_err(|error| RepositoryError::InvalidClassification(error.to_string()))?;
        let actions = serde_json::to_string(&request.actions)
            .map_err(|error| RepositoryError::InvalidClassification(error.to_string()))?;
        Ok(sqlx::query(
            "UPDATE rules SET name=?1, enabled=?2, priority=?3, stop_processing=?4,
             conditions_json=?5, actions_json=?6, updated_at=CURRENT_TIMESTAMP WHERE id=?7",
        )
        .bind(request.name.trim())
        .bind(request.enabled)
        .bind(request.priority)
        .bind(request.stop_processing)
        .bind(conditions)
        .bind(actions)
        .bind(id)
        .execute(&self.pool)
        .await?
        .rows_affected()
            > 0)
    }

    async fn delete_rule(&self, id: i64) -> Result<bool, RepositoryError> {
        Ok(sqlx::query("DELETE FROM rules WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected()
            > 0)
    }

    async fn list_saved_views(&self) -> Result<Vec<crate::models::SavedView>, RepositoryError> {
        let rows = sqlx::query_as::<_, StoredSavedView>(
            "SELECT id, name, query_json, pinned, sort_order FROM saved_views
             ORDER BY pinned DESC, sort_order, name",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_saved_view).collect()
    }

    async fn create_saved_view(
        &self,
        request: &crate::models::SavedViewWriteRequest,
    ) -> Result<crate::models::SavedView, RepositoryError> {
        let query = serde_json::to_string(&request.query)
            .map_err(|error| RepositoryError::InvalidClassification(error.to_string()))?;
        let row = sqlx::query_as::<_, StoredSavedView>(
            "INSERT INTO saved_views(name, query_json, pinned, sort_order) VALUES (?1, ?2, ?3, ?4)
             RETURNING id, name, query_json, pinned, sort_order",
        )
        .bind(request.name.trim())
        .bind(query)
        .bind(request.pinned)
        .bind(request.sort_order)
        .fetch_one(&self.pool)
        .await?;
        decode_saved_view(row)
    }

    async fn update_saved_view(
        &self,
        id: i64,
        request: &crate::models::SavedViewWriteRequest,
    ) -> Result<bool, RepositoryError> {
        let query = serde_json::to_string(&request.query)
            .map_err(|error| RepositoryError::InvalidClassification(error.to_string()))?;
        Ok(sqlx::query(
            "UPDATE saved_views SET name=?1, query_json=?2, pinned=?3, sort_order=?4 WHERE id=?5",
        )
        .bind(request.name.trim())
        .bind(query)
        .bind(request.pinned)
        .bind(request.sort_order)
        .bind(id)
        .execute(&self.pool)
        .await?
        .rows_affected()
            > 0)
    }

    async fn delete_saved_view(&self, id: i64) -> Result<bool, RepositoryError> {
        Ok(sqlx::query("DELETE FROM saved_views WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected()
            > 0)
    }
}

#[derive(sqlx::FromRow)]
struct StoredRule {
    id: i64,
    name: String,
    enabled: bool,
    priority: i64,
    stop_processing: bool,
    conditions_json: String,
    actions_json: String,
}

fn decode_rule(row: StoredRule) -> Result<crate::models::ClassificationRule, RepositoryError> {
    Ok(crate::models::ClassificationRule {
        id: row.id,
        name: row.name,
        enabled: row.enabled,
        priority: row.priority,
        stop_processing: row.stop_processing,
        conditions: serde_json::from_str(&row.conditions_json)
            .map_err(|error| RepositoryError::InvalidClassification(error.to_string()))?,
        actions: serde_json::from_str(&row.actions_json)
            .map_err(|error| RepositoryError::InvalidClassification(error.to_string()))?,
    })
}

#[derive(sqlx::FromRow)]
struct StoredSavedView {
    id: i64,
    name: String,
    query_json: String,
    pinned: bool,
    sort_order: i64,
}

fn decode_saved_view(row: StoredSavedView) -> Result<crate::models::SavedView, RepositoryError> {
    Ok(crate::models::SavedView {
        id: row.id,
        name: row.name,
        query: serde_json::from_str(&row.query_json)
            .map_err(|error| RepositoryError::InvalidClassification(error.to_string()))?,
        pinned: row.pinned,
        sort_order: row.sort_order,
    })
}

#[derive(sqlx::FromRow)]
struct FacetRow {
    kind: String,
    value: String,
    label: String,
    message_count: i64,
}

impl From<FacetRow> for crate::models::FacetValue {
    fn from(value: FacetRow) -> Self {
        Self {
            kind: value.kind,
            value: value.value,
            label: value.label,
            message_count: value.message_count,
        }
    }
}

fn fts_query(value: &str) -> Option<String> {
    let terms = value
        .split_whitespace()
        .filter(|term| !term.is_empty())
        .map(|term| format!("\"{}\"*", term.replace('"', "\"\"")))
        .collect::<Vec<_>>();
    (!terms.is_empty()).then(|| terms.join(" AND "))
}

fn append_list_joins(
    builder: &mut QueryBuilder<'_, Sqlite>,
    tag_id: Option<i64>,
    label_id: Option<i64>,
    search: Option<&str>,
) {
    if tag_id.is_some() {
        builder.push(" JOIN message_tags mt ON mt.message_id = m.id");
    }
    if label_id.is_some() {
        builder.push(" JOIN message_labels ml_filter ON ml_filter.message_id = m.id");
    }
    if search.is_some() {
        builder.push(" JOIN message_fts ON message_fts.message_id = m.id");
    }
}

fn append_list_filters<'a>(
    builder: &mut QueryBuilder<'a, Sqlite>,
    query: &'a ListMessagesQuery,
    search: Option<&'a str>,
) {
    if query.trashed {
        builder.push(" WHERE m.trashed_at IS NOT NULL");
    } else {
        builder
            .push(" WHERE m.mailbox = ")
            .push_bind(query.mailbox.as_str())
            .push(" AND m.trashed_at IS NULL");
    }
    if let Some(read) = query.read {
        if read {
            builder.push(" AND m.read_at IS NOT NULL");
        } else {
            builder.push(" AND m.read_at IS NULL");
        }
    }
    if let Some(starred) = query.starred {
        builder.push(" AND m.is_starred = ").push_bind(starred);
    }
    if let Some(tag_id) = query.tag_id {
        builder.push(" AND mt.tag_id = ").push_bind(tag_id);
    }
    if let Some(label_id) = query.label_id {
        builder
            .push(" AND ml_filter.label_id = ")
            .push_bind(label_id);
    }
    if let Some(search) = search {
        builder.push(" AND message_fts MATCH ").push_bind(search);
    }
}

async fn insert_snapshot_indexes(
    tx: &mut Transaction<'_, Sqlite>,
    record: &InboundMessageRecord,
) -> Result<(), sqlx::Error> {
    insert_snapshot_values(tx, &record.id, &record.envelope_to, &record.snapshot).await
}

async fn insert_snapshot_values(
    tx: &mut Transaction<'_, Sqlite>,
    message_id: &str,
    envelope_to: &str,
    snapshot: &ParsedMailSnapshotV1,
) -> Result<(), sqlx::Error> {
    for (role, addresses) in [
        ("from", &snapshot.from),
        ("to", &snapshot.to),
        ("cc", &snapshot.cc),
        ("bcc", &snapshot.bcc),
        ("reply_to", &snapshot.reply_to),
        ("sender", &snapshot.sender),
    ] {
        for (position, address) in addresses.iter().enumerate() {
            let normalized = address.email().trim().to_lowercase();
            if normalized.is_empty() {
                continue;
            }
            let domain = normalized.rsplit_once('@').map(|(_, domain)| domain);
            sqlx::query(
                "INSERT INTO message_addresses(message_id, role, position, name, address, domain)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(message_id)
            .bind(role)
            .bind(position as i64)
            .bind(&address.name)
            .bind(&normalized)
            .bind(domain)
            .execute(&mut **tx)
            .await?;
        }
    }
    let envelope = envelope_to.trim().to_lowercase();
    if !envelope.is_empty() {
        let domain = envelope.rsplit_once('@').map(|(_, domain)| domain);
        sqlx::query(
            "INSERT INTO message_addresses(message_id, role, position, address, domain)
             VALUES (?1, 'envelope_to', 0, ?2, ?3)",
        )
        .bind(message_id)
        .bind(&envelope)
        .bind(domain)
        .execute(&mut **tx)
        .await?;
    }

    let sender = snapshot
        .from
        .iter()
        .map(|address| address.display())
        .collect::<Vec<_>>()
        .join(" ");
    let recipients = snapshot
        .to
        .iter()
        .chain(snapshot.cc.iter())
        .chain(snapshot.bcc.iter())
        .map(|address| address.display())
        .chain(std::iter::once(envelope.clone()))
        .collect::<Vec<_>>()
        .join(" ");
    let body = [
        snapshot.body_text().unwrap_or_default(),
        snapshot.body_html().unwrap_or_default(),
    ]
    .join(" ");
    sqlx::query(
        "INSERT INTO message_fts(message_id, subject, sender, recipients, body)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(message_id)
    .bind(&snapshot.subject)
    .bind(sender)
    .bind(recipients)
    .bind(body)
    .execute(&mut **tx)
    .await?;
    Ok(())
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
            ingest_fingerprint: None,
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
            label_ids: Vec::new(),
            initial_state: Default::default(),
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
        assert!(columns.contains(&"mailbox".to_string()));
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
                label_id: None,
                mailbox: Mailbox::Inbox,
                search: None,
                read: None,
                starred: None,
                trashed: false,
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
    async fn mailbox_filtering_keeps_archive_out_of_inbox_and_tag_counts() {
        let repo = SqlxRepository::init_pool_in_memory().await.unwrap();
        repo.ingest_message(inbound_record("msg-inbox", "att-inbox", "<msg-inbox>"))
            .await
            .unwrap();
        repo.ingest_message(inbound_record(
            "msg-archive",
            "att-archive",
            "<msg-archive>",
        ))
        .await
        .unwrap();

        let archived = repo
            .update_message_mailbox("msg-archive", Mailbox::Archive)
            .await
            .unwrap();
        assert!(archived);

        let inbox_page = repo
            .list_messages(ListMessagesQuery {
                tag_id: None,
                label_id: None,
                mailbox: Mailbox::Inbox,
                search: None,
                read: None,
                starred: None,
                trashed: false,
                limit: 20,
                offset: 0,
            })
            .await
            .unwrap();
        assert_eq!(inbox_page.total, 1);
        assert_eq!(inbox_page.items[0].id, "msg-inbox");
        assert_eq!(inbox_page.items[0].mailbox, Mailbox::Inbox);

        let archive_page = repo
            .list_messages(ListMessagesQuery {
                tag_id: None,
                label_id: None,
                mailbox: Mailbox::Archive,
                search: None,
                read: None,
                starred: None,
                trashed: false,
                limit: 20,
                offset: 0,
            })
            .await
            .unwrap();
        assert_eq!(archive_page.total, 1);
        assert_eq!(archive_page.items[0].id, "msg-archive");
        assert_eq!(archive_page.items[0].mailbox, Mailbox::Archive);

        let tagged_inbox = repo
            .list_messages(ListMessagesQuery {
                tag_id: Some(1),
                label_id: None,
                mailbox: Mailbox::Inbox,
                search: None,
                read: None,
                starred: None,
                trashed: false,
                limit: 20,
                offset: 0,
            })
            .await
            .unwrap();
        assert_eq!(tagged_inbox.total, 1);
        assert_eq!(tagged_inbox.items[0].id, "msg-inbox");

        let tags = repo.list_tags().await.unwrap();
        let recipient_tag = tags
            .iter()
            .find(|tag| tag.kind == "recipient_address")
            .unwrap();
        assert_eq!(recipient_tag.message_count, Some(1));
    }

    #[tokio::test]
    async fn update_message_mailbox_reports_missing_messages() {
        let repo = SqlxRepository::init_pool_in_memory().await.unwrap();

        let updated = repo
            .update_message_mailbox("missing", Mailbox::Archive)
            .await
            .unwrap();

        assert!(!updated);
    }

    #[tokio::test]
    async fn delete_message_removes_database_graph_and_returns_blob_paths() {
        let repo = SqlxRepository::init_pool_in_memory().await.unwrap();
        repo.ingest_message(inbound_record("msg-1", "att-1", "<msg-1>"))
            .await
            .unwrap();

        let deleted = repo.delete_message("msg-1").await.unwrap().unwrap();

        assert_eq!(deleted.raw_path, "/tmp/msg-1.eml");
        assert_eq!(deleted.attachment_paths, vec!["/tmp/att-1.txt"]);
        assert!(repo.get_message("msg-1").await.unwrap().is_none());
        assert!(repo.get_message_raw("msg-1").await.unwrap().is_none());
        assert!(
            repo.get_attachment_download("att-1")
                .await
                .unwrap()
                .is_none()
        );

        let page = repo
            .list_messages(ListMessagesQuery {
                tag_id: None,
                label_id: None,
                mailbox: Mailbox::Inbox,
                search: None,
                read: None,
                starred: None,
                trashed: false,
                limit: 20,
                offset: 0,
            })
            .await
            .unwrap();
        assert_eq!(page.total, 0);
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
