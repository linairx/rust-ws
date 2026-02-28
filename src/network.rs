use reqwest::Client;
use serde::Deserialize;
use crate::error::{ProxyError, Result};

#[derive(Debug, Deserialize)]
pub struct GeoIpInfo {
    #[serde(default)]
    pub ip: String,
    #[serde(default)]
    pub org: String,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub city: String,
}

/// Get public IP address
pub async fn get_public_ip(client: &Client) -> Result<String> {
    let response = client
        .get("https://api-ipv4.ip.sb/ip")
        .send()
        .await
        .map_err(|e| ProxyError::Http(e.to_string()))?;

    let ip = response
        .text()
        .await
        .map_err(|e| ProxyError::Http(e.to_string()))?;

    Ok(ip.trim().to_string())
}

/// Get ISP/Geo information
pub async fn get_geo_info(client: &Client) -> Result<GeoIpInfo> {
    let response = client
        .get("https://api.ip.sb/geoip")
        .send()
        .await
        .map_err(|e| ProxyError::Http(e.to_string()))?;

    let info: GeoIpInfo = response
        .json()
        .await
        .map_err(|e| ProxyError::Http(e.to_string()))?;

    Ok(info)
}
