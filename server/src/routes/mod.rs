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
pub mod facets;
pub mod health;
pub mod inbound;
pub mod labels;
pub mod messages;
pub mod rules;
pub mod saved_views;

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
            "/api/messages/bulk-state",
            patch(messages::update_bulk_state),
        )
        .route(
            "/api/messages/{id}",
            get(messages::detail).delete(messages::delete),
        )
        .route(
            "/api/messages/{id}/mailbox",
            patch(messages::update_mailbox),
        )
        .route("/api/messages/{id}/state", patch(messages::update_state))
        .route("/api/messages/{id}/raw", get(messages::raw_download))
        .route("/api/messages/{id}/headers", get(messages::headers))
        .route("/api/attachments/{id}", get(attachments::download))
        .route("/api/facets", get(facets::list))
        .route("/api/labels", get(labels::list).post(labels::create))
        .route(
            "/api/labels/{id}",
            patch(labels::update).delete(labels::delete),
        )
        .route(
            "/api/messages/{id}/labels",
            axum::routing::put(labels::set_message_labels),
        )
        .route("/api/rules", get(rules::list).post(rules::create))
        .route(
            "/api/rules/{id}",
            patch(rules::update).delete(rules::delete),
        )
        .route(
            "/api/saved-views",
            get(saved_views::list).post(saved_views::create),
        )
        .route(
            "/api/saved-views/{id}",
            patch(saved_views::update).delete(saved_views::delete),
        )
        .route("/api/trash", axum::routing::delete(messages::empty_trash))
        .with_state(state)
}
