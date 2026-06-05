use axum::{Json, extract::State};
use serde::Serialize;
use utoipa::ToSchema;

use crate::routes::AppState;

/// Health check response payload.
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
    pub classification_model: &'static str,
}

/// Simple health check endpoint.
#[utoipa::path(
    get,
    path = "/api/healthz",
    operation_id = "getHealth",
    responses((status = 200, description = "Service health", body = HealthResponse))
)]
#[tracing::instrument]
pub async fn handler(State(_state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        classification_model: "system-tags:recipient_address,recipient_domain",
    })
}
