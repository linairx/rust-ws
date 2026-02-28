use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("Invalid UUID")]
    InvalidUuid,
    #[error("Invalid protocol")]
    InvalidProtocol,
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("DNS resolution failed: {0}")]
    DnsFailed(String),
    #[error("Blocked domain: {0}")]
    BlockedDomain(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, ProxyError>;
