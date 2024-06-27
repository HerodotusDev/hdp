use alloy::{
    dyn_abi::{DynSolType, DynSolValue},
    primitives::U256,
};
use anyhow::{bail, Result};

use crate::{
    aggregate_fn::{integer::Operator, AggregationFunction, FunctionContext},
    datalake::compute::Computation,
    solidity_types::traits::Codecs,
};

pub type BatchedComputation = Vec<Computation>;

impl Codecs for BatchedComputation {
    fn encode(&self) -> Result<Vec<u8>> {
        let mut encoded_tasks: Vec<DynSolValue> = Vec::new();

        for task in self {
            let encoded_task = task.encode()?;
            encoded_tasks.push(DynSolValue::Bytes(encoded_task));
        }

        let array_encoded_tasks = DynSolValue::Array(encoded_tasks);
        let encoded_tasks = array_encoded_tasks.abi_encode();
        Ok(encoded_tasks)
    }

    fn decode(encoded: &[u8]) -> Result<Self> {
        let tasks_type: DynSolType = "bytes[]".parse()?;

        let serialized_tasks = tasks_type.abi_decode(encoded)?;

        let mut decoded_tasks = Vec::new();
        if let Some(tasks) = serialized_tasks.as_array() {
            for task in tasks {
                decoded_tasks.push(Computation::decode(
                    task.as_bytes().expect("Cannot get bytes from task"),
                )?);
            }
        }

        Ok(decoded_tasks)
    }
}

impl Codecs for Computation {
    fn decode(encoded_compute: &[u8]) -> Result<Self> {
        let aggregate_fn_type: DynSolType = "(uint8,uint8,uint256)".parse()?;
        let decoded = aggregate_fn_type.abi_decode(encoded_compute)?;

        let value = decoded.as_tuple().unwrap();

        let aggregate_fn_id = match value[0] {
            DynSolValue::Uint(index, size) => {
                if size != 8 {
                    bail!("Invalid aggregate_fn_id size");
                }
                AggregationFunction::from_index(index.to_string().parse().unwrap())?
            }
            _ => bail!("Invalid aggregate_fn_id type"),
        };
        // Turn bytes into hex string
        match value[1].as_uint() {
            Some((index, size)) => {
                if size != 8 {
                    bail!("Invalid operator size");
                }
                let operator = Operator::from_index(index.to_string().parse().unwrap())?;
                match value[2].as_uint() {
                    Some((value, size)) => {
                        if size != 256 {
                            bail!("Invalid value_to_compare size");
                        }
                        let aggregate_fn_ctx: FunctionContext =
                            FunctionContext::new(operator, value);

                        Ok(Self {
                            aggregate_fn_id,
                            aggregate_fn_ctx,
                        })
                    }
                    None => bail!("Invalid value_to_compare type"),
                }
            }
            None => bail!("Invalid operator type"),
        }
    }

    /// Encode the task without datalake
    fn encode(&self) -> Result<Vec<u8>> {
        let aggregate_fn_id = DynSolValue::Uint(
            U256::from(AggregationFunction::to_index(&self.aggregate_fn_id)),
            8,
        );

        let operator = DynSolValue::Uint(
            U256::from(Operator::to_index(&self.aggregate_fn_ctx.operator)),
            8,
        );

        let value_to_compare = DynSolValue::Uint(self.aggregate_fn_ctx.value_to_compare, 32);

        let header_tuple_value =
            DynSolValue::Tuple(vec![aggregate_fn_id, operator, value_to_compare]);

        Ok(header_tuple_value.abi_encode())
    }
}

#[cfg(test)]
mod tests {

    use alloy::hex::FromHex;

    use crate::{
        datalake::{
            block_sampled::BlockSampledDatalake, envelope::DatalakeEnvelope, DatalakeCompute,
        },
        solidity_types::traits::DatalakeComputeCodecs,
    };

    use super::*;

    #[test]
    fn test_task_with_ctx_serialize() {
        let task = Computation::new(
            "count",
            Some(FunctionContext::new(
                Operator::GreaterThanOrEqual,
                U256::from(100),
            )),
        );

        let inner_task = Computation {
            aggregate_fn_id: AggregationFunction::COUNT,
            aggregate_fn_ctx: FunctionContext::new(Operator::GreaterThanOrEqual, U256::from(100)),
        };

        let serialized = task.encode().unwrap();
        let inner_task_serialized = inner_task.encode().unwrap();
        assert_eq!(serialized, inner_task_serialized);

        let serialized: &str = "0x000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000064";
        let deserialized = Computation::decode(&Vec::from_hex(serialized).unwrap()).unwrap();
        assert_eq!(task, deserialized)
    }

    #[test]
    fn test_task_without_ctx_serialize() {
        // AVG
        let task = Computation::new("avg", None);

        let inner_task = Computation {
            aggregate_fn_id: AggregationFunction::AVG,
            aggregate_fn_ctx: FunctionContext::default(),
        };

        let serialized = task.encode().unwrap();
        let inner_task_serialized = inner_task.encode().unwrap();
        assert_eq!(serialized, inner_task_serialized);
        let serialized_bytes: Vec<u8> = Vec::from_hex("0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(serialized, serialized_bytes);
        let deserialized = Computation::decode(&serialized_bytes).unwrap();
        assert_eq!(task, deserialized);
        // MIN
        let task = Computation::new("min", None);

        let inner_task = Computation {
            aggregate_fn_id: AggregationFunction::MIN,
            aggregate_fn_ctx: FunctionContext::default(),
        };

        let serialized = task.encode().unwrap();
        let inner_task_serialized = inner_task.encode().unwrap();
        assert_eq!(serialized, inner_task_serialized);
        let serialized_bytes: Vec<u8> = Vec::from_hex("0x000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(serialized, serialized_bytes);
        let deserialized = Computation::decode(&serialized_bytes).unwrap();
        assert_eq!(task, deserialized);
    }

    #[test]
    fn test_task_with_datalake() {
        let task = Computation::new(
            "count",
            Some(FunctionContext::new(
                Operator::GreaterThanOrEqual,
                U256::from(100),
            )),
        );
        let datalake = DatalakeEnvelope::BlockSampled(
            BlockSampledDatalake::new(0, 100, "header.base_fee_per_gas".to_string(), 1).unwrap(),
        );
        let task_with_datalake = DatalakeCompute::new(datalake, task);

        let serialized = task_with_datalake.encode().unwrap();
        let serialized_bytes: Vec<u8> = Vec::from_hex("cfa530587401307617ef751178c78751c83757e2143b73b4ffadb5969ca6215e000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000064").unwrap();
        assert_eq!(serialized, serialized_bytes);
    }
}
