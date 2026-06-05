use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::error::AppError;
use crate::models::{InboundMetadata, InboundResponse};
use crate::mime_parser::parse_message;
use crate::routes::AppState;
use crate::storage;

/// Ingest a raw MIME email via multipart form data.
///
/// Expects two fields:
/// - `raw_mime` — the raw email bytes.
/// - `metadata` — JSON with `from`, `to`, and `headers`.
///
/// On success returns `201 Created` with the generated message ID.
#[tracing::instrument(skip(state, multipart))]
pub async fn handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut raw_mime: Option<Vec<u8>> = None;
    let mut metadata: Option<InboundMetadata> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "raw_mime" {
            raw_mime = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?
                    .to_vec(),
            );
        } else if name == "metadata" {
            let text = field
                .text()
                .await
                .map_err(|e| AppError::BadRequest(e.to_string()))?;
            metadata = Some(
                serde_json::from_str(&text)
                    .map_err(|e| AppError::BadRequest(e.to_string()))?,
            );
        }
    }

    let raw_mime = raw_mime.ok_or_else(|| AppError::BadRequest("missing raw_mime".to_string()))?;
    let metadata = metadata.ok_or_else(|| AppError::BadRequest("missing metadata".to_string()))?;

    let msg_id = storage::generate_id();
    storage::ensure_dirs(&state.data_dir)?;
    storage::save_raw(&state.data_dir, &msg_id, &raw_mime).await?;

    let parsed = parse_message(&raw_mime)?;

    state
        .repo
        .create_message(
            &msg_id,
            &metadata.from,
            &metadata.to,
            Some(&metadata.headers.subject),
            Some(&metadata.headers.date),
            Some(&metadata.headers.message_id),
            &storage::raw_path(&state.data_dir, &msg_id).to_string_lossy(),
            parsed.body_text.as_deref(),
            parsed.body_html.as_deref(),
        )
        .await?;

    for att in parsed.attachments {
        let att_id = storage::generate_id();
        storage::save_attachment(&state.data_dir, &att_id, &att.body).await?;
        state
            .repo
            .create_attachment(
                &att_id,
                &msg_id,
                att.filename.as_deref(),
                Some(&att.content_type),
                att.body.len() as i64,
                &storage::attachment_path(&state.data_dir, &att_id).to_string_lossy(),
            )
            .await?;
    }

    let to = metadata.to.trim().to_lowercase();
    let address_tag = state
        .repo
        .ensure_tag("recipient_address", &to, &format!("To: {to}"))
        .await?;
    state.repo.link_message_tag(&msg_id, address_tag).await?;

    if let Some((_, domain)) = to.rsplit_once('@') {
        let domain_tag = state
            .repo
            .ensure_tag("recipient_domain", domain, &format!("Domain: {domain}"))
            .await?;
        state.repo.link_message_tag(&msg_id, domain_tag).await?;
    }

    tracing::info!(msg_id, from = %metadata.from, to = %metadata.to, "inbound message ingested");
    Ok((StatusCode::CREATED, Json(InboundResponse { id: msg_id })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::repository::sqlx::SqlxRepository;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn build_multipart_body(raw_mime: &[u8], metadata: &str) -> (Vec<u8>, String) {
        let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
        let mut body = Vec::new();
        body.extend_from_slice(
            format!("--{boundary}\r\nContent-Disposition: form-data; name=\"raw_mime\"; filename=\"message.eml\"\r\nContent-Type: message/rfc822\r\n\r\n").as_bytes(),
        );
        body.extend_from_slice(raw_mime);
        body.extend_from_slice(
            format!("\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"metadata\"\r\n\r\n{metadata}\r\n--{boundary}--\r\n").as_bytes(),
        );
        (body, boundary.to_string())
    }

    #[tokio::test]
    async fn test_inbound_handler_multipart() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = SqlxRepository::init_pool_in_memory().await.unwrap();
        let state = AppState {
            repo: Arc::new(repo),
            data_dir: tmp.path().to_path_buf(),
        };

        let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello\r\nContent-Type: text/plain\r\n\r\nBody";
        let meta = r#"{"from":"sender@example.com","to":"recipient@example.com","headers":{"message-id":"<abc>","subject":"Hello","date":"Mon, 01 Jan 2024 00:00:00 +0000"}}"#;
        let (body, boundary) = build_multipart_body(raw, meta);

        let req = Request::builder()
            .method("POST")
            .uri("/api/inbound")
            .header(
                "Content-Type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap();

        let app = crate::routes::router(state);
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_inbound_missing_raw_mime() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = SqlxRepository::init_pool_in_memory().await.unwrap();
        let state = AppState {
            repo: Arc::new(repo),
            data_dir: tmp.path().to_path_buf(),
        };

        let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
        let body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"metadata\"\r\n\r\n{{}}\r\n--{boundary}--\r\n"
        );

        let req = Request::builder()
            .method("POST")
            .uri("/api/inbound")
            .header(
                "Content-Type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap();

        let app = crate::routes::router(state);
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}
