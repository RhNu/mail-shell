use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::error::AppError;
use crate::models::{
    AttachmentMeta, MessageDetail, MessageDetailResponse, MessageSummary, Paginated,
};
use crate::routes::AppState;

/// Query parameters for listing messages.
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    page: Option<u32>,
    limit: Option<u32>,
    tag: Option<i64>,
}

/// List messages with optional tag filtering and pagination.
#[tracing::instrument(skip(state))]
pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Paginated<MessageSummary>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) as i64 * limit as i64;

    let total: i64 = if let Some(tag_id) = query.tag {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM message_tags WHERE tag_id = ?1",
        )
        .bind(tag_id)
        .fetch_one(&state.pool)
        .await?
    } else {
        sqlx::query_scalar("SELECT COUNT(*) FROM messages")
            .fetch_one(&state.pool)
            .await?
    };

    let items = if let Some(tag_id) = query.tag {
        sqlx::query_as::<_, MessageSummary>(
            "SELECT m.* FROM messages m
             JOIN message_tags mt ON mt.message_id = m.id
             WHERE mt.tag_id = ?1
             ORDER BY m.created_at DESC
             LIMIT ?2 OFFSET ?3",
        )
        .bind(tag_id)
        .bind(limit as i64)
        .bind(offset)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, MessageSummary>(
            "SELECT * FROM messages ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
        )
        .bind(limit as i64)
        .bind(offset)
        .fetch_all(&state.pool)
        .await?
    };

    tracing::debug!(page, limit, total, item_count = items.len(), "listed messages");
    Ok(Json(Paginated {
        items,
        total,
        page,
        limit,
    }))
}

/// Retrieve the full details of a single message, including its attachments.
#[tracing::instrument(skip(state))]
pub async fn detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<MessageDetailResponse>, AppError> {
    let message = sqlx::query_as::<_, MessageDetail>("SELECT * FROM messages WHERE id = ?1")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let attachments =
        sqlx::query_as::<_, AttachmentMeta>(
            "SELECT id, message_id, filename, content_type, size FROM attachments WHERE message_id = ?1",
        )
        .bind(&id)
        .fetch_all(&state.pool)
        .await?;

    tracing::debug!(message_id = %id, attachment_count = attachments.len(), "retrieved message detail");
    Ok(Json(MessageDetailResponse {
        message,
        attachments,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    async fn setup_state() -> AppState {
        let pool = db::init_pool_in_memory().await.unwrap();
        AppState {
            pool: pool.clone(),
            data_dir: std::path::PathBuf::from("/tmp"),
        }
    }

    async fn insert_messages(pool: &sqlx::SqlitePool, n: usize) {
        for i in 0..n {
            sqlx::query(
                "INSERT INTO messages (id, from_address, to_address, subject, raw_path)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .bind(format!("msg-{i}"))
            .bind("from@example.com")
            .bind("to@example.com")
            .bind(format!("Subject {i}"))
            .bind(format!("/tmp/{i}.eml"))
            .execute(pool)
            .await
            .unwrap();
        }
    }

    #[tokio::test]
    async fn test_list_pagination() {
        let state = setup_state().await;
        insert_messages(&state.pool, 5).await;

        let query = ListQuery {
            page: Some(1),
            limit: Some(2),
            tag: None,
        };
        let res = list(State(state), Query(query)).await.unwrap();
        assert_eq!(res.total, 5);
        assert_eq!(res.items.len(), 2);
    }

    #[tokio::test]
    async fn test_list_filter_by_tag() {
        let state = setup_state().await;
        insert_messages(&state.pool, 3).await;

        let tag_id = db::ensure_tag(&state.pool, "recipient_address", "to@example.com", "To: to@example.com")
            .await
            .unwrap();
        db::link_message_tag(&state.pool, "msg-0", tag_id).await.unwrap();

        let query = ListQuery {
            page: None,
            limit: None,
            tag: Some(tag_id),
        };
        let res = list(State(state), Query(query)).await.unwrap();
        assert_eq!(res.total, 1);
        assert_eq!(res.items[0].id, "msg-0");
    }

    #[tokio::test]
    async fn test_get_detail_404() {
        let state = setup_state().await;
        let res = detail(State(state), Path("missing".to_string())).await;
        assert!(matches!(res, Err(AppError::NotFound)));
    }
}
