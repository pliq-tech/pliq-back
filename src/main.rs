mod api;
mod config;
mod crypto;
mod domain;
mod grpc;
mod services;
mod websocket;

use std::net::SocketAddr;
use std::sync::Arc;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use config::Config;
use websocket::manager::WsManager;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<Config>,
    pub http_client: reqwest::Client,
    pub ws_manager: WsManager,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env().unwrap_or_else(|e| {
        tracing::error!("Configuration error: {}", e);
        std::process::exit(1);
    });

    tracing::info!("Starting pliq-back on {}", config.bind_address());

    let db = pliq_back_db::create_pool(&config.database_url).await.unwrap_or_else(|e| {
        tracing::error!("Database connection error: {}", e);
        std::process::exit(1);
    });

    let state = AppState {
        db,
        config: Arc::new(config.clone()),
        http_client: reqwest::Client::new(),
        ws_manager: WsManager::new(),
    };

    let app = api::routes::build_router(state)
        .layer(api::middleware::cors::cors_layer())
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = config.bind_address().parse().expect("Invalid bind address");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind");
    tracing::info!("pliq-back listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server error");
}

async fn shutdown_signal() {
    let ctrl_c = async { tokio::signal::ctrl_c().await.expect("Ctrl+C handler failed"); };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Signal handler failed").recv().await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
    tracing::info!("Shutdown signal received");
}
