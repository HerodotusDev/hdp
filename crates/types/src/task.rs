use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{keccak256, U256};
use anyhow::Result;

use crate::{datalake_base::DatalakeBase, utils::bytes32_to_str};

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

        let aggregate_fn_id = bytes32_to_str(value[0].as_fixed_bytes().unwrap().0).unwrap();
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

        let identifier = u64::from_str_radix(&datalake.identifier, 16)
            .expect("Failed to parse identifier as a hexadecimal number");
        let identifier_value = DynSolValue::Uint(U256::from(identifier), 256);
        let aggregate_fn_id_value = DynSolValue::String(self.aggregate_fn_id.clone());
        let aggregate_fn_ctx_value =
            DynSolValue::Bytes(self.aggregate_fn_ctx.clone().unwrap().into_bytes());
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
