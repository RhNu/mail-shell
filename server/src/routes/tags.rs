use axum::{extract::State, Json};

use crate::db::list_tags;
use crate::error::AppError;
use crate::models::Tag;
use crate::routes::AppState;

/// List all tags with their associated message counts.
#[tracing::instrument]
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Tag>>, AppError> {
    let tags = list_tags(&state.pool).await?;
    Ok(Json(tags))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[tokio::test]
    async fn test_list_tags_with_counts() {
        let pool = db::init_pool_in_memory().await.unwrap();
        let state = AppState {
            pool: pool.clone(),
            data_dir: std::path::PathBuf::from("/tmp"),
        };

        let tag_id = db::ensure_tag(&pool, "recipient_address", "to@example.com", "To: to@example.com")
            .await
            .unwrap();

        sqlx::query("INSERT INTO messages (id, from_address, to_address, raw_path) VALUES (?1, ?2, ?3, ?4)")
            .bind("msg-1")
            .bind("from@example.com")
            .bind("to@example.com")
            .bind("/tmp/raw.msg")
            .execute(&pool)
            .await
            .unwrap();

        db::link_message_tag(&pool, "msg-1", tag_id).await.unwrap();

        let res = list(State(state)).await.unwrap();
        let tag = res.0.iter().find(|t| t.value == "to@example.com").unwrap();
        assert_eq!(tag.message_count, Some(1));
    }
}
