mod api;
mod application;
mod config;
mod crypto;
mod domain;
mod grpc;
mod infrastructure;
mod services;
mod websocket;

use std::net::SocketAddr;
use std::sync::Arc;
use sqlx::PgPool;
use tokio::sync::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use config::Config;
use crypto::merkle::MerkleTree;
use websocket::manager::WsManager;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<Config>,
    pub http_client: reqwest::Client,
    pub ws_manager: WsManager,
    pub merkle_tree: Arc<RwLock<MerkleTree>>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    init_tracing();

    let config = Config::from_env().unwrap_or_else(|e| {
        tracing::error!("Configuration error: {e}");
        std::process::exit(1);
    });

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting pliq-back on {}",
        config.bind_address()
    );

    let db = pliq_back_db::create_pool(&config.database_url)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Database connection error: {e}");
            std::process::exit(1);
        });

    let merkle_tree = rebuild_merkle_tree(&db).await;

    let state = AppState {
        db,
        config: Arc::new(config.clone()),
        http_client: reqwest::Client::new(),
        ws_manager: WsManager::new(),
        merkle_tree: Arc::new(RwLock::new(merkle_tree)),
    };

    let app = api::routes::build_router(state.clone());

    let addr: SocketAddr = config
        .bind_address()
        .parse()
        .expect("Invalid bind address");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");
    tracing::info!("pliq-back listening on {addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server error");
}

fn init_tracing() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info".into());

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn rebuild_merkle_tree(pool: &PgPool) -> MerkleTree {
    let mut tree = MerkleTree::new();
    match pliq_back_db::queries::merkle::get_all_leaves_ordered(pool).await {
        Ok(leaves) => {
            for leaf in &leaves {
                tree.insert_leaf(&hex::decode(&leaf.leaf_hash).unwrap_or_default());
            }
            tracing::info!("Rebuilt Merkle tree with {} leaves", leaves.len());
        }
        Err(e) => tracing::warn!("Could not load Merkle leaves: {e}"),
    }
    tree
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Ctrl+C handler failed");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Signal handler failed")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
    tracing::info!("Shutdown signal received");
}
