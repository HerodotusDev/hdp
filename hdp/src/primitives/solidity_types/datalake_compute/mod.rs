use alloy::{
    dyn_abi::DynSolValue,
    primitives::{keccak256, B256, U256},
};
use anyhow::Result;
use datalake::envelope::BatchedDatalakeEnvelope;

use crate::{
    primitives::aggregate_fn::{integer::Operator, AggregationFunction},
    primitives::task::datalake::{
        compute::Computation, envelope::DatalakeEnvelope, DatalakeCompute,
    },
};

use self::compute::BatchedComputation;

use super::traits::{BatchedDatalakeComputeCodecs, Codecs, DatalakeCodecs, DatalakeComputeCodecs};

mod compute;
mod datalake;

impl DatalakeComputeCodecs for DatalakeCompute {
    fn decode(serialized_datalake: &[u8], serialized_task: &[u8]) -> Result<DatalakeCompute> {
        let decoded_datalake = DatalakeEnvelope::decode(serialized_datalake)?;
        let decoded_compute = Computation::decode(serialized_task)?;
        Ok(DatalakeCompute::new(decoded_datalake, decoded_compute))
    }

    fn commit(&self) -> B256 {
        let encoded_datalake = self.encode().unwrap();
        keccak256(encoded_datalake)
    }

    fn encode(&self) -> Result<Vec<u8>> {
        let identifier_value = DynSolValue::FixedBytes(self.datalake.commit(), 32);

        let aggregate_fn_id = DynSolValue::Uint(
            U256::from(AggregationFunction::to_index(&self.compute.aggregate_fn_id)),
            8,
        );

        let operator = DynSolValue::Uint(
            U256::from(Operator::to_index(&self.compute.aggregate_fn_ctx.operator)),
            8,
        );
        let value_to_compare =
            DynSolValue::Uint(self.compute.aggregate_fn_ctx.value_to_compare, 32);

        let tuple_value = DynSolValue::Tuple(vec![
            identifier_value,
            aggregate_fn_id,
            operator,
            value_to_compare,
        ]);

        Ok(tuple_value.abi_encode())
    }
}

pub type BatchedDatalakeCompute = Vec<DatalakeCompute>;

impl BatchedDatalakeComputeCodecs for BatchedDatalakeCompute {
    fn decode(
        serialized_datalakes: &[u8],
        serialized_computes: &[u8],
    ) -> Result<Vec<DatalakeCompute>> {
        // decode datalakes and tasks
        let decoded_datalakes = BatchedDatalakeEnvelope::decode(serialized_datalakes)?;
        let decoded_computes = BatchedComputation::decode(serialized_computes)?;

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
    fn encode(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let (datalakes, computes): (BatchedDatalakeEnvelope, BatchedComputation) = self
            .iter()
            .map(|datalake_compute| {
                (
                    datalake_compute.datalake.clone(),
                    datalake_compute.compute.clone(),
                )
            })
            .unzip();
        let encoded_datalakes = datalakes.encode()?;
        let encoded_computes = computes.encode()?;
        Ok((encoded_datalakes, encoded_computes))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{
        primitives::aggregate_fn::FunctionContext,
        primitives::task::datalake::{
            block_sampled::{BlockSampledCollection, BlockSampledDatalake},
            transactions::{IncludedTypes, TransactionsCollection, TransactionsInBlockDatalake},
        },
    };

    use super::*;

    #[test]
    fn test_block_sampled_commit() {
        let datalake_compute = DatalakeCompute {
            datalake: DatalakeEnvelope::BlockSampled(BlockSampledDatalake {
                chain_id: 11155111,
                block_range_start: 5858987,
                block_range_end: 5858997,
                increment: 2,
                sampled_property: BlockSampledCollection::Header(
                    crate::primitives::task::datalake::block_sampled::HeaderField::ExcessBlobGas,
                ),
            }),
            compute: Computation {
                aggregate_fn_id: AggregationFunction::SLR,
                aggregate_fn_ctx: FunctionContext {
                    operator: Operator::None,
                    value_to_compare: U256::from_str("10000000").unwrap(),
                },
            },
        };

        assert_eq!(
            datalake_compute.commit(),
            B256::from_str("0x8b610552e6badf24b2d4c44ebd7281d64bb9bdcbb229e29d8be53d4917989ee9")
                .unwrap()
        )
    }

    #[test]
    fn test_transactions_commit() {
        let datalake_compute = DatalakeCompute {
            datalake: DatalakeEnvelope::TransactionsInBlock(TransactionsInBlockDatalake {
                chain_id: 11155111,
                target_block: 5605816,
                start_index: 12,
                end_index: 53,
                increment: 1,
                included_types: IncludedTypes::from_be_bytes([0, 0, 1, 1]),
                sampled_property: TransactionsCollection::TranasactionReceipts(
                    crate::primitives::task::datalake::transactions::TransactionReceiptField::Success,
                ),
            }),
            compute: Computation {
                aggregate_fn_id: AggregationFunction::SLR,
                aggregate_fn_ctx: FunctionContext {
                    operator: Operator::None,
                    value_to_compare: U256::from_str("50").unwrap(),
                },
            },
        };

        assert_eq!(
            datalake_compute.commit(),
            B256::from_str("0x4db0b8b60e30ad6300ad54021c0ee6b250d986b33d07e4a7b28631850b5752ec")
                .unwrap()
        )
    }
}
