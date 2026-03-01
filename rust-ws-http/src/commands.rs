use crate::types::{Request, Response};

/// 处理请求
pub fn handle(req: Request) -> Response {
    match req.cmd.as_str() {
        "health" => handle_health(),
        "sub" => handle_sub(req),
        _ => Response::error(format!("Unknown command: {}", req.cmd)),
    }
}

/// 健康检查
fn handle_health() -> Response {
    Response::success(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// 生成订阅链接
fn handle_sub(req: Request) -> Response {
    let server = req.server.unwrap_or_else(|| "example.com".to_string());
    let port = req.port.unwrap_or(443);
    let password = req.password.unwrap_or_else(|| "password".to_string());
    let method = req.method.unwrap_or_else(|| "aes-256-gcm".to_string());
    let name = req.name.unwrap_or_else(|| "Proxy Node".to_string());

    // 生成 ss:// 链接
    let config_str = format!("{}:{}@{}:{}", method, password, server, port);
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, config_str);
    let ss_link = format!(
        "ss://{}#{}",
        encoded,
        percent_encoding::percent_encode(name.as_bytes(), percent_encoding::NON_ALPHANUMERIC)
    );

    Response::success(serde_json::json!({
        "link": ss_link,
        "server": server,
        "port": port,
        "method": method
    }))
}
