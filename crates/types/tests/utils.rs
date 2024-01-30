use alloy_primitives::hex::FromHex;
use types::utils::{bytes32_to_str, to_u256_bytes};

#[test]
fn test_to_u256_bytes() {
    let input = "0x1234";
    let expected = [0u8; 32];
    let mut expected_vec = expected.to_vec();
    expected_vec[26..].copy_from_slice("0x1234".as_bytes());
    let result_vec = to_u256_bytes(input).unwrap().to_vec();
    assert_eq!(result_vec, expected_vec);

    let input_string = "avg".to_string();
    let expected = [0u8; 32];
    let mut expected_vec = expected.to_vec();
    expected_vec[29..].copy_from_slice("avg".as_bytes());
    let result_vec = to_u256_bytes(&input_string).unwrap().to_vec();
    assert_eq!(result_vec, expected_vec);
}

#[test]
fn test_bytes32_to_str() {
    let value = "0x6d61780000000000000000000000000000000000000000000000000000000000";
    let input = Vec::from_hex(value).expect("Invalid hex string");
    let result = bytes32_to_str(&input).unwrap();
    assert_eq!(result, "max".to_string());

    let value = "0x6d696e0000000000000000000000000000000000000000000000000000000000";
    let input = Vec::from_hex(value).expect("Invalid hex string");
    let result = bytes32_to_str(&input).unwrap();
    assert_eq!(result, "min".to_string());

    let value = "0x73756d0000000000000000000000000000000000000000000000000000000000";
    let input = Vec::from_hex(value).expect("Invalid hex string");
    let result = bytes32_to_str(&input).unwrap();
    assert_eq!(result, "sum".to_string());

    let value = "0x6176670000000000000000000000000000000000000000000000000000000000";
    let input = Vec::from_hex(value).expect("Invalid hex string");
    let result = bytes32_to_str(&input).unwrap();
    assert_eq!(result, "avg".to_string());
}
