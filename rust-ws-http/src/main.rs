//! rust-ws-http - WebSocket Proxy Toolkit (WASM Compatible)
//!
//! 通信协议:
//!   stdin: JSON 请求
//!   stdout: JSON 响应
//!
//! 可用命令:
//!   health  - 健康检查
//!   sub     - 生成订阅内容 (Base64)
//!   urls    - 生成各协议链接 (vless/trojan/ss)
//!   parse   - 解析代理链接
//!   help    - 显示帮助

use std::io::{Read, Write};

mod commands;
mod types;

use types::{Request, Response};

fn main() {
    // 1. 从 stdin 读取 JSON 请求
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        let err_response = Response::error(format!("Failed to read stdin: {}", e));
        let _ = write_response(&err_response);
        return;
    }

    // 空输入显示帮助
    if input.trim().is_empty() {
        let help_response = commands::handle(types::Request { cmd: "help".to_string(), uuid: None, host: None, port: None, name: None, ws_path: None, url: None, protocol: None, format: None });
        let _ = write_response(&help_response);
        return;
    }

    // 2. 解析请求
    let request: Request = match serde_json::from_str(&input) {
        Ok(r) => r,
        Err(e) => {
            let err_response = Response::error(format!("Invalid JSON: {}", e));
            let _ = write_response(&err_response);
            return;
        }
    };

    // 3. 处理命令
    let response = commands::handle(request);

    // 4. 输出响应
    let _ = write_response(&response);
}

fn write_response(response: &Response) -> std::io::Result<()> {
    let json = serde_json::to_string(response)?;
    std::io::stdout().write_all(json.as_bytes())?;
    std::io::stdout().flush()
}
