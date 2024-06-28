use alloy::{
    dyn_abi::DynSolValue,
    primitives::{keccak256, B256, U256},
};
use anyhow::Result;
use datalake::envelope::BatchedDatalakeEnvelope;

use crate::{
    aggregate_fn::{integer::Operator, AggregationFunction},
    datalake::{compute::Computation, envelope::DatalakeEnvelope, DatalakeCompute},
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
