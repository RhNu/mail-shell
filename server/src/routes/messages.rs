use axum::{
    Json,
    extract::{Path, Query, State},
    http::header,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::error::AppError;
use crate::models::{
    BulkMessageStateUpdateRequest, ErrorResponse, Mailbox, MailboxUpdateRequest,
    MessageDetailResponse, MessageHeadersResponse, MessageListResponse, MessageStateUpdateRequest,
};
use crate::repository::ListMessagesQuery;
use crate::routes::AppState;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    page: Option<u32>,
    limit: Option<u32>,
    label: Option<i64>,
    mailbox: Option<Mailbox>,
    q: Option<String>,
    read: Option<bool>,
    starred: Option<bool>,
    trashed: Option<bool>,
}

#[utoipa::path(
    get,
    path = "/api/messages",
    operation_id = "listMessages",
    params(
        ("page" = Option<u32>, Query, description = "1-based page number"),
        ("limit" = Option<u32>, Query, description = "Page size between 1 and 100"),
        ("label" = Option<i64>, Query, description = "Filter by user label id"),
        ("mailbox" = Option<Mailbox>, Query, description = "Filter by mailbox; defaults to inbox"),
        ("q" = Option<String>, Query, description = "Full-text search"),
        ("read" = Option<bool>, Query, description = "Filter by read state"),
        ("starred" = Option<bool>, Query, description = "Filter by starred state"),
        ("trashed" = Option<bool>, Query, description = "Show trashed messages")
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
            label_id: query.label,
            mailbox: query.mailbox.unwrap_or_default(),
            search: query.q.filter(|value| !value.trim().is_empty()),
            read: query.read,
            starred: query.starred,
            trashed: query.trashed.unwrap_or(false),
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

#[utoipa::path(
    patch,
    path = "/api/messages/{id}/state",
    operation_id = "updateMessageState",
    params(("id" = String, Path, description = "Message id")),
    request_body = MessageStateUpdateRequest,
    responses(
        (status = 204, description = "Message state updated"),
        (status = 404, description = "Message not found", body = ErrorResponse)
    )
)]
pub async fn update_state(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<MessageStateUpdateRequest>,
) -> Result<axum::http::StatusCode, AppError> {
    let updated = state.repo.update_message_state(&[id], &request).await?;
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    patch,
    path = "/api/messages/bulk-state",
    operation_id = "updateMessagesState",
    request_body = BulkMessageStateUpdateRequest,
    responses((status = 204, description = "Message states updated"))
)]
pub async fn update_bulk_state(
    State(state): State<AppState>,
    Json(request): Json<BulkMessageStateUpdateRequest>,
) -> Result<axum::http::StatusCode, AppError> {
    state
        .repo
        .update_message_state(&request.ids, &request.state)
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    patch,
    path = "/api/messages/read-all",
    operation_id = "markAllMessagesRead",
    params(
        ("label" = Option<i64>, Query, description = "Filter by user label id"),
        ("mailbox" = Option<Mailbox>, Query, description = "Filter by mailbox; defaults to inbox"),
        ("q" = Option<String>, Query, description = "Full-text search"),
        ("read" = Option<bool>, Query, description = "Filter by current read state"),
        ("starred" = Option<bool>, Query, description = "Filter by starred state"),
        ("trashed" = Option<bool>, Query, description = "Show trashed messages")
    ),
    responses((status = 204, description = "All matching messages marked as read"))
)]
pub async fn mark_all_read(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<axum::http::StatusCode, AppError> {
    state
        .repo
        .mark_all_messages_read(&ListMessagesQuery {
            label_id: query.label,
            mailbox: query.mailbox.unwrap_or_default(),
            search: query.q.filter(|value| !value.trim().is_empty()),
            read: query.read,
            starred: query.starred,
            trashed: query.trashed.unwrap_or(false),
            limit: 0,
            offset: 0,
        })
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/trash",
    operation_id = "emptyTrash",
    responses((status = 204, description = "Trash permanently emptied"))
)]
pub async fn empty_trash(
    State(state): State<AppState>,
) -> Result<axum::http::StatusCode, AppError> {
    crate::services::trash::purge_all(&*state.repo).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

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

#[utoipa::path(
    patch,
    path = "/api/messages/{id}/mailbox",
    operation_id = "updateMessageMailbox",
    params(("id" = String, Path, description = "Message id")),
    request_body = MailboxUpdateRequest,
    responses(
        (status = 204, description = "Message mailbox updated"),
        (status = 404, description = "Message not found", body = ErrorResponse),
        (status = 500, description = "Repository failure", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state))]
pub async fn update_mailbox(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<MailboxUpdateRequest>,
) -> Result<axum::http::StatusCode, AppError> {
    let updated = state
        .repo
        .update_message_mailbox(&id, request.mailbox)
        .await?;
    if !updated {
        return Err(AppError::NotFound);
    }

    tracing::debug!(message_id = %id, mailbox = %request.mailbox, "updated message mailbox");
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/messages/{id}",
    operation_id = "deleteMessage",
    params(("id" = String, Path, description = "Message id")),
    responses(
        (status = 204, description = "Message permanently deleted"),
        (status = 404, description = "Message not found", body = ErrorResponse),
        (status = 500, description = "Repository failure", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state))]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<axum::http::StatusCode, AppError> {
    let files = state
        .repo
        .delete_message(&id)
        .await?
        .ok_or(AppError::NotFound)?;

    remove_blob_file(&files.raw_path).await;
    for path in files.attachment_paths {
        remove_blob_file(&path).await;
    }

    tracing::debug!(message_id = %id, "permanently deleted message");
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn remove_blob_file(path: &str) {
    match tokio::fs::remove_file(path).await {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => tracing::warn!(path, error = %err, "failed to delete message blob"),
    }
}

#[utoipa::path(
    get,
    path = "/api/messages/{id}/raw",
    operation_id = "downloadRawMessage",
    params(("id" = String, Path, description = "Message id")),
    responses(
        (
            status = 200,
            description = "Raw EML bytes",
            content_type = "message/rfc822"
        ),
        (status = 404, description = "Message not found", body = ErrorResponse),
        (status = 500, description = "Storage or repository failure", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state))]
pub async fn raw_download(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let message = state
        .repo
        .get_message_raw(&id)
        .await?
        .ok_or(AppError::NotFound)?;

    let bytes = tokio::fs::read(&message.raw_path).await?;

    let filename = {
        let sanitized: String = message
            .subject
            .chars()
            .take(80)
            .filter(|c| !c.is_control())
            .collect();
        if sanitized.is_empty() {
            format!("{id}.eml")
        } else {
            format!("{sanitized}.eml")
        }
    };

    tracing::debug!(
        message_id = %id,
        filename = %filename,
        byte_size = bytes.len(),
        "sending raw eml"
    );

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "message/rfc822".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!(r#"attachment; filename="{}""#, filename)
            .parse()
            .unwrap(),
    );

    Ok((headers, bytes))
}

#[utoipa::path(
    get,
    path = "/api/messages/{id}/headers",
    operation_id = "getMessageHeaders",
    params(("id" = String, Path, description = "Message id")),
    responses(
        (status = 200, description = "Parsed message headers from the persisted snapshot", body = MessageHeadersResponse),
        (status = 404, description = "Message not found", body = ErrorResponse),
        (status = 500, description = "Storage or repository failure", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state))]
pub async fn headers(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<MessageHeadersResponse>, AppError> {
    let headers = state
        .repo
        .get_message_headers(&id)
        .await?
        .ok_or(AppError::NotFound)?;

    tracing::debug!(
        message_id = %id,
        header_count = headers.len(),
        "retrieved message headers"
    );

    Ok(Json(MessageHeadersResponse { headers }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::repository::{InboundMessageRecord, sqlx::SqlxRepository};
    use crate::services::inbound::InboundMessageService;
    use crate::services::notifier::{NoopNotifier, Notifier};

    async fn setup_state() -> AppState {
        let repo = Arc::new(SqlxRepository::init_pool_in_memory().await.unwrap());
        let notifier = Arc::new(NoopNotifier) as Arc<dyn Notifier>;
        AppState {
            inbound_service: Arc::new(InboundMessageService::new(
                repo.clone(),
                std::path::PathBuf::from("/tmp"),
                notifier.clone(),
            )),
            repo,
            notifier,
        }
    }

    async fn insert_messages(repo: &dyn crate::repository::Repository, n: usize) {
        for i in 0..n {
            let raw = format!(
                "From: from@example.com\r\nTo: to@example.com\r\nSubject: Subject {i}\r\nMessage-ID: <msg-{i}>\r\nContent-Type: text/plain\r\n\r\nBody"
            );
            repo.ingest_message(InboundMessageRecord {
                id: format!("msg-{i}"),
                message_id: Some(format!("<msg-{i}>")),
                subject: format!("Subject {i}"),
                from_name: None,
                from_address: "from@example.com".to_string(),
                to_name: None,
                to_address: Some("to@example.com".to_string()),
                envelope_to: "to@example.com".to_string(),
                date: Some("2024-01-01T00:00:00+00:00".to_string()),
                raw_path: format!("/tmp/{i}.eml"),
                ingest_fingerprint: None,
                snapshot: crate::mime_parser::parse_message(raw.as_bytes())
                    .unwrap()
                    .snapshot,
                attachments: Vec::new(),
                label_ids: Vec::new(),
                initial_state: Default::default(),
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
            label: None,
            mailbox: None,
            q: None,
            read: None,
            starred: None,
            trashed: None,
        };
        let res = list(State(state), Query(query)).await.unwrap();
        assert_eq!(res.total, 5);
        assert_eq!(res.items.len(), 2);
    }

    #[tokio::test]
    async fn test_get_detail_404() {
        let state = setup_state().await;
        let res = detail(State(state), Path("missing".to_string())).await;
        assert!(matches!(res, Err(AppError::NotFound)));
    }
}
