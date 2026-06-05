use axum::{Router, routing::{get, post}};

pub mod attachments;
pub mod health;
pub mod inbound;
pub mod messages;
pub mod tags;

/// Shared application state passed to all Axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub data_dir: std::path::PathBuf,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("data_dir", &self.data_dir)
            .finish_non_exhaustive()
    }
}

/// Build the Axum router with all API routes and the given state.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/healthz", get(health::handler))
        .route("/api/inbound", post(inbound::handler))
        .route("/api/messages", get(messages::list))
        .route("/api/messages/{id}", get(messages::detail))
        .route("/api/attachments/{id}", get(attachments::download))
        .route("/api/tags", get(tags::list))
        .with_state(state)
}
