use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::error::AppError;
use crate::models::{MessageDetailResponse, MessageSummary, Paginated};
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

    let total = state.repo.count_messages(query.tag).await?;
    let items = state.repo.list_messages(query.tag, limit as i64, offset).await?;

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
    let message = state
        .repo
        .get_message_detail(&id)
        .await?
        .ok_or(AppError::NotFound)?;

    let attachments = state.repo.list_attachments_by_message(&id).await?;

    tracing::debug!(message_id = %id, attachment_count = attachments.len(), "retrieved message detail");
    Ok(Json(MessageDetailResponse {
        message,
        attachments,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::repository::sqlx::SqlxRepository;

    async fn setup_state() -> AppState {
        let repo = SqlxRepository::init_pool_in_memory().await.unwrap();
        AppState {
            repo: Arc::new(repo),
            data_dir: std::path::PathBuf::from("/tmp"),
        }
    }

    async fn insert_messages(repo: &dyn crate::repository::Repository, n: usize) {
        for i in 0..n {
            repo.create_message(
                &format!("msg-{i}"),
                "from@example.com",
                "to@example.com",
                Some(&format!("Subject {i}")),
                None,
                None,
                &format!("/tmp/{i}.eml"),
                None,
                None,
            )
            .await
            .unwrap();
        }
    }

    #[tokio::test]
    async fn test_list_pagination() {
        let state = setup_state().await;
        insert_messages(&*state.repo, 5).await;

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
        insert_messages(&*state.repo, 3).await;

        let tag_id = state
            .repo
            .ensure_tag("recipient_address", "to@example.com", "To: to@example.com")
            .await
            .unwrap();
        state.repo.link_message_tag("msg-0", tag_id).await.unwrap();

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
