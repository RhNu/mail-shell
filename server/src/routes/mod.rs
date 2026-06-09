use std::sync::Arc;

use axum::{
    Router,
    routing::{get, patch, post},
};

use crate::repository::Repository;
use crate::services::inbound::InboundMessageService;
use crate::services::notifier::Notifier;

pub mod api_docs;
pub mod attachments;
pub mod health;
pub mod inbound;
pub mod messages;
pub mod tags;

/// Shared application state passed to all Axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<dyn Repository>,
    pub inbound_service: Arc<InboundMessageService>,
    pub notifier: Arc<dyn Notifier>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState").finish_non_exhaustive()
    }
}

/// Build the Axum router with all API routes and the given state.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api-docs/openapi.json", get(api_docs::handler))
        .route("/api/healthz", get(health::handler))
        .route("/api/inbound", post(inbound::handler))
        .route("/api/messages", get(messages::list))
        .route(
            "/api/messages/{id}",
            get(messages::detail).delete(messages::delete),
        )
        .route(
            "/api/messages/{id}/mailbox",
            patch(messages::update_mailbox),
        )
        .route("/api/messages/{id}/raw", get(messages::raw_download))
        .route("/api/messages/{id}/headers", get(messages::headers))
        .route("/api/attachments/{id}", get(attachments::download))
        .route("/api/tags", get(tags::list))
        .with_state(state)
}
