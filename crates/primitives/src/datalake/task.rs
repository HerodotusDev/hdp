use std::str::FromStr;

use crate::aggregate_fn::{integer::Operator, AggregationFunction, FunctionContext};

use super::envelope::DatalakeEnvelope;
use alloy::primitives::{hex::FromHex, keccak256, U256};
use alloy::{
    dyn_abi::{DynSolType, DynSolValue},
    primitives::B256,
};
use anyhow::{bail, Result};

#[derive(Debug)]
pub struct DatalakeCompute {
    pub datalake: DatalakeEnvelope,
    pub compute: Computation,
}

impl DatalakeCompute {
    pub fn new(datalake: DatalakeEnvelope, compute: Computation) -> Self {
        Self { datalake, compute }
    }

    pub fn commit(&self) -> B256 {
        let encoded_datalake = self.encode().unwrap();
        let bytes = Vec::from_hex(encoded_datalake).expect("Invalid hex string");
        keccak256(bytes)
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        let identifier_value = DynSolValue::FixedBytes(self.datalake.get_commitment(), 32);

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

        // match header_tuple_value.abi_encode_sequence() {
        //     Some(encoded) => Ok(bytes_to_hex_string(&encoded)),
        //     None => bail!("Failed to encode the task"),
        // }
    }
}

/// [`Computation`] is a structure that contains the aggregate function id and context
#[derive(Debug, PartialEq, Eq)]
pub struct Computation {
    pub aggregate_fn_id: AggregationFunction,
    pub aggregate_fn_ctx: FunctionContext,
}

impl Computation {
    pub fn new(aggregate_fn_id: &str, aggregate_fn_ctx: Option<FunctionContext>) -> Self {
        let aggregate_fn_ctn_parsed = match aggregate_fn_ctx {
            None => FunctionContext::new(Operator::None, U256::ZERO),
            Some(ctx) => ctx,
        };
        Self {
            aggregate_fn_id: AggregationFunction::from_str(aggregate_fn_id).unwrap(),
            aggregate_fn_ctx: aggregate_fn_ctn_parsed,
        }
    }

    /// Encode the task without datalake
    pub fn encode(&self) -> Result<Vec<u8>> {
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

    /// Decode task that is not filled with datalake
    pub fn decode_not_filled_task(serialized: &[u8]) -> Result<Self> {
        let aggregate_fn_type: DynSolType = "(uint8,uint8,uint256)".parse()?;
        let decoded = aggregate_fn_type.abi_decode(serialized)?;

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
}

pub struct ExtendedDatalakeTask {
    pub task: DatalakeCompute,
    pub values: Vec<U256>,
}

#[cfg(test)]
mod tests {

    use crate::datalake::block_sampled::BlockSampledDatalake;

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
        let deserialized =
            Computation::decode_not_filled_task(&Vec::from_hex(serialized).unwrap()).unwrap();
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
        let deserialized = Computation::decode_not_filled_task(&serialized_bytes).unwrap();
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
        let deserialized = Computation::decode_not_filled_task(&serialized_bytes).unwrap();
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
