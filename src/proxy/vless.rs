use crate::error::{ProxyError, Result};

/// VLESS protocol address types
pub const ATYPE_IPV4: u8 = 0x01;
pub const ATYPE_DOMAIN: u8 = 0x02;
pub const ATYPE_IPV6: u8 = 0x03;

/// VLESS protocol commands
pub const CMD_TCP: u8 = 0x01;
pub const CMD_UDP: u8 = 0x02;

/// Parsed VLESS request
#[derive(Debug, Clone)]
pub struct VlessRequest {
    pub uuid: [u8; 16],
    pub command: u8,
    pub address_type: u8,
    pub host: String,
    pub port: u16,
}

/// Parse VLESS protocol first packet
/// Format:
/// - Version (1 byte): 0x00
/// - UUID (16 bytes)
/// - Additional info length (1 byte)
/// - Command (1 byte): 0x01=TCP, 0x02=UDP
/// - Address type (1 byte): 0x01=IPv4, 0x02=Domain, 0x03=IPv6
/// - Target address (variable length)
/// - Target port (2 bytes)
pub fn parse_vless(data: &[u8]) -> Result<VlessRequest> {
    if data.len() < 20 {
        return Err(ProxyError::Parse("VLESS packet too short".to_string()));
    }

    // Check version
    if data[0] != 0x00 {
        return Err(ProxyError::InvalidProtocol);
    }

    // Extract UUID (bytes 1-16)
    let mut uuid = [0u8; 16];
    uuid.copy_from_slice(&data[1..17]);

    // Additional info length (byte 17)
    let addl_len = data[17] as usize;

    // Command is after additional info
    let cmd_offset = 18 + addl_len;
    if data.len() < cmd_offset + 1 {
        return Err(ProxyError::Parse("VLESS packet incomplete".to_string()));
    }

    let command = data[cmd_offset];

    // Address type
    let atype_offset = cmd_offset + 1;
    if data.len() < atype_offset + 1 {
        return Err(ProxyError::Parse("VLESS packet incomplete".to_string()));
    }
    let address_type = data[atype_offset];

    // Parse address based on type
    let (host, port_offset) = match address_type {
        ATYPE_IPV4 => {
            let addr_offset = atype_offset + 1;
            if data.len() < addr_offset + 4 {
                return Err(ProxyError::Parse("VLESS packet incomplete".to_string()));
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
            let len_offset = atype_offset + 1;
            if data.len() < len_offset + 1 {
                return Err(ProxyError::Parse("VLESS packet incomplete".to_string()));
            }
            let domain_len = data[len_offset] as usize;
            let domain_offset = len_offset + 1;
            if data.len() < domain_offset + domain_len {
                return Err(ProxyError::Parse("VLESS packet incomplete".to_string()));
            }
            let host = String::from_utf8_lossy(&data[domain_offset..domain_offset + domain_len]).to_string();
            (host, domain_offset + domain_len)
        }
        ATYPE_IPV6 => {
            let addr_offset = atype_offset + 1;
            if data.len() < addr_offset + 16 {
                return Err(ProxyError::Parse("VLESS packet incomplete".to_string()));
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
        _ => return Err(ProxyError::Parse(format!("Unknown address type: {}", address_type))),
    };

    // Parse port
    if data.len() < port_offset + 2 {
        return Err(ProxyError::Parse("VLESS packet incomplete".to_string()));
    }
    let port = u16::from_be_bytes([data[port_offset], data[port_offset + 1]]);

    Ok(VlessRequest {
        uuid,
        command,
        address_type,
        host,
        port,
    })
}
