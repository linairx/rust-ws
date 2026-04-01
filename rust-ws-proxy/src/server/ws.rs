//! WebSocket handler

use std::sync::Arc;

use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, error, info, warn};

use rust_ws_core::{parse_shadowsocks, parse_trojan, parse_vless, sha224_hash};

use crate::AppState;

/// Blocked domains for speed testing
const BLOCKED_DOMAINS: &[&str] = &[
    "speedtest.net",
    "fast.com",
    "speedtest.cn",
    "speed.cloudflare.com",
    "speedof.me",
    "nflxvideo.net",
    "nflxso.net",
    "nflxext.com",
];

/// Protocol types
#[derive(Debug, Clone, Copy)]
pub enum Protocol {
    Vless,
    Trojan,
    Shadowsocks,
    Unknown,
}

/// Detect protocol from first packet data
pub fn detect_protocol(data: &[u8]) -> Protocol {
    if data.len() > 17 && data[0] == 0x00 {
        Protocol::Vless
    } else if data.len() >= 56 {
        // Trojan: starts with SHA224 hash (56 hex chars)
        let is_hex = data[0..56].iter().all(|&b| b.is_ascii_hexdigit());
        if is_hex && data.len() > 58 && data[56] == 0x0D && data[57] == 0x0A {
            Protocol::Trojan
        } else {
            Protocol::Unknown
        }
    } else if data.len() > 0 && matches!(data[0], 1 | 3 | 4) {
        Protocol::Shadowsocks
    } else {
        Protocol::Unknown
    }
}

fn parse_target(
    data: &[u8],
    hint: Protocol,
    expected_uuid: Option<&[u8; 16]>,
    expected_trojan_hash: Option<&str>,
    allow_shadowsocks: bool,
) -> Option<(Protocol, String, u16)> {
    let orders: &[Protocol] = match hint {
        Protocol::Vless => &[Protocol::Vless, Protocol::Trojan, Protocol::Shadowsocks],
        Protocol::Trojan => &[Protocol::Trojan, Protocol::Vless, Protocol::Shadowsocks],
        Protocol::Shadowsocks => &[Protocol::Shadowsocks, Protocol::Vless, Protocol::Trojan],
        Protocol::Unknown => &[Protocol::Vless, Protocol::Trojan, Protocol::Shadowsocks],
    };

    for p in orders {
        match p {
            Protocol::Vless => {
                if let Ok(req) = parse_vless(data) {
                    if let Some(uuid) = expected_uuid {
                        if &req.uuid != uuid {
                            continue;
                        }
                    }
                    return Some((Protocol::Vless, req.host, req.port));
                }
            }
            Protocol::Trojan => {
                if let Ok(req) = parse_trojan(data) {
                    if let Some(password_hash) = expected_trojan_hash {
                        if req.password_hash != password_hash {
                            continue;
                        }
                    }
                    return Some((Protocol::Trojan, req.host, req.port));
                }
            }
            Protocol::Shadowsocks => {
                if !allow_shadowsocks {
                    continue;
                }
                if let Ok(req) = parse_shadowsocks(data) {
                    return Some((Protocol::Shadowsocks, req.host, req.port));
                }
            }
            Protocol::Unknown => {}
        }
    }

    None
}

/// Check if domain is blocked
pub fn is_blocked_domain(host: &str) -> bool {
    let host_lower = host.to_lowercase();
    BLOCKED_DOMAINS
        .iter()
        .any(|d| host_lower == *d || host_lower.ends_with(&format!(".{}", d)))
}

/// Handle WebSocket upgrade request
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

/// Main WebSocket handler
async fn handle_websocket(socket: WebSocket, state: Arc<AppState>) {
    let (mut ws_tx, mut ws_rx) = socket.split();

    // Wait for first message to determine protocol
    let first_msg = match ws_rx.next().await {
        Some(Ok(msg)) => msg,
        _ => {
            debug!("No first message received");
            return;
        }
    };

    let data: axum::body::Bytes = match first_msg {
        Message::Binary(data) => data,
        _ => {
            debug!("First message is not binary, dropping");
            return;
        }
    };

    if data.is_empty() {
        warn!("Empty first packet");
        return;
    }

    // Detect protocol
    let hint = detect_protocol(&data);
    debug!("Detected protocol hint: {:?}", hint);

    let expected_uuid = parse_uuid_bytes(&state.config.uuid);
    let expected_trojan_hash = sha224_hash(&state.config.uuid);

    let (protocol, host, port) = match parse_target(
        &data,
        hint,
        expected_uuid.as_ref(),
        Some(expected_trojan_hash.as_str()),
        state.config.allow_shadowsocks,
    ) {
        Some(parsed) => parsed,
        None => {
            debug!("Unable to parse first packet as authorized VLESS/Trojan/Shadowsocks");
            return;
        }
    };

    // Check blocked domains
    if is_blocked_domain(&host) {
        warn!("Blocked domain: {}", host);
        return;
    }

    info!("Proxying to {}:{}", host, port);

    // Connect to target
    let target_addr = format!("{}:{}", host, port);
    let mut target_stream = match TcpStream::connect(&target_addr).await {
        Ok(stream) => stream,
        Err(e) => {
            error!("Failed to connect to {}: {}", target_addr, e);
            return;
        }
    };

    // For protocols that need initial data forwarding
    if matches!(protocol, Protocol::Shadowsocks) {
        // For Shadowsocks, forward the entire first packet
        if let Err(e) = target_stream.write_all(&data).await {
            error!("Failed to forward initial data: {}", e);
            return;
        }
    }
    // For VLESS and Trojan, forward the payload after the header
    else if let Some(payload_start) = get_payload_offset(&data, protocol) {
        if payload_start < data.len() {
            if let Err(e) = target_stream.write_all(&data[payload_start..]).await {
                error!("Failed to forward initial payload: {}", e);
                return;
            }
        }
    }

    // Start bidirectional forwarding
    let (mut target_read, mut target_write) = target_stream.split();

    let client_to_target = async {
        while let Some(msg) = ws_rx.next().await {
            match msg {
                Ok(Message::Binary(data)) => {
                    if target_write.write_all(&data).await.is_err() {
                        break;
                    }
                }
                Ok(Message::Text(text)) => {
                    if target_write.write_all(text.as_bytes()).await.is_err() {
                        break;
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => {}
            }
        }
    };

    let target_to_client = async {
        let mut buf = [0u8; 8192];
        loop {
            match target_read.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    let data = buf[..n].to_vec();
                    if ws_tx.send(Message::Binary(data.into())).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    };

    tokio::select! {
        _ = client_to_target => {}
        _ = target_to_client => {}
    }

    info!("Connection closed: {}:{}", host, port);
}

fn parse_uuid_bytes(uuid: &str) -> Option<[u8; 16]> {
    let hex = uuid.replace('-', "");
    if hex.len() != 32 {
        return None;
    }

    let mut out = [0u8; 16];
    for (i, byte) in out.iter_mut().enumerate() {
        let idx = i * 2;
        let chunk = &hex[idx..idx + 2];
        let value = u8::from_str_radix(chunk, 16).ok()?;
        *byte = value;
    }
    Some(out)
}

/// Get the offset where payload starts in the first packet
fn get_payload_offset(data: &[u8], protocol: Protocol) -> Option<usize> {
    match protocol {
        Protocol::Vless => {
            // VLESS: version(1) + uuid(16) + addl_len(1) + addl_info + cmd(1) + port(2) + atype(1) + addr
            if data.len() < 18 {
                return None;
            }
            let addl_len = data[17] as usize;
            let cmd_offset = 18 + addl_len;
            if data.len() < cmd_offset + 4 {
                return None;
            }
            let atype = data[cmd_offset + 3];
            let addr_offset = cmd_offset + 4;
            match atype {
                0x01 => Some(addr_offset + 4), // IPv4
                0x02 => {
                    if data.len() < addr_offset {
                        return None;
                    }
                    let domain_len = data[addr_offset] as usize;
                    Some(addr_offset + 1 + domain_len)
                }
                0x03 => Some(addr_offset + 16), // IPv6
                _ => None,
            }
        }
        Protocol::Trojan => {
            // Trojan: hash(56) + crlf(2) + cmd(1) + atype(1) + addr + port(2) + crlf(2)
            if data.len() < 60 {
                return None;
            }
            let atype = data[59];
            let addr_offset = 60;
            match atype {
                0x01 => Some(addr_offset + 4 + 2 + 2), // IPv4 + port + crlf
                0x03 => {
                    if data.len() < addr_offset {
                        return None;
                    }
                    let domain_len = data[addr_offset] as usize;
                    Some(addr_offset + 1 + domain_len + 2 + 2)
                }
                0x04 => Some(addr_offset + 16 + 2 + 2), // IPv6 + port + crlf
                _ => None,
            }
        }
        Protocol::Shadowsocks => {
            // Shadowsocks: entire packet is payload (already encrypted)
            Some(0)
        }
        Protocol::Unknown => None,
    }
}
