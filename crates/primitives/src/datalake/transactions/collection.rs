use anyhow::{bail, Result};
use serde::de::{self, Deserializer, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

use crate::datalake::{DatalakeCollection, DatalakeField};

use super::{TransactionField, TransactionReceiptField};

pub enum TransactionsCollectionType {
    Transactions,
    TransactionReceipts,
}

impl TransactionsCollectionType {
    pub fn variants() -> Vec<String> {
        vec!["TX".to_string(), "TX_RECEIPT".to_string()]
    }
}

impl FromStr for TransactionsCollectionType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "TX" => Ok(TransactionsCollectionType::Transactions),
            "TX_RECEIPT" => Ok(TransactionsCollectionType::TransactionReceipts),
            _ => bail!("Unknown transactions collection type"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionsCollection {
    Transactions(TransactionField),
    TranasactionReceipts(TransactionReceiptField),
}

impl DatalakeCollection for TransactionsCollection {
    fn to_index(&self) -> u8 {
        match self {
            TransactionsCollection::Transactions(ref field) => field.to_index(),
            TransactionsCollection::TranasactionReceipts(ref field) => field.to_index(),
        }
    }
}

impl Serialize for TransactionsCollection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = match self {
            TransactionsCollection::Transactions(ref field) => {
                let mut bytes = Vec::with_capacity(2);
                bytes.push(1);
                bytes.push(field.to_index());
                bytes
            }
            TransactionsCollection::TranasactionReceipts(ref field) => {
                let mut bytes = Vec::with_capacity(2);
                bytes.push(2);
                bytes.push(field.to_index());
                bytes
            }
        };

        serializer.serialize_bytes(&bytes)
    }
}

impl<'de> Deserialize<'de> for TransactionsCollection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TransactionsCollectionVisitor;

        impl<'de> Visitor<'de> for TransactionsCollectionVisitor {
            type Value = TransactionsCollection;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a byte array representing TransactionsCollection")
            }

            fn visit_byte_buf<E>(self, bytes: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if bytes.len() != 2 {
                    return Err(de::Error::invalid_length(bytes.len(), &self));
                }

                match bytes[0] {
                    1 => {
                        let field =
                            TransactionField::from_index(bytes[1]).map_err(de::Error::custom)?;
                        Ok(TransactionsCollection::Transactions(field))
                    }
                    2 => {
                        let field = TransactionReceiptField::from_index(bytes[1])
                            .map_err(de::Error::custom)?;
                        Ok(TransactionsCollection::TranasactionReceipts(field))
                    }
                    _ => Err(de::Error::custom("Unknown transactions collection")),
                }
            }
        }

        deserializer.deserialize_byte_buf(TransactionsCollectionVisitor)
    }
}

impl FromStr for TransactionsCollection {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        // Split into two parts by '.'
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 2 {
            bail!("Invalid transactions collection format");
        }

        match parts[0].to_uppercase().as_str() {
            "TX" => Ok(TransactionsCollection::Transactions(
                parts[1].to_uppercase().as_str().parse()?,
            )),
            "TX_RECEIPT" => Ok(TransactionsCollection::TranasactionReceipts(
                parts[1].to_uppercase().as_str().parse()?,
            )),
            _ => bail!("Unknown transactions collection"),
        }
    }
}

impl Display for TransactionsCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionsCollection::Transactions(ref field) => write!(f, "TX.{}", field),
            TransactionsCollection::TranasactionReceipts(ref field) => {
                write!(f, "TX_RECEIPT.{}", field)
            }
        }
    }
}
