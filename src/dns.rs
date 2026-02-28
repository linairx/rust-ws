use reqwest::Client;
use serde::Deserialize;
use crate::error::{ProxyError, Result};

#[derive(Debug, Deserialize)]
struct DnsResponse {
    #[serde(default)]
    answer: Vec<DnsAnswer>,
}

#[derive(Debug, Deserialize)]
struct DnsAnswer {
    #[serde(default)]
    data: String,
}

/// Resolve domain using Google DNS API
pub async fn resolve_domain(client: &Client, host: &str) -> Result<String> {
    let url = format!(
        "https://dns.google/resolve?name={}&type=A",
        host
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| ProxyError::DnsFailed(e.to_string()))?;

    let dns_response: DnsResponse = response
        .json()
        .await
        .map_err(|e| ProxyError::DnsFailed(e.to_string()))?;

    dns_response
        .answer
        .first()
        .map(|a| a.data.clone())
        .ok_or_else(|| ProxyError::DnsFailed(format!("No A record found for {}", host)))
}
