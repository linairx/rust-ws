//! Subscription URL generation
//!
//! This module provides functions to generate subscription URLs for various proxy protocols.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

use crate::utils::sha224_hash;

/// Node configuration for subscription generation
#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub uuid: String,
    pub name: String,
    pub ws_path: String,
}

impl NodeConfig {
    /// Create a new node configuration
    pub fn new(uuid: String, name: String, ws_path: String) -> Self {
        Self { uuid, name, ws_path }
    }
}

/// Generate VLESS subscription URL
pub fn generate_vless_url(config: &NodeConfig, host: &str, port: u16) -> String {
    format!(
        "vless://{}@{}:{}?encryption=none&security=none&type=ws&path=/{}#{}",
        config.uuid,
        host,
        port,
        config.ws_path,
        urlencoding::encode(&config.name)
    )
}

/// Generate Trojan subscription URL
pub fn generate_trojan_url(config: &NodeConfig, host: &str, port: u16) -> String {
    // Trojan password is the UUID itself
    let password_hash = sha224_hash(&config.uuid);
    format!(
        "trojan://{}@{}:{}?security=none&type=ws&path=/{}#{}",
        password_hash,
        host,
        port,
        config.ws_path,
        urlencoding::encode(&config.name)
    )
}

/// Generate Shadowsocks subscription URL
pub fn generate_shadowsocks_url(config: &NodeConfig, host: &str, port: u16) -> String {
    // Using UUID as password with AEAD-2022 cipher
    let userinfo = BASE64.encode(format!("2022-blake3-aes-256-gcm:{}", config.uuid));
    format!(
        "ss://{}@{}:{}#{}",
        userinfo,
        host,
        port,
        urlencoding::encode(&config.name)
    )
}

/// Generate full subscription content (Base64 encoded)
pub fn generate_subscription(config: &NodeConfig, host: &str, port: u16) -> String {
    let mut urls = Vec::new();

    urls.push(generate_vless_url(config, host, port));
    urls.push(generate_trojan_url(config, host, port));
    urls.push(generate_shadowsocks_url(config, host, port));

    let combined = urls.join("\n");
    BASE64.encode(combined)
}

/// URL encoding module
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut encoded = String::new();
        for c in s.chars() {
            match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                    encoded.push(c);
                }
                _ => {
                    for byte in c.to_string().as_bytes() {
                        encoded.push_str(&format!("%{:02X}", byte));
                    }
                }
            }
        }
        encoded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urlencoding() {
        assert_eq!(urlencoding::encode("hello"), "hello");
        assert_eq!(urlencoding::encode("hello world"), "hello%20world");
        assert_eq!(urlencoding::encode("测试"), "%E6%B5%8B%E8%AF%95");
    }

    #[test]
    fn test_generate_vless_url() {
        let config = NodeConfig::new(
            "7bd180e8-1142-4387-93f5-03e8d750a896".to_string(),
            "Test Node".to_string(),
            "7bd180e8".to_string(),
        );
        let url = generate_vless_url(&config, "example.com", 443);
        assert!(url.starts_with("vless://"));
        assert!(url.contains("example.com"));
        assert!(url.contains("443"));
    }
}
