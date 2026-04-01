use crate::error::{ProxyError, Result};

/// Trojan protocol address types
pub const ATYPE_IPV4: u8 = 0x01;
pub const ATYPE_DOMAIN: u8 = 0x03;
pub const ATYPE_IPV6: u8 = 0x04;
pub const CMD_CONNECT: u8 = 0x01;

/// Parsed Trojan request
#[derive(Debug, Clone)]
pub struct TrojanRequest {
    pub password_hash: String,
    pub command: u8,
    pub address_type: u8,
    pub host: String,
    pub port: u16,
}

/// Parse Trojan protocol first packet
/// Format:
/// - Password (SHA224 hash, 56 bytes)
/// - CRLF (2 bytes: \r\n)
/// - Command (1 byte): 0x01=CONNECT
/// - Address type (1 byte): 0x01=IPv4, 0x03=Domain, 0x04=IPv6
/// - Target address (variable length)
/// - Target port (2 bytes)
/// - CRLF (2 bytes: \r\n)
pub fn parse_trojan(data: &[u8]) -> Result<TrojanRequest> {
    if data.len() < 62 {
        return Err(ProxyError::Parse("Trojan packet too short".to_string()));
    }

    // Extract password hash (first 56 bytes)
    let password_hash = String::from_utf8_lossy(&data[0..56]).to_string();

    // Check CRLF after password
    if data[56] != 0x0D || data[57] != 0x0A {
        return Err(ProxyError::Parse(
            "Invalid Trojan format: missing CRLF after password".to_string(),
        ));
    }

    // Command (byte 58)
    let command = data[58];
    if command != CMD_CONNECT {
        return Err(ProxyError::Parse(format!(
            "Unknown Trojan command: {}",
            command
        )));
    }

    // Address type (byte 59)
    let address_type = data[59];

    // Parse address based on type
    let (host, port_offset) = match address_type {
        ATYPE_IPV4 => {
            let addr_offset = 60;
            if data.len() < addr_offset + 4 {
                return Err(ProxyError::Parse("Trojan packet incomplete".to_string()));
            }
            let host = format!(
                "{}.{}.{}.{}",
                data[addr_offset],
                data[addr_offset + 1],
                data[addr_offset + 2],
                data[addr_offset + 3]
            );
            (host, addr_offset + 4)
        }
        ATYPE_DOMAIN => {
            let len_offset = 60;
            if data.len() < len_offset + 1 {
                return Err(ProxyError::Parse("Trojan packet incomplete".to_string()));
            }
            let domain_len = data[len_offset] as usize;
            let domain_offset = len_offset + 1;
            if data.len() < domain_offset + domain_len {
                return Err(ProxyError::Parse("Trojan packet incomplete".to_string()));
            }
            let host = String::from_utf8_lossy(&data[domain_offset..domain_offset + domain_len])
                .to_string();
            (host, domain_offset + domain_len)
        }
        ATYPE_IPV6 => {
            let addr_offset = 60;
            if data.len() < addr_offset + 16 {
                return Err(ProxyError::Parse("Trojan packet incomplete".to_string()));
            }
            let mut parts = Vec::new();
            for i in (0..16).step_by(2) {
                parts.push(format!(
                    "{:02x}{:02x}",
                    data[addr_offset + i],
                    data[addr_offset + i + 1]
                ));
            }
            (parts.join(":"), addr_offset + 16)
        }
        _ => {
            return Err(ProxyError::Parse(format!(
                "Unknown address type: {}",
                address_type
            )));
        }
    };

    // Parse port
    if data.len() < port_offset + 2 {
        return Err(ProxyError::Parse("Trojan packet incomplete".to_string()));
    }
    let port = u16::from_be_bytes([data[port_offset], data[port_offset + 1]]);

    Ok(TrojanRequest {
        password_hash,
        command,
        address_type,
        host,
        port,
    })
}
