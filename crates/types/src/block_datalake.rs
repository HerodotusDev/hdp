use alloy_dyn_abi::DynSolType;
use alloy_primitives::Address;
use anyhow::{bail, Result};

use crate::block_fields::{AccountField, HeaderField};

/// BlockDatalake represents a datalake for a block range
#[derive(Debug, Clone)]
pub struct BlockDatalake {
    pub block_range_start: usize,
    pub block_range_end: usize,
    pub sampled_property: String,
    pub increment: usize,
}

impl BlockDatalake {
    pub fn new(
        block_range_start: usize,
        block_range_end: usize,
        sampled_property: String,
        increment: usize,
    ) -> Self {
        Self {
            block_range_start,
            block_range_end,
            sampled_property,
            increment,
        }
    }

    pub fn from_serialized(serialized: &[u8]) -> Result<Self> {
        println!("serialized: {:?}", serialized);
        let datalake_type: DynSolType = "(uint256,uint256,uint256,bytes)".parse()?;
        let decoded = datalake_type.abi_decode(serialized)?;
        println!("decoded: {:?}", decoded);
        let value = decoded.as_tuple().unwrap();

        println!("value: {:?}", value);

        let block_range_start = value[0].as_uint().unwrap().1;
        let block_range_end = value[1].as_uint().unwrap().1;
        let sampled_property = Self::deserialize_sampled_property(value[3].as_bytes().unwrap())?;
        let increment = value[2].as_uint().unwrap().1;

        Ok(Self {
            block_range_start,
            block_range_end,
            sampled_property,
            increment,
        })
    }

    fn deserialize_sampled_property(serialized_sample_property: &[u8]) -> Result<String> {
        let collection_id = serialized_sample_property[0] as usize;
        let collection = ["header", "account", "storage"][collection_id - 1];
        match collection {
            "header" => {
                let header_prop_index = serialized_sample_property[1] as usize;
                let prop = HeaderField::from_index(header_prop_index)
                    .ok_or("Invalid header property index")
                    .unwrap()
                    .as_str();
                Ok(format!("{}.{}", collection, prop.to_lowercase()))
            }
            "account" => {
                let account = Address::from_slice(&serialized_sample_property[1..21]);
                let account_checksum = format!("{:?}", account);
                let account_prop_index = serialized_sample_property[21] as usize;
                let prop = AccountField::from_index(account_prop_index)
                    .ok_or("Invalid account property index")
                    .unwrap()
                    .as_str();
                Ok(format!(
                    "{}.{}.{}",
                    collection,
                    account_checksum,
                    prop.to_lowercase()
                ))
            }
            "storage" => {
                let account = Address::from_slice(&serialized_sample_property[1..21]);
                let account_checksum = format!("{:?}", account);
                let slot = &serialized_sample_property[21..53];
                let slot_hex = format!("0x{:x?}", slot);
                Ok(format!("{}.{}.{}", collection, account_checksum, slot_hex))
            }
            _ => bail!("Invalid collection id"),
        }
    }
}
