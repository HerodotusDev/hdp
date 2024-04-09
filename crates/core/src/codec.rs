use crate::task::ComputationalTask;
use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::hex::FromHex;
use anyhow::{bail, Ok, Result};
use hdp_primitives::{
    datalake::{
        block_sampled::BlockSampledDatalake, datalake_type::DatalakeType,
        envelope::DatalakeEnvelope, transactions::TransactionsDatalake, Datalake,
    },
    utils::{bytes_to_hex_string, last_byte_to_u8},
};

/// Decode a batch of tasks
pub fn tasks_decoder(serialized_tasks_batch: String) -> Result<Vec<ComputationalTask>> {
    let tasks_type: DynSolType = "bytes[]".parse()?;
    let bytes = Vec::from_hex(serialized_tasks_batch).expect("Invalid hex string");

    let serialized_tasks = tasks_type.abi_decode(&bytes)?;

    let mut decoded_tasks = Vec::new();

    if let Some(tasks) = serialized_tasks.as_array() {
        for task in tasks {
            let computational_task =
                ComputationalTask::decode_not_filled_task(task.as_bytes().unwrap())?;
            decoded_tasks.push(computational_task);
        }
    }

    Ok(decoded_tasks)
}

/// Decode a single task
pub fn task_decoder(serialized_task: String) -> Result<ComputationalTask> {
    let computational_task = ComputationalTask::decode_not_filled_task(serialized_task.as_bytes())?;
    Ok(computational_task)
}

/// Decode a batch of datalakes
pub fn datalakes_decoder(serialized_datalakes_batch: String) -> Result<Vec<DatalakeEnvelope>> {
    let datalakes_type: DynSolType = "bytes[]".parse()?;
    let bytes = Vec::from_hex(serialized_datalakes_batch).expect("Invalid hex string");
    let serialized_datalakes = datalakes_type.abi_decode(&bytes)?;

    let mut decoded_datalakes = Vec::new();

    if let Some(datalakes) = serialized_datalakes.as_array() {
        for datalake in datalakes {
            let datalake_code = datalake.as_bytes().unwrap().chunks(32).next().unwrap();
            let datalake_string = bytes_to_hex_string(datalake.as_bytes().unwrap());

            let decoded_datalake = match DatalakeType::from_index(last_byte_to_u8(datalake_code))? {
                DatalakeType::BlockSampled => {
                    DatalakeEnvelope::BlockSampled(BlockSampledDatalake::decode(&datalake_string)?)
                }
                DatalakeType::DynamicLayout => bail!("Unsupported datalake type"),
                DatalakeType::Transactions => {
                    DatalakeEnvelope::Transactions(TransactionsDatalake::decode(&datalake_string)?)
                }
            };

            decoded_datalakes.push(decoded_datalake);
        }
    }

    Ok(decoded_datalakes)
}

/// Decode a single datalake
pub fn datalake_decoder(serialized_datalake: String) -> Result<DatalakeEnvelope> {
    let datalake_code = serialized_datalake.as_bytes().chunks(32).next().unwrap();
    let datalake_string = bytes_to_hex_string(serialized_datalake.as_bytes());

    let decoded_datalake = match DatalakeType::from_index(last_byte_to_u8(datalake_code))? {
        DatalakeType::BlockSampled => {
            DatalakeEnvelope::BlockSampled(BlockSampledDatalake::decode(&datalake_string)?)
        }
        DatalakeType::DynamicLayout => bail!("Unsupported datalake type"),
        DatalakeType::Transactions => {
            DatalakeEnvelope::Transactions(TransactionsDatalake::decode(&datalake_string)?)
        }
    };

    Ok(decoded_datalake)
}

/// Encode a batch of datalakes
pub fn datalakes_encoder(datalakes: Vec<DatalakeEnvelope>) -> Result<String> {
    let mut encoded_datalakes: Vec<DynSolValue> = Vec::new();

    for datalake in datalakes {
        let encoded_datalake = match datalake {
            DatalakeEnvelope::BlockSampled(block_sampled_datalake) => {
                block_sampled_datalake.encode()?
            }
            DatalakeEnvelope::Transactions(transactions_datalake) => {
                transactions_datalake.encode()?
            }
        };
        let bytes = Vec::from_hex(encoded_datalake).expect("Invalid hex string");
        encoded_datalakes.push(DynSolValue::Bytes(bytes));
    }

    let array_encoded_datalakes = DynSolValue::Array(encoded_datalakes);
    let encoded_datalakes = array_encoded_datalakes.abi_encode();
    Ok(bytes_to_hex_string(&encoded_datalakes))
}

/// Encode batch of tasks
pub fn tasks_encoder(tasks: Vec<ComputationalTask>) -> Result<String> {
    let mut encoded_tasks: Vec<DynSolValue> = Vec::new();

    for task in tasks {
        let encoded_task = task.encode()?;
        let bytes = Vec::from_hex(encoded_task).expect("Invalid hex string");
        encoded_tasks.push(DynSolValue::Bytes(bytes));
    }

    let array_encoded_tasks = DynSolValue::Array(encoded_tasks);
    let encoded_tasks = array_encoded_tasks.abi_encode();
    Ok(bytes_to_hex_string(&encoded_tasks))
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use alloy_primitives::Address;
    use hdp_primitives::datalake::block_sampled::{
        AccountField, BlockSampledCollection, HeaderField,
    };

    use crate::aggregate_fn::AggregationFunction;

    use super::*;

    #[test]
    fn test_task_decoder() {
        // Note: all task's datalake is None
        let original_tasks = vec![
            ComputationalTask::new("avg", None),
            ComputationalTask::new("sum", None),
            ComputationalTask::new("min", None),
            ComputationalTask::new("max", None),
        ];

        let encoded_tasks = tasks_encoder(original_tasks).unwrap();

        let decoded_tasks = tasks_decoder(encoded_tasks).unwrap();
        assert_eq!(decoded_tasks.len(), 4);
        assert_eq!(decoded_tasks[0].aggregate_fn_id, AggregationFunction::AVG);
        assert_eq!(decoded_tasks[0].aggregate_fn_ctx, None);

        assert_eq!(decoded_tasks[1].aggregate_fn_id, AggregationFunction::SUM);
        assert_eq!(decoded_tasks[1].aggregate_fn_ctx, None);

        assert_eq!(decoded_tasks[2].aggregate_fn_id, AggregationFunction::MIN);
        assert_eq!(decoded_tasks[2].aggregate_fn_ctx, None);

        assert_eq!(decoded_tasks[3].aggregate_fn_id, AggregationFunction::MAX);
        assert_eq!(decoded_tasks[3].aggregate_fn_ctx, None);
    }

    #[test]
    fn test_block_datalake_decoder() {
        let batched_block_datalake = "0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f000000000000000000000000000000000000000000000000000000000000";
        let decoded_datalakes = datalakes_decoder(batched_block_datalake.to_string()).unwrap();
        assert_eq!(decoded_datalakes.len(), 4);
        for datalake in decoded_datalakes.clone() {
            if let DatalakeEnvelope::BlockSampled(block_datalake) = datalake {
                assert_eq!(block_datalake.block_range_start, 10399990);
                assert_eq!(block_datalake.block_range_end, 10400000);
                assert_eq!(
                    block_datalake.sampled_property,
                    BlockSampledCollection::Header(HeaderField::BaseFeePerGas)
                );
                assert_eq!(block_datalake.increment, 1);
            } else {
                panic!("Expected block datalake");
            }
        }

        assert_eq!(
            datalakes_encoder(decoded_datalakes).unwrap(),
            batched_block_datalake
        );
    }

    #[test]
    fn test_block_datalake_decoder_for_account() {
        let batched_block_datalake = "0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004b902400000000000000000000000000000000000000000000000000000000004b9027000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000016020a4de450feb156a2a51ed159b2fb99da26e5f3a30000000000000000000000";
        let block_datalake = BlockSampledDatalake::new(
            4952100,
            4952103,
            "account.0x0a4de450feb156a2a51ed159b2fb99da26e5f3a3.nonce".to_string(),
            1,
        )
        .unwrap();
        let datalakes = vec![DatalakeEnvelope::BlockSampled(block_datalake.clone())];
        assert_eq!(datalakes.len(), 1);
        for datalake in datalakes.clone() {
            if let DatalakeEnvelope::BlockSampled(block_datalake) = datalake {
                assert_eq!(block_datalake.block_range_start, 4952100);
                assert_eq!(block_datalake.block_range_end, 4952103);
                assert_eq!(
                    block_datalake.sampled_property,
                    BlockSampledCollection::Account(
                        Address::from_str("0x0a4de450feb156a2a51ed159b2fb99da26e5f3a3").unwrap(),
                        AccountField::Nonce
                    )
                );
                assert_eq!(block_datalake.increment, 1);
            } else {
                panic!("Expected block datalake");
            }
        }

        assert_eq!(
            datalakes_encoder(datalakes.clone()).unwrap(),
            batched_block_datalake
        );
        assert_eq!(
            datalakes_decoder(batched_block_datalake.to_string()).unwrap(),
            datalakes
        );
    }

    #[test]
    fn test_block_massive_datalake_decoder() {
        let batched_block_datalake = "0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009ead1800000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009ead1800000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009ead1800000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009ead1800000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f000000000000000000000000000000000000000000000000000000000000";
        let datalake_massive_block = DatalakeEnvelope::BlockSampled(
            BlockSampledDatalake::new(10399000, 10400000, "header.base_fee_per_gas".to_string(), 1)
                .unwrap(),
        );

        let batched_datalakes = vec![
            datalake_massive_block.clone(),
            datalake_massive_block.clone(),
            datalake_massive_block.clone(),
            datalake_massive_block.clone(),
        ];
        let decoded_datalakes = datalakes_decoder(batched_block_datalake.to_string()).unwrap();
        assert_eq!(decoded_datalakes.len(), 4);

        assert_eq!(
            datalakes_encoder(batched_datalakes).unwrap(),
            batched_block_datalake
        );
    }

    #[test]
    fn test_transaction_datalakes_encoder() {
        let transaction_datalake1 = TransactionsDatalake::new(
            "0xcb96AcA8719987D15aecd066B7a1Ad5D4d92fdD3".to_string(),
            0,
            3,
            "tx.nonce".to_string(),
            1,
        )
        .unwrap();

        let transaction_datalake2 = TransactionsDatalake::new(
            "0xcb96AcA8719987D15aecd066B7a1Ad5D4d92fdD3".to_string(),
            0,
            3,
            "tx.access_list".to_string(),
            1,
        )
        .unwrap();

        let datalakes = vec![
            DatalakeEnvelope::Transactions(transaction_datalake1),
            DatalakeEnvelope::Transactions(transaction_datalake2),
        ];
        let encoded_datalakes = datalakes_encoder(datalakes).unwrap();

        assert_eq!(encoded_datalakes, "0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000cb96aca8719987d15aecd066b7a1ad5d4d92fdd300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000cb96aca8719987d15aecd066b7a1ad5d4d92fdd300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000002000a000000000000000000000000000000000000000000000000000000000000")
    }

    #[test]
    fn test_transaction_datalake_decoder() {
        let encoded_datalake = "0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000cb96aca8719987d15aecd066b7a1ad5d4d92fdd300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000cb96aca8719987d15aecd066b7a1ad5d4d92fdd300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000002000a000000000000000000000000000000000000000000000000000000000000";
        let decoded_datalake = datalakes_decoder(encoded_datalake.to_string()).unwrap();
        assert_eq!(decoded_datalake.len(), 2);

        let transaction_datalake1 = TransactionsDatalake::new(
            "0xcb96AcA8719987D15aecd066B7a1Ad5D4d92fdD3".to_string(),
            0,
            3,
            "tx.nonce".to_string(),
            1,
        )
        .unwrap();

        let transaction_datalake2 = TransactionsDatalake::new(
            "0xcb96AcA8719987D15aecd066B7a1Ad5D4d92fdD3".to_string(),
            0,
            3,
            "tx.access_list".to_string(),
            1,
        )
        .unwrap();

        assert_eq!(
            decoded_datalake[0],
            DatalakeEnvelope::Transactions(transaction_datalake1)
        );
        assert_eq!(
            decoded_datalake[1],
            DatalakeEnvelope::Transactions(transaction_datalake2)
        );
    }

    #[test]
    fn test_transaction_datalakes_encoder_receipt() {
        let transaction_datalake1 = TransactionsDatalake::new(
            "0xcb96AcA8719987D15aecd066B7a1Ad5D4d92fdD3".to_string(),
            0,
            3,
            "tx_receipt.success".to_string(),
            1,
        )
        .unwrap();

        let transaction_datalake2 = TransactionsDatalake::new(
            "0xcb96AcA8719987D15aecd066B7a1Ad5D4d92fdD3".to_string(),
            0,
            3,
            "tx_receipt.bloom".to_string(),
            1,
        )
        .unwrap();

        let datalakes = vec![
            DatalakeEnvelope::Transactions(transaction_datalake1),
            DatalakeEnvelope::Transactions(transaction_datalake2),
        ];
        let encoded_datalakes = datalakes_encoder(datalakes).unwrap();

        assert_eq!(encoded_datalakes, "0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000cb96aca8719987d15aecd066b7a1ad5d4d92fdd300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000002010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000cb96aca8719987d15aecd066b7a1ad5d4d92fdd300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000020103000000000000000000000000000000000000000000000000000000000000")
    }

    #[test]
    fn test_transaction_datalake_decoder_receipt() {
        let encoded_datalake = "0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000cb96aca8719987d15aecd066b7a1ad5d4d92fdd300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000002010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000cb96aca8719987d15aecd066b7a1ad5d4d92fdd300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000020103000000000000000000000000000000000000000000000000000000000000";
        let decoded_datalake = datalakes_decoder(encoded_datalake.to_string()).unwrap();
        assert_eq!(decoded_datalake.len(), 2);

        let transaction_datalake1 = TransactionsDatalake::new(
            "0xcb96AcA8719987D15aecd066B7a1Ad5D4d92fdD3".to_string(),
            0,
            3,
            "tx_receipt.success".to_string(),
            1,
        )
        .unwrap();

        let transaction_datalake2 = TransactionsDatalake::new(
            "0xcb96AcA8719987D15aecd066B7a1Ad5D4d92fdD3".to_string(),
            0,
            3,
            "tx_receipt.bloom".to_string(),
            1,
        )
        .unwrap();

        assert_eq!(
            decoded_datalake[0],
            DatalakeEnvelope::Transactions(transaction_datalake1)
        );
        assert_eq!(
            decoded_datalake[1],
            DatalakeEnvelope::Transactions(transaction_datalake2)
        );
    }
}
