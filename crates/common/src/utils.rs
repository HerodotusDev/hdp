use alloy_primitives::hex::{self};
use alloy_primitives::keccak256;
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

pub fn bytes32_to_utf8_str(bytes32: &[u8]) -> Result<String> {
    // Find the position of the first zero byte, if any, to trim the padding.
    let end_pos = bytes32
        .iter()
        .position(|&x| x == 0)
        .unwrap_or(bytes32.len());
    // Convert directly to UTF-8 string without unnecessary hex conversion.
    Ok(from_utf8(&bytes32[..end_pos]).map(|s| s.to_string())?)
}

pub fn utf8_str_to_fixed_bytes32(s: &str) -> [u8; 32] {
    let mut fixed_bytes = [0u8; 32];
    let bytes = s.as_bytes();
    for (i, &byte) in bytes.iter().enumerate().take(32) {
        fixed_bytes[i] = byte;
    }

    fixed_bytes
}

pub fn bytes_to_hex_string(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

pub fn last_byte_to_u8(bytes: &[u8]) -> u8 {
    *bytes.last().unwrap_or(&0)
}

pub fn rlp_string_to_block_hash(rlp_string: &str) -> Result<String> {
    Ok(keccak256(hex::decode(rlp_string)?).to_string())
}
