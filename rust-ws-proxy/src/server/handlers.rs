//! HTTP handlers

use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse},
};
use tracing::{error, info};

use crate::server::handlers::network::get_public_ip;
use crate::AppState;

/// Home page handler
pub async fn index() -> impl IntoResponse {
    Html(include_str!("../../static/index.html"))
}

/// Subscription handler
pub async fn subscription(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let config = &state.config;

    // Get host from config or detect public IP
    let host = if !config.domain.is_empty() {
        config.domain.clone()
    } else {
        match get_public_ip(&state.client).await {
            Ok(ip) => {
                info!("Detected public IP: {}", ip);
                ip
            }
            Err(e) => {
                error!("Failed to get public IP: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to get public IP".to_string(),
                )
                    .into_response();
            }
        }
    };

    let port = config.port;

    // Use rust-ws-core for subscription generation
    use rust_ws_core::{generate_subscription, NodeConfig};
    let node_config = NodeConfig::new(
        config.uuid.clone(),
        config.name.clone(),
        config.ws_path.clone(),
    );
    let sub_content = generate_subscription(&node_config, &host, port);

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"subscription.txt\""),
        ],
        sub_content,
    )
        .into_response()
}

/// Health check handler
pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Network utilities module
pub mod network {
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct GeoIpInfo {
        #[serde(default)]
        pub ip: String,
        #[serde(default)]
        pub org: String,
        #[serde(default)]
        pub country: String,
        #[serde(default)]
        pub city: String,
    }

    /// Get public IP address
    pub async fn get_public_ip(client: &Client) -> Result<String, String> {
        let response = client
            .get("https://api-ipv4.ip.sb/ip")
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let ip = response
            .text()
            .await
            .map_err(|e| e.to_string())?;

        Ok(ip.trim().to_string())
    }

    /// Get ISP/Geo information
    #[allow(dead_code)]
    pub async fn get_geo_info(client: &Client) -> Result<GeoIpInfo, String> {
        let response = client
            .get("https://api.ip.sb/geoip")
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let info: GeoIpInfo = response
            .json()
            .await
            .map_err(|e| e.to_string())?;

        Ok(info)
    }
}
