use serde::{Deserialize, Serialize};

/// 请求结构
#[derive(Debug, Deserialize)]
pub struct Request {
    pub cmd: String,
    // sub 命令参数
    pub server: Option<String>,
    pub port: Option<u16>,
    pub password: Option<String>,
    pub method: Option<String>,
    pub name: Option<String>,
}

/// 响应结构
#[derive(Debug, Serialize)]
pub struct Response {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Response {
    pub fn success(data: serde_json::Value) -> Self {
        Response {
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: String) -> Self {
        Response {
            ok: false,
            data: None,
            error: Some(msg),
        }
    }
}
