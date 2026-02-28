//! HTTP handlers

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse},
};

use rust_ws_core::{generate_subscription, NodeConfig};

use crate::config::Config;

/// Home page handler
pub async fn index() -> impl IntoResponse {
    Html(include_str!("../static/index.html"))
}

/// Health check handler
pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Subscription handler
pub async fn subscription(State(config): State<Config>) -> impl IntoResponse {
    if config.nodes.is_empty() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "No nodes configured. Set NODES environment variable.",
        )
            .into_response();
    }

    // Generate subscription content for all nodes
    let mut all_urls = Vec::new();

    for node in &config.nodes {
        let node_config = NodeConfig::new(
            node.uuid.clone(),
            node.name.clone(),
            node.ws_path.clone(),
        );

        let sub_content = generate_subscription(&node_config, &node.domain, node.port);
        // Decode and add to combined list
        if let Ok(decoded) = base64_decode(&sub_content) {
            if let Ok(urls) = String::from_utf8(decoded) {
                all_urls.push(urls);
            }
        }
    }

    let combined = all_urls.join("\n");
    let encoded = base64_encode(&combined);

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"subscription.txt\""),
        ],
        encoded,
    )
        .into_response()
}

/// Base64 encode helper
fn base64_encode(input: &str) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.encode(input)
}

/// Base64 decode helper
fn base64_decode(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.decode(input)
}
