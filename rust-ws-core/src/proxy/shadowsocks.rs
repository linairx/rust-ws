use crate::error::{ProxyError, Result};

/// Shadowsocks protocol address types
pub const ATYPE_IPV4: u8 = 0x01;
pub const ATYPE_DOMAIN: u8 = 0x03;
pub const ATYPE_IPV6: u8 = 0x04;

/// Parsed Shadowsocks request
#[derive(Debug, Clone)]
pub struct ShadowsocksRequest {
    pub address_type: u8,
    pub host: String,
    pub port: u16,
}

/// Parse Shadowsocks protocol first packet
/// Format:
/// - Address type (1 byte): 0x01=IPv4, 0x03=Domain, 0x04=IPv6
/// - Target address (variable length)
/// - Target port (2 bytes)
pub fn parse_shadowsocks(data: &[u8]) -> Result<ShadowsocksRequest> {
    if data.is_empty() {
        return Err(ProxyError::Parse("Shadowsocks packet empty".to_string()));
    }

    let address_type = data[0];

    // Parse address based on type
    let (host, port_offset) = match address_type {
        ATYPE_IPV4 => {
            if data.len() < 7 {
                return Err(ProxyError::Parse(
                    "Shadowsocks packet incomplete".to_string(),
                ));
            }
            let host = format!("{}.{}.{}.{}", data[1], data[2], data[3], data[4]);
            (host, 5)
        }
        ATYPE_DOMAIN => {
            if data.len() < 2 {
                return Err(ProxyError::Parse(
                    "Shadowsocks packet incomplete".to_string(),
                ));
            }
            let domain_len = data[1] as usize;
            let domain_offset = 2;
            if data.len() < domain_offset + domain_len + 2 {
                return Err(ProxyError::Parse(
                    "Shadowsocks packet incomplete".to_string(),
                ));
            }
            let host = String::from_utf8_lossy(&data[domain_offset..domain_offset + domain_len])
                .to_string();
            (host, domain_offset + domain_len)
        }
        ATYPE_IPV6 => {
            if data.len() < 19 {
                return Err(ProxyError::Parse(
                    "Shadowsocks packet incomplete".to_string(),
                ));
            }
            let mut parts = Vec::new();
            for i in (1..17).step_by(2) {
                parts.push(format!("{:02x}{:02x}", data[i], data[i + 1]));
            }
            (parts.join(":"), 17)
        }
        _ => {
            return Err(ProxyError::Parse(format!(
                "Unknown address type: {}",
                address_type
            )))
        }
    };

    // Parse port
    if data.len() < port_offset + 2 {
        return Err(ProxyError::Parse(
            "Shadowsocks packet incomplete".to_string(),
        ));
    }
    let port = u16::from_be_bytes([data[port_offset], data[port_offset + 1]]);

    Ok(ShadowsocksRequest {
        address_type,
        host,
        port,
    })
}
