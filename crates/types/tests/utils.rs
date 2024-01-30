use types::utils::to_u256_bytes;

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
