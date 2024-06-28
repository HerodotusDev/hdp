use std::{fmt::Display, str::FromStr};

use anyhow::{bail, Result};

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

#[derive(Debug, Clone, PartialEq, Eq)]
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

    fn serialize(&self) -> Result<Vec<u8>> {
        match self {
            TransactionsCollection::Transactions(ref field) => Ok([1, field.to_index()].to_vec()),
            TransactionsCollection::TranasactionReceipts(ref field) => {
                Ok([2, field.to_index()].to_vec())
            }
        }
    }

    fn deserialize(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 2 {
            return Err(anyhow::Error::msg("Invalid transactions collection"));
        }

        match bytes[0] {
            1 => Ok(TransactionsCollection::Transactions(
                TransactionField::from_index(bytes[1])?,
            )),
            2 => Ok(TransactionsCollection::TranasactionReceipts(
                TransactionReceiptField::from_index(bytes[1])?,
            )),
            _ => Err(anyhow::Error::msg("Unknown transactions collection")),
        }
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
