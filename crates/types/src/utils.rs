use alloy_primitives::hex::{self, encode};
use anyhow::{bail, Result};
use std::str::from_utf8;

const U256_BYTE_SIZE: usize = 32;

pub fn to_u256_bytes(input: &str) -> Result<[u8; U256_BYTE_SIZE]> {
    let bytes = input.as_bytes();
    if bytes.len() > U256_BYTE_SIZE {
        bail!("Input is too long");
    }
    let mut fixed_bytes = [0u8; U256_BYTE_SIZE];
    fixed_bytes[U256_BYTE_SIZE - bytes.len()..].copy_from_slice(bytes);
    Ok(fixed_bytes)
}

pub fn bytes32_to_str(bytes32: &[u8]) -> Result<String> {
    let mut hex_str = encode(bytes32); // Convert to hex string
    while hex_str.ends_with("00") {
        hex_str.pop(); // Remove one '0'
        hex_str.pop(); // Remove the second '0'
    }

    if hex_str.len() % 2 != 0 {
        hex_str.push('0'); // Pad with '0' if necessary
    }

    let bytes = hex::decode(&hex_str)?; // Convert back to bytes
    let result = from_utf8(&bytes)?.to_string(); // Convert to UTF-8 string

    Ok(result)
}
