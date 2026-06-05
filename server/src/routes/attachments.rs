use axum::{
    extract::{Path, State},
    http::header,
    response::IntoResponse,
};

use crate::error::AppError;
use crate::routes::AppState;

/// Download an attachment by its ID.
///
/// Returns the file bytes with `Content-Type` and `Content-Disposition`
/// headers inferred from the database record.
#[tracing::instrument]
pub async fn download(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let row: (String, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT path, filename, content_type FROM attachments WHERE id = ?1",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let (path, filename, content_type) = row;
    let bytes = tokio::fs::read(&path).await?;

    let mut headers = axum::http::HeaderMap::new();
    if let Some(ct) = content_type
        && let Ok(h) = ct.parse()
    {
        headers.insert(header::CONTENT_TYPE, h);
    }
    if let Some(fname) = filename
        && let Ok(h) = format!(r#"attachment; filename="{}""#, fname).parse()
    {
        headers.insert(header::CONTENT_DISPOSITION, h);
    }

    Ok((headers, bytes))
}
