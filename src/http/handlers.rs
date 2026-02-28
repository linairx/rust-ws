use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse},
};
use tracing::{error, info};

use crate::http::subscription::generate_subscription;
use crate::network::get_public_ip;
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
    let sub_content = generate_subscription(config, &host, port);

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
