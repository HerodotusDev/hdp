use alloy_primitives::hex::{self, encode};
use alloy_primitives::keccak256;
use anyhow::{bail, Result};
use std::fmt::Write;
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

pub fn bytes32_to_utf8_str(bytes32: &[u8]) -> Result<String> {
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

pub fn bytes_to_hex_string(bytes: &[u8]) -> String {
    let last_non_zero = bytes.iter().rposition(|&b| b != 0).unwrap_or(0);

    let hex_str = bytes[..=last_non_zero]
        .iter()
        .fold(String::new(), |mut acc, &byte| {
            write!(acc, "{:02x}", byte).expect("Failed to write");
            acc
        });

    format!("0x{}", hex_str)
}

pub fn last_byte_to_u8(bytes: &[u8]) -> u8 {
    *bytes.last().unwrap_or(&0)
}

pub fn rlp_string_to_block_hash(rlp_string: &str) -> Result<String> {
    Ok(keccak256(hex::decode(rlp_string)?).to_string())
}
