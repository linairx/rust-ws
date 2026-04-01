//! Configuration for WebSocket proxy service

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_uuid")]
    pub uuid: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub domain: String,
    #[serde(default = "default_sub_path")]
    pub sub_path: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_ws_path")]
    pub ws_path: String,
    #[serde(default)]
    pub auto_access: bool,
    #[serde(default)]
    pub debug: bool,
    #[serde(default)]
    pub allow_shadowsocks: bool,
}

fn default_uuid() -> String {
    "7bd180e8-1142-4387-93f5-03e8d750a896".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_sub_path() -> String {
    "sub".to_string()
}

fn default_ws_path() -> String {
    "7bd180e8".to_string()
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env()
    }

    pub fn ws_full_path(&self) -> String {
        format!("/{}", self.ws_path)
    }
}
