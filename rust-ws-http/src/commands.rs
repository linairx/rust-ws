//! Command handlers for rust-ws-http toolkit

use crate::types::{ParsedNode, Request, Response};
use rust_ws_core::{
    generate_shadowsocks_url, generate_subscription, generate_trojan_url, generate_vless_url,
    NodeConfig,
};

/// 处理请求
pub fn handle(req: Request) -> Response {
    match req.cmd.as_str() {
        "health" => handle_health(),
        "sub" => handle_sub(req),
        "parse" => handle_parse(req),
        "urls" => handle_urls(req),
        "help" => handle_help(),
        _ => Response::error(format!("Unknown command: {}. Use 'help' for available commands.", req.cmd)),
    }
}

/// 健康检查
fn handle_health() -> Response {
    Response::success(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "features": ["vless", "trojan", "shadowsocks", "subscription"]
    }))
}

/// 帮助信息
fn handle_help() -> Response {
    Response::success(serde_json::json!({
        "commands": {
            "health": {
                "desc": "健康检查",
                "params": []
            },
            "sub": {
                "desc": "生成订阅内容 (Base64)",
                "params": ["uuid", "host", "port", "name", "ws_path"]
            },
            "urls": {
                "desc": "生成各协议链接",
                "params": ["uuid", "host", "port", "name", "ws_path"]
            },
            "parse": {
                "desc": "解析代理链接",
                "params": ["url"]
            },
            "help": {
                "desc": "显示帮助",
                "params": []
            }
        },
        "examples": {
            "health": {"cmd": "health"},
            "sub": {"cmd": "sub", "uuid": "xxx", "host": "example.com", "port": 443, "name": "Node", "ws_path": "ws"},
            "parse": {"cmd": "parse", "url": "vless://xxx@host:443?..."}
        }
    }))
}

/// 生成订阅内容
fn handle_sub(req: Request) -> Response {
    let config = match build_node_config(&req) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let host = req.host.unwrap_or_else(|| "example.com".to_string());
    let port = req.port.unwrap_or(443);

    let subscription = generate_subscription(&config, &host, port);

    Response::success(serde_json::json!({
        "subscription": subscription,
        "host": host,
        "port": port,
        "name": config.name,
        "protocols": ["vless", "trojan", "shadowsocks"]
    }))
}

/// 生成各协议链接
fn handle_urls(req: Request) -> Response {
    let config = match build_node_config(&req) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let host = req.host.unwrap_or_else(|| "example.com".to_string());
    let port = req.port.unwrap_or(443);

    let vless = generate_vless_url(&config, &host, port);
    let trojan = generate_trojan_url(&config, &host, port);
    let shadowsocks = generate_shadowsocks_url(&config, &host, port);

    Response::success(serde_json::json!({
        "vless": vless,
        "trojan": trojan,
        "shadowsocks": shadowsocks,
        "host": host,
        "port": port,
        "name": config.name
    }))
}

/// 解析代理链接
fn handle_parse(req: Request) -> Response {
    let url = match req.url {
        Some(u) => u,
        None => return Response::error("Missing 'url' parameter"),
    };

    let node = if url.starts_with("vless://") {
        parse_vless_url(&url)
    } else if url.starts_with("trojan://") {
        parse_trojan_url(&url)
    } else if url.starts_with("ss://") {
        parse_ss_url(&url)
    } else {
        return Response::error("Unsupported protocol. Supported: vless, trojan, ss");
    };

    match node {
        Some(n) => Response::success(serde_json::to_value(n).unwrap()),
        None => Response::error("Failed to parse URL"),
    }
}

// ============ 辅助函数 ============

fn build_node_config(req: &Request) -> Result<NodeConfig, Response> {
    let uuid = match &req.uuid {
        Some(u) => u.clone(),
        None => return Err(Response::error("Missing 'uuid' parameter")),
    };

    let ws_path = req.ws_path.clone().unwrap_or_else(|| {
        uuid.split('-').next().unwrap_or("ws").to_string()
    });

    Ok(NodeConfig::new(
        uuid,
        req.name.clone().unwrap_or_else(|| "Proxy Node".to_string()),
        ws_path,
    ))
}

fn parse_vless_url(url: &str) -> Option<ParsedNode> {
    // vless://uuid@host:port?params#name
    let url = url.strip_prefix("vless://")?;

    let (uuid, rest) = url.split_once('@')?;
    let (server_part, name) = rest.split_once('#').unwrap_or((rest, "Unknown"));

    let (host_port, _params) = server_part.split_once('?').unwrap_or((server_part, ""));
    let (host, port) = host_port.rsplit_once(':')?;

    Some(ParsedNode {
        protocol: "vless".to_string(),
        host: urldecode(host),
        port: port.parse().ok()?,
        uuid: Some(uuid.to_string()),
        password: None,
        name: urldecode(name),
        raw: format!("vless://{}@{}:{}", uuid, host, port),
    })
}

fn parse_trojan_url(url: &str) -> Option<ParsedNode> {
    // trojan://password@host:port?params#name
    let url = url.strip_prefix("trojan://")?;

    let (password, rest) = url.split_once('@')?;
    let (server_part, name) = rest.split_once('#').unwrap_or((rest, "Unknown"));

    let (host_port, _params) = server_part.split_once('?').unwrap_or((server_part, ""));
    let (host, port) = host_port.rsplit_once(':')?;

    Some(ParsedNode {
        protocol: "trojan".to_string(),
        host: urldecode(host),
        port: port.parse().ok()?,
        uuid: None,
        password: Some(password.to_string()),
        name: urldecode(name),
        raw: format!("trojan://***@{}:{}", host, port),
    })
}

fn parse_ss_url(url: &str) -> Option<ParsedNode> {
    // ss://base64(method:password)@host:port#name
    // or ss://base64(method:password@host:port)#name
    let url = url.strip_prefix("ss://")?;

    let (main_part, name) = url.split_once('#').unwrap_or((url, "Unknown"));

    // Try format: userinfo@host:port
    if let Some((userinfo, host_port)) = main_part.split_once('@') {
        let decoded = base64_decode(userinfo)?;
        let (method, password) = decoded.split_once(':')?;
        let (host, port) = host_port.rsplit_once(':')?;

        return Some(ParsedNode {
            protocol: "shadowsocks".to_string(),
            host: urldecode(host),
            port: port.parse().ok()?,
            uuid: None,
            password: Some(format!("{}:{}", method, password)),
            name: urldecode(name),
            raw: format!("ss://***@{}:{}", host, port),
        });
    }

    // Try format: base64(method:password@host:port)
    let decoded = base64_decode(main_part)?;
    if let Some((userinfo, host_port)) = decoded.split_once('@') {
        let (method, password) = userinfo.split_once(':')?;
        let (host, port) = host_port.rsplit_once(':')?;

        return Some(ParsedNode {
            protocol: "shadowsocks".to_string(),
            host: host.to_string(),
            port: port.parse().ok()?,
            uuid: None,
            password: Some(format!("{}:{}", method, password)),
            name: urldecode(name),
            raw: format!("ss://***@{}:{}", host, port),
        });
    }

    None
}

fn base64_decode(s: &str) -> Option<String> {
    // Add padding if needed
    let padded = match s.len() % 4 {
        0 => s.to_string(),
        2 => format!("{}==", s),
        3 => format!("{}=", s),
        _ => return None,
    };

    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, padded)
        .ok()
        .and_then(|b| String::from_utf8(b).ok())
}

fn urldecode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                if let Some(decoded) = char::from_u32(byte as u32) {
                    result.push(decoded);
                    continue;
                }
            }
        }
        result.push(c);
    }

    // Also replace + with space
    result.replace('+', " ")
}
