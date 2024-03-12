use alloy_primitives::{hex::FromHex, FixedBytes};
use common::utils::{bytes_to_hex_string, fixed_bytes_str_to_utf8_str, last_byte_to_u8};

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

    let input = [0, 0, 0, 9, 2];
    let result = bytes_to_hex_string(&input);
    assert_eq!(result, "0x0000000902");
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
