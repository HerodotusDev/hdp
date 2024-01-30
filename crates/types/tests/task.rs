use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::U256;
use types::{datalake_base::DatalakeBase, utils::to_u256_bytes};

#[test]
fn test_header_encode() {
    let datalake_header_type: DynSolType = "(uint256,uint256,bytes)".parse().unwrap();
    let datalake = DatalakeBase::new("a1234", Vec::new);

    let identifier_value = DynSolValue::Uint(
        U256::from_str_radix(&datalake.identifier, 16).unwrap(), // Consider error handling
        256,
    );

    let aggregate_fn_id = "avg".to_string();
    let fixed_bytes = to_u256_bytes(&aggregate_fn_id).expect("Failed to convert to U256 bytes");
    let aggregate_fn_id_value = DynSolValue::Uint(U256::from_be_bytes(fixed_bytes), 256);

    let aggregate_fn_ctx = "0x1234".to_string();
    let aggregate_fn_ctx_value = DynSolValue::Bytes(aggregate_fn_ctx.into_bytes());

    let header_tuple_value = DynSolValue::Tuple(vec![
        identifier_value,
        aggregate_fn_id_value,
        aggregate_fn_ctx_value,
    ]);

    let datalake_header_encode = header_tuple_value.abi_encode();
    let decoded = datalake_header_type
        .abi_decode(&datalake_header_encode)
        .unwrap(); // Consider error handling

    assert_eq!(decoded, header_tuple_value);
}
