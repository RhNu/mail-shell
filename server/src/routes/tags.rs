use axum::{Json, extract::State};

use crate::error::AppError;
use crate::models::{ErrorResponse, Tag};
use crate::routes::AppState;

/// List all tags with their associated message counts.
#[utoipa::path(
    get,
    path = "/api/tags",
    operation_id = "listTags",
    responses(
        (status = 200, description = "Tag list with message counts", body = [Tag]),
        (status = 500, description = "Repository failure", body = ErrorResponse)
    )
)]
#[tracing::instrument]
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Tag>>, AppError> {
    let tags = state.repo.list_tags().await?;
    tracing::debug!(tag_count = tags.len(), "listed tags");
    Ok(Json(tags))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::repository::{InboundMessageRecord, InboundTagRecord, sqlx::SqlxRepository};
    use crate::services::inbound::InboundMessageService;
    use crate::services::notifier::{NoopNotifier, Notifier};

    #[tokio::test]
    async fn test_list_tags_with_counts() {
        let repo = Arc::new(SqlxRepository::init_pool_in_memory().await.unwrap());
        let notifier = Arc::new(NoopNotifier) as Arc<dyn Notifier>;
        let state = AppState {
            inbound_service: Arc::new(InboundMessageService::new(
                repo.clone(),
                std::path::PathBuf::from("/tmp"),
                notifier.clone(),
            )),
            repo,
            notifier,
        };

        state
            .repo
            .ingest_message(InboundMessageRecord {
                id: "msg-1".to_string(),
                from_address: "from@example.com".to_string(),
                to_address: "to@example.com".to_string(),
                subject: None,
                date: None,
                message_id: Some("<msg-1>".to_string()),
                raw_path: "/tmp/raw.msg".to_string(),
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

        let res = list(State(state)).await.unwrap();
        let tag = res.0.iter().find(|t| t.value == "to@example.com").unwrap();
        assert_eq!(tag.message_count, Some(1));
    }
}
