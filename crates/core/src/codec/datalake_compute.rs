use alloy::dyn_abi::{DynSolType, DynSolValue};
use anyhow::{Ok, Result};
use hdp_primitives::{
    datalake::{
        block_sampled::BlockSampledDatalake,
        datalake_type::DatalakeType,
        envelope::DatalakeEnvelope,
        task::{Computation, DatalakeCompute},
        transactions::TransactionsInBlockDatalake,
        Datalake,
    },
    utils::last_byte_to_u8,
};
use tracing::info;

#[derive(Default)]
struct DatalakeCodec {}

impl DatalakeCodec {
    pub fn new() -> Self {
        Self {}
    }

    /// Internal function to decode a single datalake
    fn _decode_single(
        &self,
        datalake_code: &[u8],
        encoded_datalake: &[u8],
    ) -> Result<DatalakeEnvelope> {
        let decoded_datalake = match DatalakeType::from_index(last_byte_to_u8(datalake_code))? {
            DatalakeType::BlockSampled => {
                DatalakeEnvelope::BlockSampled(BlockSampledDatalake::decode(encoded_datalake)?)
            }
            DatalakeType::TransactionsInBlock => DatalakeEnvelope::Transactions(
                TransactionsInBlockDatalake::decode(encoded_datalake)?,
            ),
        };

        Ok(decoded_datalake)
    }

    /// Decode a single datalake
    fn decode_single(&self, serialized_datalake: &[u8]) -> Result<DatalakeEnvelope> {
        let datalake_code = serialized_datalake.chunks(32).next().unwrap();
        Ok(self._decode_single(datalake_code, serialized_datalake)?)
    }

    /// Decode a batch of datalakes
    fn decode_batch(&self, serialized_datalakes_batch: &[u8]) -> Result<Vec<DatalakeEnvelope>> {
        let datalakes_type: DynSolType = "bytes[]".parse()?;
        println!(
            "Serialized datalakes batch: {:#?}",
            serialized_datalakes_batch
        );
        let serialized_datalakes = datalakes_type.abi_decode(serialized_datalakes_batch)?;
        println!("Serialized datalakes: {:#?}", serialized_datalakes);
        let mut decoded_datalakes = Vec::new();

        if let Some(datalakes) = serialized_datalakes.as_array() {
            for datalake in datalakes {
                let datalake_as_bytes =
                    datalake.as_bytes().expect("Cannot get bytes from datalake");
                let datalake_code = datalake_as_bytes.chunks(32).next().unwrap();
                decoded_datalakes.push(self._decode_single(datalake_code, datalake_as_bytes)?);
            }
        }

        Ok(decoded_datalakes)
    }

    pub fn encode_single(&self, datalake: DatalakeEnvelope) -> Result<Vec<u8>> {
        let encoded_datalake = match datalake {
            DatalakeEnvelope::BlockSampled(block_sampled_datalake) => {
                block_sampled_datalake.encode()?
            }
            DatalakeEnvelope::Transactions(transactions_datalake) => {
                transactions_datalake.encode()?
            }
        };
        Ok(encoded_datalake)
    }

    pub fn encode_batch(&self, datalakes: Vec<DatalakeEnvelope>) -> Result<Vec<u8>> {
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
            encoded_datalakes.push(DynSolValue::Bytes(encoded_datalake));
        }

        let array_encoded_datalakes = DynSolValue::Array(encoded_datalakes);
        let encoded_datalakes = array_encoded_datalakes.abi_encode();
        Ok(encoded_datalakes)
    }
}

#[derive(Default)]
struct ComputeCodec {}

impl ComputeCodec {
    pub fn new() -> Self {
        Self {}
    }

    fn _decode_single(&self, serialized_task: &[u8]) -> Result<Computation> {
        let computation = Computation::decode_not_filled_task(serialized_task)?;
        Ok(computation)
    }

    /// Decode a single computation
    pub fn decode_single(&self, serialized_task: &[u8]) -> Result<Computation> {
        Ok(self._decode_single(serialized_task)?)
    }

    /// Decode a batch of computations
    pub fn decode_batch(&self, serialized_tasks_batch: &[u8]) -> Result<Vec<Computation>> {
        let tasks_type: DynSolType = "bytes[]".parse()?;

        let serialized_tasks = tasks_type.abi_decode(serialized_tasks_batch)?;

        let mut decoded_tasks = Vec::new();
        if let Some(tasks) = serialized_tasks.as_array() {
            for task in tasks {
                decoded_tasks.push(
                    self._decode_single(task.as_bytes().expect("Cannot get bytes from task"))?,
                );
            }
        }

        Ok(decoded_tasks)
    }

    /// Encode batch of computations
    pub fn encode_batch(&self, tasks: Vec<Computation>) -> Result<Vec<u8>> {
        let mut encoded_tasks: Vec<DynSolValue> = Vec::new();

        for task in tasks {
            let encoded_task = task.encode()?;
            encoded_tasks.push(DynSolValue::Bytes(encoded_task));
        }

        let array_encoded_tasks = DynSolValue::Array(encoded_tasks);
        let encoded_tasks = array_encoded_tasks.abi_encode();
        Ok(encoded_tasks)
    }

    /// Encode single computation
    fn encode_single(&self, task: Computation) -> Result<Vec<u8>> {
        Ok(task.encode()?)
    }
}

#[derive(Default)]
pub struct DatalakeComputeCodec {
    datalake_codec: DatalakeCodec,
    compute_codec: ComputeCodec,
}

impl DatalakeComputeCodec {
    pub fn new() -> Self {
        Self {
            datalake_codec: DatalakeCodec::new(),
            compute_codec: ComputeCodec::new(),
        }
    }

    pub fn decode_single(
        &self,
        serialized_datalake: &[u8],
        serialized_task: &[u8],
    ) -> Result<DatalakeCompute> {
        let decoded_datalake = self.datalake_codec.decode_single(serialized_datalake)?;
        let decoded_compute = self.compute_codec.decode_single(serialized_task)?;
        info!("Decoded compute: \n{:?}\n", decoded_compute);
        info!("Decoded datalake: \n{:?}\n", decoded_datalake);
        Ok(DatalakeCompute::new(decoded_datalake, decoded_compute))
    }

    pub fn decode_batch(
        &self,
        serialized_datalakes_batch: &[u8],
        serialized_tasks_batch: &[u8],
    ) -> Result<Vec<DatalakeCompute>> {
        // decode datalakes and tasks
        let decoded_datalakes = self
            .datalake_codec
            .decode_batch(serialized_datalakes_batch)?;
        let decoded_computes = self.compute_codec.decode_batch(serialized_tasks_batch)?;
        // check if the number of datalakes and tasks are the same
        if decoded_datalakes.len() != decoded_computes.len() {
            return Err(anyhow::anyhow!(
                "Number of datalakes and tasks are not the same"
            ));
        }

        // combine datalakes and tasks into DatalakeCompute
        let mut decoded_datalakes_compute = Vec::new();
        for (datalake, compute) in decoded_datalakes
            .into_iter()
            .zip(decoded_computes.into_iter())
        {
            decoded_datalakes_compute.push(DatalakeCompute::new(datalake, compute));
        }

        Ok(decoded_datalakes_compute)
    }

    pub fn encode_single(&self, datalake_compute: DatalakeCompute) -> Result<(Vec<u8>, Vec<u8>)> {
        let encoded_datalake = self
            .datalake_codec
            .encode_single(datalake_compute.datalake)?;
        let encoded_compute = self.compute_codec.encode_single(datalake_compute.compute)?;
        Ok((encoded_datalake, encoded_compute))
    }

    pub fn encode_batch(
        &self,
        datalakes_compute: Vec<DatalakeCompute>,
    ) -> Result<(Vec<u8>, Vec<u8>)> {
        let (datalakes, computes) = datalakes_compute
            .into_iter()
            .map(|datalake_compute| (datalake_compute.datalake, datalake_compute.compute))
            .unzip();

        let encoded_datalakes = self.datalake_codec.encode_batch(datalakes)?;
        let encoded_computes = self.compute_codec.encode_batch(computes)?;

        Ok((encoded_datalakes, encoded_computes))
    }
}

#[cfg(test)]
mod tests {
    use alloy::{hex, primitives::Address};
    use hdp_primitives::{
        aggregate_fn::{AggregationFunction, FunctionContext},
        datalake::block_sampled::{AccountField, BlockSampledCollection, HeaderField},
    };
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_compute_decoder() {
        let compute_decoder = ComputeCodec::new();

        // Note: all task's datalake is None
        let original_tasks = vec![
            Computation::new("avg", None),
            Computation::new("sum", None),
            Computation::new("min", None),
            Computation::new("max", None),
        ];

        let encoded_tasks = compute_decoder.encode_batch(original_tasks).unwrap();
        let decoded_tasks = compute_decoder.decode_batch(&encoded_tasks).unwrap();

        assert_eq!(decoded_tasks.len(), 4);
        assert_eq!(decoded_tasks[0].aggregate_fn_id, AggregationFunction::AVG);
        assert_eq!(
            decoded_tasks[0].aggregate_fn_ctx,
            FunctionContext::default()
        );

        assert_eq!(decoded_tasks[1].aggregate_fn_id, AggregationFunction::SUM);
        assert_eq!(
            decoded_tasks[1].aggregate_fn_ctx,
            FunctionContext::default()
        );

        assert_eq!(decoded_tasks[2].aggregate_fn_id, AggregationFunction::MIN);
        assert_eq!(
            decoded_tasks[2].aggregate_fn_ctx,
            FunctionContext::default()
        );

        assert_eq!(decoded_tasks[3].aggregate_fn_id, AggregationFunction::MAX);
        assert_eq!(
            decoded_tasks[3].aggregate_fn_ctx,
            FunctionContext::default()
        );
    }

    #[test]
    fn test_block_datalake_decoder() {
        let datalake_decoder = DatalakeCodec::new();
        let batched_block_datalake = hex::decode("0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f000000000000000000000000000000000000000000000000000000000000").unwrap();
        let decoded_datalakes = datalake_decoder
            .decode_batch(&batched_block_datalake)
            .unwrap();

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
            datalake_decoder.encode_batch(decoded_datalakes).unwrap(),
            batched_block_datalake
        );
    }

    #[test]
    fn test_block_datalake_decoder_for_account() {
        let datalake_decoder = DatalakeCodec::new();
        let batched_block_datalake = hex::decode("0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004b902400000000000000000000000000000000000000000000000000000000004b9027000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000016020a4de450feb156a2a51ed159b2fb99da26e5f3a30000000000000000000000").unwrap();
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
            datalake_decoder.encode_batch(datalakes.clone()).unwrap(),
            batched_block_datalake
        );
        assert_eq!(
            datalake_decoder
                .decode_batch(&batched_block_datalake)
                .unwrap(),
            datalakes
        );
    }

    #[test]
    fn test_block_massive_datalake_decoder() {
        let datalake_decoder = DatalakeCodec::new();
        let batched_block_datalake = hex::decode("0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009ead1800000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009ead1800000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009ead1800000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009ead1800000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f000000000000000000000000000000000000000000000000000000000000").unwrap();
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
        let decoded_datalakes = datalake_decoder
            .decode_batch(&batched_block_datalake)
            .unwrap();
        assert_eq!(decoded_datalakes.len(), 4);

        assert_eq!(
            datalake_decoder.encode_batch(batched_datalakes).unwrap(),
            batched_block_datalake
        );
    }

    #[test]
    fn test_transaction_datalakes_encoder() {
        let datalake_decoder = DatalakeCodec::new();
        let transaction_datalake1 = TransactionsInBlockDatalake::new(
            100000,
            "tx.nonce".to_string(),
            1,
            100,
            1,
            &[0, 0, 1, 1],
        )
        .unwrap();

        let transaction_datalake2 = TransactionsInBlockDatalake::new(
            100000,
            "tx.access_list".to_string(),
            1,
            100,
            1,
            &[0, 0, 1, 1],
        )
        .unwrap();

        let datalakes = vec![
            DatalakeEnvelope::Transactions(transaction_datalake1),
            DatalakeEnvelope::Transactions(transaction_datalake2),
        ];
        let encoded_datalakes = datalake_decoder.encode_batch(datalakes).unwrap();

        assert_eq!(encoded_datalakes, hex::decode("00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000186a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000010100000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000201000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000186a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000010100000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000002010a000000000000000000000000000000000000000000000000000000000000").unwrap());
    }

    #[test]
    fn test_transaction_datalake_decoder() {
        let datalake_decoder = DatalakeCodec::new();
        let encoded_datalake = hex::decode("00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000186a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000010100000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000201000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000186a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000010100000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000002010a000000000000000000000000000000000000000000000000000000000000").unwrap();
        let decoded_datalake = datalake_decoder.decode_batch(&encoded_datalake).unwrap();
        assert_eq!(decoded_datalake.len(), 2);

        let transaction_datalake1 = TransactionsInBlockDatalake::new(
            100000,
            "tx.nonce".to_string(),
            1,
            100,
            1,
            &[0, 0, 1, 1],
        )
        .unwrap();

        let transaction_datalake2 = TransactionsInBlockDatalake::new(
            100000,
            "tx.access_list".to_string(),
            1,
            100,
            1,
            &[0, 0, 1, 1],
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
        let datalake_decoder = DatalakeCodec::new();
        let transaction_datalake1 = TransactionsInBlockDatalake::new(
            100000,
            "tx_receipt.success".to_string(),
            1,
            100,
            1,
            &[0, 0, 1, 1],
        )
        .unwrap();

        let transaction_datalake2 = TransactionsInBlockDatalake::new(
            100000,
            "tx_receipt.bloom".to_string(),
            1,
            100,
            1,
            &[0, 0, 1, 1],
        )
        .unwrap();

        let datalakes = vec![
            DatalakeEnvelope::Transactions(transaction_datalake1),
            DatalakeEnvelope::Transactions(transaction_datalake2),
        ];
        let encoded_datalakes = datalake_decoder.encode_batch(datalakes).unwrap();

        assert_eq!(encoded_datalakes, hex::decode("0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000186a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000010100000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000202000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000186a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000010100000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000020203000000000000000000000000000000000000000000000000000000000000").unwrap())
    }

    #[test]
    fn test_transaction_datalake_decoder_receipt() {
        let datalake_decoder = DatalakeCodec::new();
        let encoded_datalake = hex::decode("0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000186a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000010100000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000202000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000186a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000010100000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000020203000000000000000000000000000000000000000000000000000000000000").unwrap();
        let decoded_datalake = datalake_decoder.decode_batch(&encoded_datalake).unwrap();
        assert_eq!(decoded_datalake.len(), 2);

        let transaction_datalake1 = TransactionsInBlockDatalake::new(
            100000,
            "tx_receipt.success".to_string(),
            1,
            100,
            1,
            &[0, 0, 1, 1],
        )
        .unwrap();

        let transaction_datalake2 = TransactionsInBlockDatalake::new(
            100000,
            "tx_receipt.bloom".to_string(),
            1,
            100,
            1,
            &[0, 0, 1, 1],
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
