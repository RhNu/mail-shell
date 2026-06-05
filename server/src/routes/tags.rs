use axum::{extract::State, Json};

use crate::error::AppError;
use crate::models::Tag;
use crate::routes::AppState;

/// List all tags with their associated message counts.
#[tracing::instrument]
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Tag>>, AppError> {
    let tags = state.repo.list_tags().await?;
    Ok(Json(tags))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::repository::sqlx::SqlxRepository;

    #[tokio::test]
    async fn test_list_tags_with_counts() {
        let repo = SqlxRepository::init_pool_in_memory().await.unwrap();
        let state = AppState {
            repo: Arc::new(repo),
            data_dir: std::path::PathBuf::from("/tmp"),
        };

        let tag_id = state
            .repo
            .ensure_tag("recipient_address", "to@example.com", "To: to@example.com")
            .await
            .unwrap();

        state
            .repo
            .create_message(
                "msg-1",
                "from@example.com",
                "to@example.com",
                None,
                None,
                None,
                "/tmp/raw.msg",
                None,
                None,
            )
            .await
            .unwrap();

        state.repo.link_message_tag("msg-1", tag_id).await.unwrap();

        let res = list(State(state)).await.unwrap();
        let tag = res.0.iter().find(|t| t.value == "to@example.com").unwrap();
        assert_eq!(tag.message_count, Some(1));
    }
}
