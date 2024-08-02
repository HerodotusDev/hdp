use std::{fmt::Display, str::FromStr};

use alloy::primitives::{Address, StorageKey};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::primitives::task::datalake::{DatalakeCollection, DatalakeField};

use super::rlp_fields::{AccountField, HeaderField};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub enum BlockSampledCollection {
    Header(HeaderField),
    Account(Address, AccountField),
    Storage(Address, StorageKey),
}

pub enum BlockSampledCollectionType {
    Header,
    Account,
    Storage,
}

impl BlockSampledCollectionType {
    pub fn variants() -> Vec<String> {
        vec![
            "HEADER".to_string(),
            "ACCOUNT".to_string(),
            "STORAGE".to_string(),
        ]
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            BlockSampledCollectionType::Header => 0,
            BlockSampledCollectionType::Account => 1,
            BlockSampledCollectionType::Storage => 2,
        }
    }
}

impl FromStr for BlockSampledCollectionType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "HEADER" => Ok(BlockSampledCollectionType::Header),
            "ACCOUNT" => Ok(BlockSampledCollectionType::Account),
            "STORAGE" => Ok(BlockSampledCollectionType::Storage),
            _ => bail!("Unknown block sampled collection type"),
        }
    }
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
                serialized.extend_from_slice(slot.as_ref());
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
                let slot = StorageKey::from_slice(&serialized[21..53]);
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
                let slot = StorageKey::from_str(parts[2])?;
                Ok(BlockSampledCollection::Storage(address, slot))
            }
            _ => bail!("Unknown block sampled collection"),
        }
    }
}

impl TryFrom<String> for BlockSampledCollection {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        BlockSampledCollection::from_str(&value)
    }
}

impl Display for BlockSampledCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockSampledCollection::Header(field) => write!(f, "header.{}", field),
            BlockSampledCollection::Account(address, field) => {
                write!(f, "account.{}.{}", address, field)
            }
            BlockSampledCollection::Storage(address, slot) => {
                write!(f, "storage.{}.{}", address, slot)
            }
        }
    }
}
