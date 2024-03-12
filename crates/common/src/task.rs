use std::str::FromStr;

use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{hex::FromHex, keccak256, FixedBytes};
use anyhow::{bail, Result};

use crate::{
    datalake::base::DatalakeBase,
    utils::{bytes32_to_utf8_str, bytes_to_hex_string, utf8_str_to_fixed_bytes32},
};

/// ComputationalTask represents a task for certain datalake with a specified aggregate function
#[derive(Debug)]
pub struct ComputationalTask {
    /// Target datalake as optional.
    /// - If None, task is non filled with datalake.
    /// - If Some, task is filled with datalake.
    ///
    /// Encoding and Commit will be different based on this field.
    pub datalake: Option<DatalakeBase>,
    pub aggregate_fn_id: String,
    pub aggregate_fn_ctx: Option<String>,
}

impl ComputationalTask {
    pub fn new(
        datalake: Option<DatalakeBase>,
        aggregate_fn_id: String,
        aggregate_fn_ctx: Option<String>,
    ) -> Self {
        Self {
            datalake,
            aggregate_fn_id,
            aggregate_fn_ctx,
        }
    }

    /// Encode the task into a hex string
    /// - If datalake is None, it will encode the task without datalake
    /// - If datalake is Some, it will encode the task with datalake commitment
    pub fn encode(&self) -> Result<String> {
        match &self.datalake {
            None => {
                let aggregate_fn_id_value = DynSolValue::FixedBytes(
                    alloy_primitives::FixedBytes(utf8_str_to_fixed_bytes32(&self.aggregate_fn_id)),
                    32,
                );

                let aggregate_fn_ctx_value = match &self.aggregate_fn_ctx {
                    None => DynSolValue::Bytes("".to_string().into_bytes()),
                    Some(ctx) => DynSolValue::Bytes(ctx.clone().into_bytes()),
                };

                let header_tuple_value =
                    DynSolValue::Tuple(vec![aggregate_fn_id_value, aggregate_fn_ctx_value]);

                let encoded_datalake = header_tuple_value.abi_encode();
                Ok(bytes_to_hex_string(&encoded_datalake))
            }
            Some(datalake) => {
                let identifier_value = DynSolValue::FixedBytes(
                    FixedBytes::from_str(&datalake.commitment).unwrap(),
                    32,
                );

                let aggregate_fn_id_value = DynSolValue::FixedBytes(
                    FixedBytes(utf8_str_to_fixed_bytes32(&self.aggregate_fn_id)),
                    32,
                );
                let aggregate_fn_ctx_value = match &self.aggregate_fn_ctx {
                    None => DynSolValue::Bytes("".to_string().into_bytes()),
                    Some(ctx) => DynSolValue::Bytes(ctx.clone().into_bytes()),
                };

                let header_tuple_value = DynSolValue::Tuple(vec![
                    identifier_value,
                    aggregate_fn_id_value,
                    aggregate_fn_ctx_value,
                ]);

                match header_tuple_value.abi_encode_sequence() {
                    Some(encoded) => Ok(bytes_to_hex_string(&encoded)),
                    None => bail!("Failed to encode the task"),
                }
            }
        }
    }

    /// Decode a serialized task that filled with datalake
    pub fn decode(serialized: &[u8]) -> Result<Self> {
        let task_type: DynSolType = "(uint256,bytes32,bytes)".parse()?;
        let decoded = task_type.abi_decode(serialized)?;

        let value = decoded.as_tuple().unwrap();

        let datalake_value = if let Some(datalake) = value[0].as_uint() {
            let datalake = DatalakeBase {
                commitment: format!("0x{:x}", datalake.0),
                datalake_type: None,
                result: None,
            };

            Some(datalake)
        } else {
            None
        };

        let aggregate_fn_id = bytes32_to_utf8_str(value[1].as_bytes().unwrap()).unwrap();
        let aggregate_fn_ctx = value[2].as_str().map(|s| s.to_string());

        Ok(ComputationalTask {
            datalake: datalake_value,
            aggregate_fn_id,
            aggregate_fn_ctx,
        })
    }

    /// Decode task that is not filled with datalake
    pub fn decode_not_filled_task(serialized: &[u8]) -> Result<Self> {
        let aggregate_fn_type: DynSolType = "(bytes32,bytes)".parse()?;
        let decoded = aggregate_fn_type.abi_decode(serialized)?;

        let value = decoded.as_tuple().unwrap();

        let aggregate_fn_id = bytes32_to_utf8_str(value[0].as_fixed_bytes().unwrap().0).unwrap();
        let aggregate_fn_ctx = value[1].as_str().map(|s| s.to_string());

        Ok(ComputationalTask {
            datalake: None,
            aggregate_fn_id,
            aggregate_fn_ctx,
        })
    }
}

impl ToString for ComputationalTask {
    fn to_string(&self) -> String {
        let encoded_datalake = self.encode().unwrap();
        let bytes = Vec::from_hex(encoded_datalake).expect("Invalid hex string");
        let hash = keccak256(bytes);
        format!("0x{:x}", hash)
    }
}
