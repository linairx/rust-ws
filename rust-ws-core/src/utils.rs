use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use sha2::{Digest, Sha224};

/// Generate SHA224 hash and return as hex string
pub fn sha224_hash(input: &str) -> String {
    let mut hasher = Sha224::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

/// Base64 encode string
pub fn base64_encode(input: &str) -> String {
    BASE64.encode(input)
}

/// Base64 decode string
pub fn base64_decode(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    BASE64.decode(input)
}

/// Read a u16 from big-endian bytes
pub fn read_u16_be(bytes: &[u8]) -> u16 {
    u16::from_be_bytes([bytes[0], bytes[1]])
}

/// Read a u8 from bytes
pub fn read_u8(bytes: &[u8], offset: usize) -> u8 {
    bytes[offset]
}

/// Read a variable length string from bytes
pub fn read_string(bytes: &[u8], offset: usize, len: usize) -> Result<&str, std::str::Utf8Error> {
    std::str::from_utf8(&bytes[offset..offset + len])
}
