use std::{env, net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{Router, http::HeaderValue};
use mail_shell_server::repository::sqlx::SqlxRepository;
use mail_shell_server::services::bark::BarkConfig;
use mail_shell_server::services::inbound::InboundMessageService;
use mail_shell_server::services::notifier::{NoopNotifier, Notifier};
use mail_shell_server::{routes, storage};
use tower::Layer;
use tower_http::{services::ServeDir, set_header::SetResponseHeaderLayer, trace::TraceLayer};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG")
                .unwrap_or_else(|_| "mail_shell_server=info,tower_http=info".into()),
        )
        .with_target(true)
        .init();

    let host = env::var("MAIL_SHELL_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = env::var("MAIL_SHELL_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);
    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("MAIL_SHELL_HOST and MAIL_SHELL_PORT must form a valid socket address");

    let data_dir = PathBuf::from(env::var("MAIL_SHELL_DATA_DIR").unwrap_or_else(|_| "data".into()));
    storage::ensure_dirs(&data_dir).expect("failed to create data directories");

    let repo = Arc::new(
        SqlxRepository::init_pool(&data_dir)
            .await
            .expect("failed to initialize database"),
    );

    let notifier: Arc<dyn Notifier> = match env::var("MAIL_SHELL_NOTIFIER")
        .unwrap_or_else(|_| "disabled".into())
        .as_str()
    {
        "bark" => {
            let server_url =
                env::var("MAIL_SHELL_BARK_SERVER_URL").unwrap_or_else(|_| "https://api.day.app".into());
            let key = env::var("MAIL_SHELL_BARK_KEY")
                .expect("MAIL_SHELL_BARK_KEY is required when MAIL_SHELL_NOTIFIER=bark");
            let group = env::var("MAIL_SHELL_BARK_GROUP").ok();
            let sound = env::var("MAIL_SHELL_BARK_SOUND").ok();
            let level = env::var("MAIL_SHELL_BARK_LEVEL").ok();
            info!(
                notifier = "bark",
                server_url = %server_url,
                group = ?group,
                "notifier configured"
            );
            Arc::new(mail_shell_server::services::bark::BarkNotifier::new(BarkConfig {
                server_url,
                key,
                group,
                sound,
                level,
            }))
        }
        _ => {
            info!(notifier = "disabled", "notifier configured");
            Arc::new(NoopNotifier)
        }
    };

    let inbound_service = Arc::new(InboundMessageService::new(repo.clone(), data_dir.clone(), notifier.clone()));

    let state = routes::AppState {
        repo,
        inbound_service,
        notifier,
    };

    let assets_dir = PathBuf::from("client/dist");
    let serve_dir = ServeDir::new(assets_dir).append_index_html_on_directories(true);
    let serve_dir = SetResponseHeaderLayer::overriding(
        axum::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    )
    .layer(serve_dir);

    let app = Router::new()
        .merge(routes::router(state))
        .fallback_service(serve_dir)
        .layer(TraceLayer::new_for_http());

    info!(listen_addr = %addr, "mail-shell server listening");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind listener");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server exited unexpectedly");
}

#[tracing::instrument]
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("received Ctrl+C, starting graceful shutdown"),
        _ = terminate => info!("received SIGTERM, starting graceful shutdown"),
    }
}
