use std::str::FromStr;

use crate::{
    datalake::{datalake_type::DatalakeType, Datalake, DatalakeCollection},
    utils::bytes_to_hex_string,
};

use super::collection::BlockSampledCollection;
use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{hex::FromHex, keccak256};
use anyhow::{bail, Result};

/// [`BlockSampledDatalake`] is a struct that represents a block sampled datalake.
/// It contains the block range, the sampled property, and the increment.
///
/// The block range is inclusive, so the block range is from `block_range_start` to `block_range_end`
#[derive(Debug, Clone, PartialEq)]
pub struct BlockSampledDatalake {
    /// The start of the block range
    pub block_range_start: u64,
    /// The end of the block range
    pub block_range_end: u64,
    /// The sampled property
    pub sampled_property: BlockSampledCollection,
    /// The increment
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
}

impl Datalake for BlockSampledDatalake {
    /// Get the datalake code for block sampled datalake
    fn get_datalake_type(&self) -> DatalakeType {
        DatalakeType::BlockSampled
    }

    /// Encode the block sampled datalake
    fn encode(&self) -> Result<String> {
        let datalake_code: DynSolValue = self.get_datalake_type().to_u8().into();
        let block_range_start: DynSolValue = self.block_range_start.into();
        let block_range_end: DynSolValue = self.block_range_end.into();
        let sampled_property: DynSolValue = self.sampled_property.serialize()?.into();
        let increment: DynSolValue = self.increment.into();

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
    fn commit(&self) -> String {
        let encoded_datalake = self.encode().expect("Encoding failed");
        let bytes = Vec::from_hex(encoded_datalake).expect("Invalid hex string");
        let hash = keccak256(bytes);
        format!("0x{:x}", hash)
    }

    /// Decode the encoded block sampled datalake
    fn decode(encoded: &str) -> Result<Self> {
        let abi_type: DynSolType = "(uint256,uint256,uint256,uint256,bytes)".parse()?;
        let bytes = Vec::from_hex(encoded).expect("Invalid hex string");
        let decoded = abi_type.abi_decode_sequence(&bytes)?;

        let value = decoded.as_tuple().unwrap();
        let datalake_code = value[0].as_uint().unwrap().0.to_string().parse::<u8>()?;

        if DatalakeType::from_index(datalake_code)? != DatalakeType::BlockSampled {
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
