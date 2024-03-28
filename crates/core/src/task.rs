use std::str::FromStr;

use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{hex::FromHex, keccak256, FixedBytes};
use anyhow::{bail, Result};

use hdp_primitives::utils::{
    bytes_to_hex_string, fixed_bytes_str_to_utf8_str, utf8_str_to_fixed_bytes32,
};

use crate::compiler::DatalakeCompiler;

/// ComputationalTask represents a task for certain datalake with a specified aggregate function
#[derive(Debug)]
pub struct ComputationalTask {
    /// Target datalake as optional.
    /// - If None, task is non filled with datalake.
    /// - If Some, task is filled with datalake.
    ///
    /// Encoding and Commit will be different based on this field.
    pub datalake: Option<DatalakeCompiler>,
    pub aggregate_fn_id: String,
    pub aggregate_fn_ctx: Option<String>,
}

impl ComputationalTask {
    pub fn new(
        datalake: Option<DatalakeCompiler>,
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
                let aggregate_fn_id_value =
                    DynSolValue::FixedBytes(utf8_str_to_fixed_bytes32(&self.aggregate_fn_id), 32);

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

                let aggregate_fn_id_value =
                    DynSolValue::FixedBytes(utf8_str_to_fixed_bytes32(&self.aggregate_fn_id), 32);
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
            let datalake = DatalakeCompiler {
                commitment: format!("0x{:x}", datalake.0),
                datalake: None,
                result: None,
            };

            Some(datalake)
        } else {
            None
        };

        let aggregate_fn_id = match value[1] {
            DynSolValue::FixedBytes(bytes, _) => fixed_bytes_str_to_utf8_str(bytes)?,
            _ => bail!("Invalid aggregate_fn_id type"),
        };

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

        let aggregate_fn_id = match value[0] {
            DynSolValue::FixedBytes(bytes, _) => fixed_bytes_str_to_utf8_str(bytes)?,
            _ => bail!("Invalid aggregate_fn_id type"),
        };

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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_dyn_abi::DynSolType;
    use alloy_primitives::hex::FromHex;

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
                    ComputationalTask::decode_not_filled_task(task.as_bytes().unwrap()).unwrap();
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
}
