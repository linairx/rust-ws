use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

use crate::config::Config;
use crate::utils::sha224_hash;

/// Generate VLESS subscription URL
pub fn generate_vless_url(config: &Config, host: &str, port: u16) -> String {
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
pub fn generate_trojan_url(config: &Config, host: &str, port: u16) -> String {
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
pub fn generate_shadowsocks_url(config: &Config, host: &str, port: u16) -> String {
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
pub fn generate_subscription(config: &Config, host: &str, port: u16) -> String {
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
