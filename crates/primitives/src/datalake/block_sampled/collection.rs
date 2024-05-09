use alloy_primitives::{Address, StorageKey};
use anyhow::{bail, Result};
use serde::de::{self, Deserializer, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use serde_bytes::Bytes;
use std::{fmt::Display, str::FromStr};

use crate::datalake::{DatalakeCollection, DatalakeField};

use super::rlp_fields::{AccountField, HeaderField};

#[derive(Debug, Clone, PartialEq)]
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
}

impl<'de> Deserialize<'de> for BlockSampledCollection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BlockSampledCollectionVisitor;

        impl<'de> Visitor<'de> for BlockSampledCollectionVisitor {
            type Value = BlockSampledCollection;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a byte array representing BlockSampledCollection")
            }

            fn visit_byte_buf<E>(self, bytes: std::vec::Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let bytes = bytes.to_vec();
                if bytes.is_empty() {
                    return Err(de::Error::invalid_length(0, &self));
                }

                match bytes[0] {
                    1 => {
                        if bytes.len() != 2 {
                            return Err(de::Error::invalid_length(bytes.len(), &self));
                        }
                        let field = HeaderField::from_index(bytes[1]).map_err(de::Error::custom)?;
                        Ok(BlockSampledCollection::Header(field))
                    }
                    2 => {
                        if bytes.len() != 22 {
                            return Err(de::Error::invalid_length(bytes.len(), &self));
                        }
                        let address = Address::from_slice(&bytes[1..21]);
                        let field =
                            AccountField::from_index(bytes[21]).map_err(de::Error::custom)?;
                        Ok(BlockSampledCollection::Account(address, field))
                    }
                    3 => {
                        if bytes.len() != 53 {
                            return Err(de::Error::invalid_length(bytes.len(), &self));
                        }
                        let address = Address::from_slice(&bytes[1..21]);
                        let slot = StorageKey::from_slice(&bytes[21..53]);
                        Ok(BlockSampledCollection::Storage(address, slot))
                    }
                    _ => Err(de::Error::custom("Unknown block sampled collection")),
                }
            }
        }

        deserializer.deserialize_byte_buf(BlockSampledCollectionVisitor)
    }
}

impl Serialize for BlockSampledCollection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = match self {
            BlockSampledCollection::Header(field) => {
                let mut bytes = Vec::with_capacity(2);
                bytes.push(1);
                bytes.push(field.to_index());
                bytes
            }
            BlockSampledCollection::Account(address, field) => {
                let mut bytes = Vec::with_capacity(22);
                bytes.push(2);
                bytes.extend_from_slice(address.as_slice());
                bytes.push(field.to_index());
                bytes
            }
            BlockSampledCollection::Storage(address, slot) => {
                let mut bytes = Vec::with_capacity(53);
                bytes.push(3);
                bytes.extend_from_slice(address.as_slice());
                bytes.extend_from_slice(slot.as_slice());
                bytes
            }
        };
        let bytes = Bytes::new(&bytes);
        serializer.serialize_bytes(&bytes)
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
