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

pub fn bytes_to_fixed_bytes32(bytes: &[u8]) -> FixedBytes<32> {
    // Initialize a 32-byte array filled with 0
    let mut bytes32: [u8; 32] = [0; 32];

    // Calculate the start position to copy the input bytes at the end of bytes32
    let start_pos = 32 - bytes.len();

    // Copy the input bytes to the right position in the bytes32 array
    bytes32[start_pos..].copy_from_slice(bytes);

    FixedBytes::from(bytes32)
}

/// Convert a byte array into a hex string
pub fn bytes_to_hex_string(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

pub fn hex_string_to_bytes(hex_string: &str) -> Result<Vec<u8>> {
    let hex_string = hex_string.trim_start_matches("0x");
    Ok(hex::decode(hex_string)?)
}

/// Get the last byte of a byte array as a u8
pub fn last_byte_to_u8(bytes: &[u8]) -> u8 {
    *bytes.last().unwrap_or(&0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{hex::FromHex, FixedBytes};

    #[test]
    fn test_bytes32_to_str() {
        let value = "0x6d61780000000000000000000000000000000000000000000000000000000000";
        let input = FixedBytes::from_hex(value).unwrap();
        let result = fixed_bytes_str_to_utf8_str(input).unwrap();
        assert_eq!(result, "max".to_string());

        let value = "0x6d696e0000000000000000000000000000000000000000000000000000000000";
        let input = FixedBytes::from_hex(value).unwrap();
        let result = fixed_bytes_str_to_utf8_str(input).unwrap();
        assert_eq!(result, "min".to_string());

        let value = "0x73756d0000000000000000000000000000000000000000000000000000000000";
        let input = FixedBytes::from_hex(value).unwrap();
        let result = fixed_bytes_str_to_utf8_str(input).unwrap();
        assert_eq!(result, "sum".to_string());

        let value = "0x6176670000000000000000000000000000000000000000000000000000000000";
        let input = FixedBytes::from_hex(value).unwrap();
        let result = fixed_bytes_str_to_utf8_str(input).unwrap();
        assert_eq!(result, "avg".to_string());
    }

    #[test]
    fn test_bytes_to_hex_string() {
        let input = [0, 0, 0, 0, 0];
        let result = bytes_to_hex_string(&input);
        assert_eq!(result, "0x0000000000");
        assert_eq!(hex_string_to_bytes(&result).unwrap(), input);

        let input = [0, 0, 0, 9, 2];
        let result = bytes_to_hex_string(&input);
        assert_eq!(result, "0x0000000902");

        let hex = "030fff";
        let input = hex_string_to_bytes(hex).unwrap();
        assert_eq!(input, [3, 15, 255]);
        let result = bytes_to_hex_string(&input);
        assert_eq!(result, "0x030fff");
        assert_eq!(hex_string_to_bytes(&result).unwrap(), input);
    }

    #[test]
    fn test_last_byte_to_u8() {
        let input = [0, 0, 0, 0, 0];
        let result = last_byte_to_u8(&input);
        assert_eq!(result, 0);

        let input = [0, 0, 0, 9, 2];
        let result = last_byte_to_u8(&input);
        assert_eq!(result, 2);

        let input = [0, 0, 0, 9, 255];
        let result = last_byte_to_u8(&input);
        assert_eq!(result, 255);
    }
}
