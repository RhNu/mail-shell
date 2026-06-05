use axum::{
    Json,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::error::AppError;
use crate::models::{ErrorResponse, InboundMetadata, InboundResponse};
use crate::routes::AppState;

/// Ingest a raw MIME email via multipart form data.
///
/// Expects two fields:
/// - `raw_mime` — the raw email bytes.
/// - `metadata` — JSON with `from`, `to`, and `headers`.
///
/// On success returns `201 Created` with the generated message ID.
#[utoipa::path(
    post,
    path = "/api/inbound",
    operation_id = "ingestInboundMessage",
    request_body(
        content = crate::api_docs::InboundMultipartRequest,
        content_type = "multipart/form-data",
        description = "Raw MIME email plus envelope metadata"
    ),
    responses(
        (status = 201, description = "Inbound message accepted", body = InboundResponse),
        (status = 400, description = "Bad multipart payload", body = ErrorResponse),
        (status = 500, description = "Ingest failure", body = ErrorResponse)
    )
)]
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
            metadata =
                Some(serde_json::from_str(&text).map_err(|e| AppError::BadRequest(e.to_string()))?);
        }
    }

    let raw_mime = raw_mime.ok_or_else(|| AppError::BadRequest("missing raw_mime".to_string()))?;
    let metadata = metadata.ok_or_else(|| AppError::BadRequest("missing metadata".to_string()))?;

    let response = state.inbound_service.ingest(raw_mime, metadata).await?;
    tracing::info!(msg_id = %response.id, "inbound message ingested");
    Ok((
        StatusCode::CREATED,
        Json(InboundResponse { id: response.id }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::repository::sqlx::SqlxRepository;
    use crate::services::inbound::InboundMessageService;
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
        let repo = Arc::new(SqlxRepository::init_pool_in_memory().await.unwrap());
        let state = AppState {
            inbound_service: Arc::new(InboundMessageService::new(
                repo.clone(),
                tmp.path().to_path_buf(),
            )),
            repo,
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
        let repo = Arc::new(SqlxRepository::init_pool_in_memory().await.unwrap());
        let state = AppState {
            inbound_service: Arc::new(InboundMessageService::new(
                repo.clone(),
                tmp.path().to_path_buf(),
            )),
            repo,
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
