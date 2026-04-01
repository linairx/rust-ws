//! rust-ws-proxy: WebSocket proxy service for VPS
//!
//! This service provides:
//! - WebSocket proxy for VLESS, Trojan, Shadowsocks protocols
//! - HTTP endpoints for health check and subscription

mod config;
mod server;

use std::sync::Arc;
use std::time::Duration;

use axum::{
    Router,
    routing::{any, get},
};
use reqwest::Client;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::server::{handlers, ws_handler};

/// Combined application state
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub client: Client,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("[fatal] rust-ws-proxy exited with error: {e:#}");
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    // Load .env file
    dotenvy::dotenv().ok();

    // Parse configuration
    let config = Config::from_env()?;
    let port = config.port;
    let ws_path = config.ws_path.clone();
    let sub_path = config.sub_path.clone();
    let auto_access = config.auto_access;

    // Initialize logging
    let log_level = if config.debug { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level)),
        )
        .init();

    info!("Starting WebSocket Proxy Server on port {}", port);
    info!("WebSocket path: /{}", ws_path);
    info!("Subscription path: /{}", sub_path);

    // Create HTTP client
    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

    // Create shared state
    let state = Arc::new(AppState {
        config: config.clone(),
        client: client.clone(),
    });

    // Build router
    let app = Router::new()
        // HTTP routes
        .route("/", get(handlers::index))
        .route("/health", get(handlers::health))
        // Subscription route
        .route(&format!("/{}", sub_path), get(handlers::subscription))
        // WebSocket route
        .route(&format!("/{}", ws_path), any(ws_handler))
        // Static files
        .nest_service("/static", ServeDir::new("static"))
        // State
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // Bind address
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("Server listening on {}", addr);

    // Start auto access task if enabled
    if auto_access {
        let config_clone = config.clone();
        let client_clone = client.clone();
        tokio::spawn(async move {
            auto_access_task(config_clone, client_clone).await;
        });
    }

    // Start server
    axum::serve(listener, app).await?;

    Ok(())
}

/// Auto access keep-alive task
async fn auto_access_task(config: Config, client: Client) {
    let url = format!("http://127.0.0.1:{}/health", config.port);

    loop {
        tokio::time::sleep(Duration::from_secs(300)).await;

        match client.get(&url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    info!("Auto access successful");
                }
            }
            Err(e) => {
                error!("Auto access failed: {}", e);
            }
        }
    }
}
