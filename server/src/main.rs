use std::{env, net::SocketAddr, path::PathBuf};

use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::Serialize;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::info;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    classification_model: &'static str,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG")
                .unwrap_or_else(|_| "mail_shell_server=info,tower_http=info".into()),
        )
        .init();

    let host = env::var("MAIL_SHELL_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = env::var("MAIL_SHELL_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);
    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("MAIL_SHELL_HOST and MAIL_SHELL_PORT must form a valid socket address");

    let assets_dir = PathBuf::from("client/dist");
    let app = Router::new()
        .route("/api/healthz", get(healthz))
        .route("/api/inbound", post(inbound_placeholder))
        .nest_service(
            "/",
            ServeDir::new(assets_dir).append_index_html_on_directories(true),
        )
        .layer(TraceLayer::new_for_http());

    info!(listen_addr = %addr, "mail-shell server scaffold listening");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind listener");
    axum::serve(listener, app)
        .await
        .expect("server exited unexpectedly");
}

async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        classification_model: "system-tags:recipient_address,recipient_domain",
    })
}

async fn inbound_placeholder() -> &'static str {
    "inbound scaffold ready"
}
