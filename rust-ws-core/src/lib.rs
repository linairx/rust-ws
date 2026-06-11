//! rust-ws-core: Core library for WebSocket proxy protocols
//!
//! This library provides protocol parsing and utilities for WebSocket proxies.

pub mod error;
pub mod proxy;
pub mod subscription;
pub mod utils;

pub use error::{ProxyError, Result};
pub use proxy::{parse_shadowsocks, parse_trojan, parse_vless};
pub use subscription::{
    generate_shadowsocks_url, generate_subscription, generate_subscription_with_options,
    generate_trojan_url, generate_trojan_url_with_options, generate_vless_url,
    generate_vless_url_with_options, NodeConfig, SubscriptionOptions,
};
pub use utils::{base64_decode, base64_encode, sha224_hash};
