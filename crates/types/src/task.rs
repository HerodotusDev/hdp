use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{keccak256, U256};
use anyhow::Result;
use common::utils::bytes32_to_utf8_str;
use num_bigint::BigUint;
use num_traits::Num;

use crate::datalake::base::DatalakeBase;

/// ComputationalTask represents a task for certain datalake with a specified aggregate function
#[derive(Debug)]
pub struct ComputationalTask {
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

    pub fn from_serialized(serialized: &[u8]) -> Result<Self> {
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
        let datalake = self.datalake.as_ref().ok_or("Datalake is None").unwrap();
        println!("datalake: {:?}", datalake);
        // Convert BigUint to a byte array

        let datalake_identifier =
            U256::from_str_radix(&datalake.identifier[2..], 16).expect("Invalid hex string");
        // Use the converted value
        let identifier_value = DynSolValue::Uint(datalake_identifier, 256);
        let aggregate_fn_id_value = DynSolValue::String(self.aggregate_fn_id.clone());
        let aggregate_fn_ctx_value = match &self.aggregate_fn_ctx {
            None => DynSolValue::Bytes("".to_string().into_bytes()),
            Some(ctx) => DynSolValue::Bytes(ctx.clone().into_bytes()),
        };

        let header_tuple_value = DynSolValue::Tuple(vec![
            identifier_value,
            aggregate_fn_id_value,
            aggregate_fn_ctx_value,
        ]);

        let datalake_header_encode = header_tuple_value.abi_encode();
        let hash = keccak256(datalake_header_encode);
        format!("0x{:x}", hash)
    }
}
