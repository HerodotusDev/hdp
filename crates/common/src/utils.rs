use alloy_primitives::hex::{self};
use alloy_primitives::FixedBytes;
use anyhow::Result;

/// Convert a `FixedBytes<32>` which originally encoded from utf8 string into original utf8 string value
pub fn fixed_bytes_str_to_utf8_str(input_bytes: FixedBytes<32>) -> Result<String> {
    // Find the position of the first zero byte, if any, to trim the padding.
    let trim_position = input_bytes
        .iter()
        .position(|&x| x == 0)
        .unwrap_or(input_bytes.len());
    Ok(String::from_utf8(input_bytes[..trim_position].to_vec())?)
}

/// Convert a utf8 string into `FixedBytes<32>` by padding zero bytes to the end
pub fn utf8_str_to_fixed_bytes32(s: &str) -> FixedBytes<32> {
    let mut fixed_bytes = [0u8; 32];
    let bytes = s.as_bytes();
    for (i, &byte) in bytes.iter().enumerate().take(32) {
        fixed_bytes[i] = byte;
    }

    FixedBytes::from(fixed_bytes)
}

/// Convert a byte array into a hex string
pub fn bytes_to_hex_string(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

/// Get the last byte of a byte array as a u8
pub fn last_byte_to_u8(bytes: &[u8]) -> u8 {
    *bytes.last().unwrap_or(&0)
}
