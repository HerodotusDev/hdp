use std::str::FromStr;

use alloy_primitives::{Address, U256};
use anyhow::{bail, Result};

use crate::datalake::{DatalakeCollection, DatalakeField};

use super::fields::{AccountField, HeaderField};

#[derive(Debug, Clone, PartialEq)]
pub enum BlockSampledCollection {
    Header(HeaderField),
    Account(Address, AccountField),
    Storage(Address, U256),
}

impl DatalakeCollection for BlockSampledCollection {
    fn to_index(&self) -> u8 {
        match self {
            BlockSampledCollection::Header(_) => 1,
            BlockSampledCollection::Account(..) => 2,
            BlockSampledCollection::Storage(..) => 3,
        }
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut serialized = Vec::new();
        match self {
            BlockSampledCollection::Header(field) => {
                serialized.push(1);
                serialized.push(field.to_index());
            }
            BlockSampledCollection::Account(address, field) => {
                serialized.push(2);
                serialized.extend_from_slice(address.as_slice());
                serialized.push(field.to_index());
            }
            BlockSampledCollection::Storage(address, slot) => {
                serialized.push(3);
                serialized.extend_from_slice(address.as_slice());
                serialized.extend_from_slice(&slot.to_be_bytes::<32>());
            }
        }

        Ok(serialized)
    }

    fn deserialize(serialized: &[u8]) -> Result<Self> {
        if serialized.is_empty() {
            bail!("Invalid block sampled collection");
        }

        match serialized[0] {
            1 => {
                if serialized.len() != 2 {
                    bail!("Invalid header property");
                }
                Ok(BlockSampledCollection::Header(HeaderField::from_index(
                    serialized[1],
                )?))
            }
            2 => {
                if serialized.len() != 22 {
                    bail!("Invalid account property");
                }
                let address = Address::from_slice(&serialized[1..21]);
                Ok(BlockSampledCollection::Account(
                    address,
                    AccountField::from_index(serialized[21])?,
                ))
            }
            3 => {
                if serialized.len() != 53 {
                    bail!("Invalid storage property");
                }
                let address = Address::from_slice(&serialized[1..21]);
                let slot = U256::from_be_slice(&serialized[21..53]);
                Ok(BlockSampledCollection::Storage(address, slot))
            }
            _ => bail!("Unknown block sampled collection"),
        }
    }
}

impl FromStr for BlockSampledCollection {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        // Split into two parts by '.'
        let parts: Vec<&str> = s.split('.').collect();
        if !(parts.len() == 2 || parts.len() == 3) {
            bail!("Invalid block sampled collection format");
        }

        match parts[0].to_uppercase().as_str() {
            "HEADER" => Ok(BlockSampledCollection::Header(HeaderField::from_str(
                parts[1].to_uppercase().as_str(),
            )?)),
            "ACCOUNT" => {
                let address = Address::from_str(parts[1])?;
                let field = AccountField::from_str(parts[2].to_uppercase().as_str())?;
                Ok(BlockSampledCollection::Account(address, field))
            }
            "STORAGE" => {
                let address = Address::from_str(parts[1])?;
                let slot = U256::from_str(parts[2])?;
                Ok(BlockSampledCollection::Storage(address, slot))
            }
            _ => bail!("Unknown block sampled collection"),
        }
    }
}
