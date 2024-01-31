use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{hex::FromHex, U256};
use types::{
    datalake::{compiler::test_closer, datalake_base::DatalakeBase},
    task::ComputationalTask,
    utils::to_u256_bytes,
};

#[test]
fn test_header_encode() {
    let datalake_header_type: DynSolType = "(uint256,uint256,bytes)".parse().unwrap();
    let datalake = DatalakeBase::new("a1234", test_closer);

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

#[test]
fn test_task_from_serialized() {
    let serialized_tasks_batch = "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000060617667000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006073756d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000606d696e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000606d6178000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000";
    let tasks_type: DynSolType = "bytes[]".parse().unwrap();
    let bytes = Vec::from_hex(serialized_tasks_batch).expect("Invalid hex string");
    let serialized_tasks = tasks_type.abi_decode(&bytes).unwrap();
    let mut computational_task_result = Vec::new();

    if let Some(tasks) = serialized_tasks.as_array() {
        for task in tasks {
            let computational_task =
                ComputationalTask::from_serialized(task.as_bytes().unwrap()).unwrap();
            computational_task_result.push(computational_task);
        }
    }

    assert_eq!(computational_task_result.len(), 4);
    assert_eq!(
        computational_task_result[0].aggregate_fn_id,
        "avg".to_string()
    );
    assert_eq!(computational_task_result[0].aggregate_fn_ctx, None);
    assert_eq!(
        computational_task_result[1].aggregate_fn_id,
        "sum".to_string()
    );
    assert_eq!(computational_task_result[1].aggregate_fn_ctx, None);
    assert_eq!(
        computational_task_result[2].aggregate_fn_id,
        "min".to_string()
    );
    assert_eq!(computational_task_result[2].aggregate_fn_ctx, None);
    assert_eq!(
        computational_task_result[3].aggregate_fn_id,
        "max".to_string()
    );
    assert_eq!(computational_task_result[3].aggregate_fn_ctx, None);
}
