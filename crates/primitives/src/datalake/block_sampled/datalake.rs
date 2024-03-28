use std::str::FromStr;

use crate::utils::bytes_to_hex_string;

use super::collection::BlockSampledCollection;
use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{hex::FromHex, keccak256, U256};
use anyhow::{bail, Result};

/// BlockSampledDatalake represents a datalake for a block range
#[derive(Debug, Clone, PartialEq)]
pub struct BlockSampledDatalake {
    pub block_range_start: u64,
    pub block_range_end: u64,
    pub sampled_property: BlockSampledCollection,
    pub increment: u64,
}

impl BlockSampledDatalake {
    pub fn new(
        block_range_start: u64,
        block_range_end: u64,
        sampled_property: String,
        increment: u64,
    ) -> Result<Self> {
        Ok(Self {
            block_range_start,
            block_range_end,
            sampled_property: BlockSampledCollection::from_str(&sampled_property)?,
            increment,
        })
    }

    /// Encode the block sampled datalake
    pub fn encode(&self) -> Result<String> {
        let block_range_start = DynSolValue::Uint(U256::from(self.block_range_start), 256);
        let block_range_end = DynSolValue::Uint(U256::from(self.block_range_end), 256);
        let sampled_property =
            DynSolValue::Bytes(self.sampled_property.serialize().unwrap().to_vec());
        let increment = DynSolValue::Uint(U256::from(self.increment), 256);
        let datalake_code = DynSolValue::Uint(U256::from(0), 256);

        let tuple_value = DynSolValue::Tuple(vec![
            datalake_code,
            block_range_start,
            block_range_end,
            increment,
            sampled_property,
        ]);

        match tuple_value.abi_encode_sequence() {
            Some(encoded_datalake) => Ok(bytes_to_hex_string(&encoded_datalake)),
            None => bail!("Encoding failed"),
        }
    }

    /// Get the commitment hash of the block sampled datalake
    pub fn commit(&self) -> String {
        let encoded_datalake = self.encode().expect("Encoding failed");
        let bytes = Vec::from_hex(encoded_datalake).expect("Invalid hex string");
        let hash = keccak256(bytes);
        format!("0x{:x}", hash)
    }

    /// Decode the encoded block sampled datalake
    pub fn decode(encoded: &str) -> Result<Self> {
        let datalake_type: DynSolType = "(uint256,uint256,uint256,uint256,bytes)".parse()?;
        let bytes = Vec::from_hex(encoded).expect("Invalid hex string");
        let decoded = datalake_type.abi_decode_sequence(&bytes)?;

        let value = decoded.as_tuple().unwrap();
        let datalake_code = value[0].as_uint().unwrap().0.to_string().parse::<u64>()?;

        if datalake_code != 0 {
            bail!("Encoded datalake is not a block sample datalake");
        }

        let block_range_start = value[1].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let block_range_end = value[2].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let sampled_property = BlockSampledCollection::deserialize(value[4].as_bytes().unwrap())?;
        let increment = value[3].as_uint().unwrap().0.to_string().parse::<u64>()?;

        Ok(Self {
            block_range_start,
            block_range_end,
            sampled_property,
            increment,
        })
    }
}
