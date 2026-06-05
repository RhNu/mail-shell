use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::db::{ensure_tag, link_message_tag};
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

    sqlx::query(
        "INSERT INTO messages (id, from_address, to_address, subject, date, message_id, raw_path, body_text, body_html)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )
    .bind(&msg_id)
    .bind(&metadata.from)
    .bind(&metadata.to)
    .bind(&metadata.headers.subject)
    .bind(&metadata.headers.date)
    .bind(&metadata.headers.message_id)
    .bind(storage::raw_path(&state.data_dir, &msg_id).to_string_lossy().to_string())
    .bind(parsed.body_text)
    .bind(parsed.body_html)
    .execute(&state.pool)
    .await?;

    for att in parsed.attachments {
        let att_id = storage::generate_id();
        storage::save_attachment(&state.data_dir, &att_id, &att.body).await?;
        sqlx::query(
            "INSERT INTO attachments (id, message_id, filename, content_type, size, path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(&att_id)
        .bind(&msg_id)
        .bind(att.filename)
        .bind(att.content_type)
        .bind(att.body.len() as i64)
        .bind(storage::attachment_path(&state.data_dir, &att_id).to_string_lossy().to_string())
        .execute(&state.pool)
        .await?;
    }

    let to = metadata.to.trim().to_lowercase();
    let address_tag = ensure_tag(
        &state.pool,
        "recipient_address",
        &to,
        &format!("To: {to}"),
    )
    .await?;
    link_message_tag(&state.pool, &msg_id, address_tag).await?;

    if let Some((_, domain)) = to.rsplit_once('@') {
        let domain_tag = ensure_tag(
            &state.pool,
            "recipient_domain",
            domain,
            &format!("Domain: {domain}"),
        )
        .await?;
        link_message_tag(&state.pool, &msg_id, domain_tag).await?;
    }

    tracing::info!(msg_id, from = %metadata.from, to = %metadata.to, "inbound message ingested");
    Ok((StatusCode::CREATED, Json(InboundResponse { id: msg_id })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
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
        let pool = db::init_pool_in_memory().await.unwrap();
        let state = AppState {
            pool,
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
        let pool = db::init_pool_in_memory().await.unwrap();
        let state = AppState {
            pool,
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
