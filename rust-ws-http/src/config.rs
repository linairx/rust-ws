//! Configuration for HTTP service

use serde::Deserialize;

/// Node configuration for a VPS proxy server
#[derive(Debug, Clone, Deserialize)]
pub struct VpsNode {
    pub uuid: String,
    pub domain: String,
    pub port: u16,
    pub name: String,
    pub ws_path: String,
}

impl VpsNode {
    /// Create from environment variable format: UUID:DOMAIN:PORT:NAME:WS_PATH
    pub fn from_str(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() < 4 {
            return None;
        }
        Some(Self {
            uuid: parts[0].to_string(),
            domain: parts[1].to_string(),
            port: parts[2].parse().ok()?,
            name: parts[3].to_string(),
            ws_path: parts.get(4).unwrap_or(&"7bd180e8").to_string(),
        })
    }
}

/// HTTP service configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub sub_path: String,
    pub nodes: Vec<VpsNode>,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);

        let sub_path = std::env::var("SUB_PATH")
            .unwrap_or_else(|_| "sub".to_string());

        // Load nodes from NODES environment variable
        // Format: UUID:DOMAIN:PORT:NAME:WS_PATH,UUID:DOMAIN:PORT:NAME:WS_PATH,...
        let nodes = std::env::var("NODES")
            .ok()
            .map(|s| {
                s.split(',')
                    .filter_map(|node_str| VpsNode::from_str(node_str.trim()))
                    .collect()
            })
            .unwrap_or_default();

        Self {
            port,
            sub_path,
            nodes,
        }
    }
}
