use std::str::FromStr;

use anyhow::{bail, Result};

use crate::datalake::{DatalakeCollection, DatalakeField};

use super::{TransactionField, TransactionReceiptField};

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

impl TransactionsCollection {
    pub fn serialize(&self) -> Result<[u8; 2]> {
        match self {
            TransactionsCollection::Transactions(ref field) => Ok([0, field.to_index()]),
            TransactionsCollection::TranasactionReceipts(ref field) => Ok([1, field.to_index()]),
        }
    }

    pub fn deserialize(bytes: &[u8; 2]) -> Result<Self> {
        if bytes.len() != 2 {
            return Err(anyhow::Error::msg("Invalid transactions collection"));
        }

        match bytes[0] {
            0 => Ok(TransactionsCollection::Transactions(
                TransactionField::from_index(bytes[1])?,
            )),
            1 => Ok(TransactionsCollection::TranasactionReceipts(
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
