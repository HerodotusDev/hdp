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

pub fn u64_to_fixed_bytes32(value: u64) -> FixedBytes<32> {
    let mut buffer = [0u8; 32];
    // Convert the u64 value to an array of bytes in little-endian format
    let value_bytes = value.to_le_bytes();

    // remove 0 from the beginning
    let value_bytes = value_bytes
        .iter()
        .skip_while(|&x| *x == 0)
        .copied()
        .collect::<Vec<u8>>();

    let end: usize = value_bytes.len();
    buffer[0..end].copy_from_slice(&value_bytes);

    FixedBytes::from(buffer)
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
    use crate::datalake::output::{split_little_endian_hex_into_parts, Uint256};

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

    #[test]
    fn test_u64_to_fixed_bytes32() {
        let value = 0u64;
        let result = u64_to_fixed_bytes32(value);
        assert_eq!(
            result,
            FixedBytes::from_hex(
                "0x0000000000000000000000000000000000000000000000000000000000000000"
            )
            .unwrap()
        );

        let value = 20u64;
        let result = u64_to_fixed_bytes32(value);
        assert_eq!(
            result,
            FixedBytes::from_hex(
                "0x1400000000000000000000000000000000000000000000000000000000000000"
            )
            .unwrap()
        );

        let value = 10000u64;
        let result = u64_to_fixed_bytes32(value);
        assert_eq!(
            result,
            FixedBytes::from_hex(
                "0x1027000000000000000000000000000000000000000000000000000000000000"
            )
            .unwrap()
        );

        let value = 33226u64;
        let result = u64_to_fixed_bytes32(value);
        assert_eq!(
            result,
            FixedBytes::from_hex(
                "0xca81000000000000000000000000000000000000000000000000000000000000"
            )
            .unwrap()
        );

        assert_eq!(
            split_little_endian_hex_into_parts(
                "0xca81000000000000000000000000000000000000000000000000000000000000"
            ),
            Uint256 {
                low: "0x000000000000000000000000000081ca".to_string(),
                high: "0x00000000000000000000000000000000".to_string()
            }
        )
    }
}
