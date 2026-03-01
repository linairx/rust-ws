use serde::{Deserialize, Serialize};

/// 请求结构
#[derive(Debug, Deserialize)]
pub struct Request {
    pub cmd: String,

    // === sub 命令参数 ===
    pub uuid: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub name: Option<String>,
    pub ws_path: Option<String>,

    // === parse 命令参数 ===
    pub url: Option<String>,

    // === config 命令参数 ===
    pub protocol: Option<String>,
    pub format: Option<String>,
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

    pub fn error(msg: impl Into<String>) -> Self {
        Response {
            ok: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

/// 解析后的节点信息
#[derive(Debug, Serialize)]
pub struct ParsedNode {
    pub protocol: String,
    pub host: String,
    pub port: u16,
    pub uuid: Option<String>,
    pub password: Option<String>,
    pub name: String,
    pub raw: String,
}
