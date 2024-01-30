use anyhow::{bail, Result};

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
