use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;

use crate::error::AppError;
use crate::models::{ErrorResponse, MessageDetailResponse, MessageListResponse};
use crate::repository::ListMessagesQuery;
use crate::routes::AppState;

/// Query parameters for listing messages.
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    page: Option<u32>,
    limit: Option<u32>,
    tag: Option<i64>,
}

/// List messages with optional tag filtering and pagination.
#[utoipa::path(
    get,
    path = "/api/messages",
    operation_id = "listMessages",
    params(
        ("page" = Option<u32>, Query, description = "1-based page number"),
        ("limit" = Option<u32>, Query, description = "Page size between 1 and 100"),
        ("tag" = Option<i64>, Query, description = "Filter by tag id")
    ),
    responses(
        (status = 200, description = "Paginated message list", body = MessageListResponse),
        (status = 500, description = "Repository failure", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state))]
pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<MessageListResponse>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) as i64 * limit as i64;

    let page_data = state
        .repo
        .list_messages(ListMessagesQuery {
            tag_id: query.tag,
            limit: limit as i64,
            offset,
        })
        .await?;

    tracing::debug!(
        page,
        limit,
        total = page_data.total,
        item_count = page_data.items.len(),
        "listed messages"
    );
    Ok(Json(MessageListResponse {
        items: page_data.items,
        total: page_data.total,
        page,
        limit,
    }))
}

/// Retrieve the full details of a single message, including its attachments.
#[utoipa::path(
    get,
    path = "/api/messages/{id}",
    operation_id = "getMessageDetail",
    params(("id" = String, Path, description = "Message id")),
    responses(
        (status = 200, description = "Full message detail", body = MessageDetailResponse),
        (status = 404, description = "Message not found", body = ErrorResponse),
        (status = 500, description = "Repository failure", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state))]
pub async fn detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<MessageDetailResponse>, AppError> {
    let message = state
        .repo
        .get_message(&id)
        .await?
        .ok_or(AppError::NotFound)?;

    tracing::debug!(
        message_id = %id,
        attachment_count = message.attachments.len(),
        "retrieved message detail"
    );
    Ok(Json(MessageDetailResponse {
        message: message.message,
        attachments: message.attachments,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::repository::{InboundMessageRecord, InboundTagRecord, sqlx::SqlxRepository};
    use crate::services::inbound::InboundMessageService;

    async fn setup_state() -> AppState {
        let repo = Arc::new(SqlxRepository::init_pool_in_memory().await.unwrap());
        AppState {
            inbound_service: Arc::new(InboundMessageService::new(
                repo.clone(),
                std::path::PathBuf::from("/tmp"),
            )),
            repo,
        }
    }

    async fn insert_messages(repo: &dyn crate::repository::Repository, n: usize) {
        for i in 0..n {
            repo.ingest_message(InboundMessageRecord {
                id: format!("msg-{i}"),
                from_address: "from@example.com".to_string(),
                to_address: "to@example.com".to_string(),
                subject: Some(format!("Subject {i}")),
                date: None,
                message_id: Some(format!("<msg-{i}>")),
                raw_path: format!("/tmp/{i}.eml"),
                body_text: None,
                body_html: None,
                attachments: Vec::new(),
                tags: Vec::new(),
            })
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

        state
            .repo
            .ingest_message(InboundMessageRecord {
                id: "msg-tagged".to_string(),
                from_address: "from@example.com".to_string(),
                to_address: "to@example.com".to_string(),
                subject: Some("Tagged".to_string()),
                date: None,
                message_id: Some("<msg-tagged>".to_string()),
                raw_path: "/tmp/tagged.eml".to_string(),
                body_text: None,
                body_html: None,
                attachments: Vec::new(),
                tags: vec![InboundTagRecord {
                    kind: "recipient_address".to_string(),
                    value: "to@example.com".to_string(),
                    label: "To: to@example.com".to_string(),
                    source: "system".to_string(),
                }],
            })
            .await
            .unwrap();

        let query = ListQuery {
            page: None,
            limit: None,
            tag: Some(1),
        };
        let res = list(State(state), Query(query)).await.unwrap();
        assert_eq!(res.total, 1);
        assert_eq!(res.items[0].id, "msg-tagged");
    }

    #[tokio::test]
    async fn test_get_detail_404() {
        let state = setup_state().await;
        let res = detail(State(state), Path("missing".to_string())).await;
        assert!(matches!(res, Err(AppError::NotFound)));
    }
}
