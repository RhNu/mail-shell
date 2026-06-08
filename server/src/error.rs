use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    models::ErrorResponse, repository::RepositoryError, services::inbound::InboundServiceError,
    services::notifier::NotifierError,
};

/// Central application error type.
///
/// Implements `IntoResponse` so that any `Result<..., AppError>` returned
/// by an Axum handler is automatically converted into an HTTP response.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("mail parse error: {0}")]
    MailParse(#[from] mailparse::MailParseError),
    #[error(transparent)]
    Repo(#[from] RepositoryError),
    #[error(transparent)]
    InboundService(#[from] InboundServiceError),
    #[error("notifier error: {0}")]
    Notifier(#[from] NotifierError),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("not found")]
    NotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Repo(err) => {
                tracing::error!(error = %err, "repository error");
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::InboundService(err) => {
                tracing::error!(error = %err, "inbound service error");
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::Notifier(err) => {
                tracing::error!(error = %err, "notifier error");
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

        let body = Json(ErrorResponse { error: message });
        (status, body).into_response()
    }
}
