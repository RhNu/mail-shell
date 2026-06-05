use axum::{
    extract::{Path, State},
    http::header,
    response::IntoResponse,
};

use crate::error::AppError;
use crate::models::ErrorResponse;
use crate::routes::AppState;

/// Download an attachment by its ID.
///
/// Returns the file bytes with `Content-Type` and `Content-Disposition`
/// headers inferred from the database record.
#[utoipa::path(
    get,
    path = "/api/attachments/{id}",
    operation_id = "downloadAttachment",
    params(("id" = String, Path, description = "Attachment id")),
    responses(
        (
            status = 200,
            description = "Attachment bytes",
            content_type = "application/octet-stream"
        ),
        (status = 404, description = "Attachment not found", body = ErrorResponse),
        (status = 500, description = "Storage or repository failure", body = ErrorResponse)
    )
)]
#[tracing::instrument]
pub async fn download(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let meta = state
        .repo
        .get_attachment_download(&id)
        .await?
        .ok_or(AppError::NotFound)?;

    let bytes = tokio::fs::read(&meta.path).await?;

    let mut headers = axum::http::HeaderMap::new();
    if let Some(ct) = meta.content_type
        && let Ok(h) = ct.parse()
    {
        headers.insert(header::CONTENT_TYPE, h);
    }
    if let Some(fname) = meta.filename
        && let Ok(h) = format!(r#"attachment; filename="{}""#, fname).parse()
    {
        headers.insert(header::CONTENT_DISPOSITION, h);
    }

    Ok((headers, bytes))
}
