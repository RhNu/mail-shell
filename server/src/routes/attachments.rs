use axum::{
    extract::{Path, Query, State},
    http::header,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::error::AppError;
use crate::models::ErrorResponse;
use crate::routes::AppState;

#[derive(Debug, Default, Deserialize)]
pub struct AttachmentQuery {
    #[serde(default)]
    inline: bool,
}

fn can_display_inline(content_type: &str) -> bool {
    matches!(
        content_type.split(';').next().unwrap_or_default().trim(),
        "image/jpeg"
            | "image/png"
            | "image/gif"
            | "image/webp"
            | "image/avif"
            | "audio/mpeg"
            | "audio/ogg"
            | "video/mp4"
            | "video/webm"
            | "application/pdf"
            | "text/plain"
    )
}

/// Download an attachment by its ID.
///
/// Returns the file bytes with `Content-Type` and `Content-Disposition`
/// headers inferred from the database record.
#[utoipa::path(
    get,
    path = "/api/attachments/{id}",
    operation_id = "downloadAttachment",
    params(
        ("id" = String, Path, description = "Attachment id"),
        ("inline" = Option<bool>, Query, description = "Request inline browser display")
    ),
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
    Query(query): Query<AttachmentQuery>,
) -> Result<impl IntoResponse, AppError> {
    let meta = state
        .repo
        .get_attachment_download(&id)
        .await?
        .ok_or(AppError::NotFound)?;

    let bytes = tokio::fs::read(&meta.path).await?;
    let display_inline = query.inline
        && meta
            .content_type
            .as_deref()
            .is_some_and(can_display_inline);

    tracing::debug!(
        attachment_id = %id,
        filename = ?meta.filename,
        content_type = ?meta.content_type,
        byte_size = bytes.len(),
        "sending attachment"
    );

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
    if display_inline {
        headers.insert(
            header::CONTENT_SECURITY_POLICY,
            "sandbox; default-src 'none'".parse().unwrap(),
        );
    }
    if let Some(ct) = meta.content_type
        && let Ok(h) = ct.parse()
    {
        headers.insert(header::CONTENT_TYPE, h);
    }
    if let Some(fname) = meta.filename
        && let Ok(h) = format!(
            r#"{}; filename="{}""#,
            if display_inline { "inline" } else { "attachment" },
            fname
        )
        .parse()
    {
        headers.insert(header::CONTENT_DISPOSITION, h);
    }

    Ok((headers, bytes))
}

#[cfg(test)]
mod tests {
    use super::can_display_inline;

    #[test]
    fn inline_types_are_an_explicit_safe_list() {
        assert!(can_display_inline("image/png"));
        assert!(can_display_inline("text/plain; charset=utf-8"));
        assert!(!can_display_inline("text/html"));
        assert!(!can_display_inline("image/svg+xml"));
        assert!(!can_display_inline("application/octet-stream"));
    }
}
