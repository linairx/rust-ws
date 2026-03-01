//! rust-ws-http - WASIX stdin/stdout 模式
//!
//! 通信协议:
//!   stdin: JSON 请求
//!   stdout: JSON 响应
//!
//! 请求格式:
//!   {"cmd": "health"}
//!   {"cmd": "sub", "server": "example.com", "port": 443, "password": "xxx", "method": "aes-256-gcm", "name": "Node"}
//!
//! 响应格式:
//!   {"ok": true, "data": {...}}
//!   {"ok": false, "error": "error message"}

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
