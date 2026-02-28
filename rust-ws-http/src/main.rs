//! rust-ws-http: HTTP service for Wasmer Edge
//!
//! This service provides:
//! - Health check endpoint
//! - Subscription page that generates proxy URLs for VPS nodes

mod config;
mod handlers;

use std::net::SocketAddr;

use axum::{
    routing::get,
    Router,
};
use tracing::info;

use crate::config::Config;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration from environment
    let config = Config::from_env();
    let port = config.port;

    info!("Starting HTTP service on port {}", port);
    info!("Subscription path: /{}", config.sub_path);

    // Build router
    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/", get(handlers::index))
        .route(&format!("/{}", config.sub_path), get(handlers::subscription))
        .with_state(config);

    // Bind address - use 0.0.0.0 for Wasmer Edge compatibility
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind");

    info!("Server listening on {}", addr);

    // Start server using axum 0.7 API
    axum::serve(listener, app).await.expect("Server failed");
}
