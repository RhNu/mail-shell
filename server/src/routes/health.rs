use axum::{extract::State, Json};
use serde::Serialize;

use crate::routes::AppState;

/// Health check response payload.
#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    classification_model: &'static str,
}

/// Simple health check endpoint.
#[tracing::instrument]
pub async fn handler(State(_state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        classification_model: "system-tags:recipient_address,recipient_domain",
    })
}
