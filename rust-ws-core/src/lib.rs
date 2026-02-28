//! rust-ws-core: Core library for WebSocket proxy protocols
//!
//! This library provides protocol parsing and utilities for WebSocket proxies.

pub mod error;
pub mod proxy;
pub mod subscription;
pub mod utils;

pub use error::{ProxyError, Result};
pub use proxy::{parse_shadowsocks, parse_trojan, parse_vless};
pub use subscription::{generate_subscription, generate_vless_url, generate_trojan_url, generate_shadowsocks_url, NodeConfig};
pub use utils::{base64_decode, base64_encode, sha224_hash};
