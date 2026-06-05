use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Central application error type.
///
/// Implements `IntoResponse` so that any `Result<..., AppError>` returned
/// by an Axum handler is automatically converted into an HTTP response.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("mail parse error: {0}")]
    MailParse(#[from] mailparse::MailParseError),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("not found")]
    NotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Db(err) => {
                tracing::error!(error = %err, "database error");
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::Io(err) => {
                tracing::error!(error = %err, "io error");
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::MailParse(err) => {
                tracing::warn!(error = %err, "mail parse error");
                (StatusCode::BAD_REQUEST, self.to_string())
            }
            AppError::BadRequest(msg) => {
                tracing::warn!(message = %msg, "bad request");
                (StatusCode::BAD_REQUEST, msg.clone())
            }
            AppError::NotFound => {
                tracing::warn!("not found");
                (StatusCode::NOT_FOUND, self.to_string())
            }
        };

        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}
