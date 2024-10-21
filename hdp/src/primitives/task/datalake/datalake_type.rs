use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Identifier for a BlockSampledDatalake
pub const BLOCK_SAMPLED_DATALAKE_TYPE_ID: u8 = 0;

/// Identifier for a TransactionsDatalake
pub const TRANSACTIONS_IN_BLOCK_DATALAKE_TYPE_ID: u8 = 1;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum DatalakeType {
    BlockSampled = 0,
    TransactionsInBlock = 1,
}

impl FromStr for DatalakeType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "BLOCK_SAMPLED" => Ok(DatalakeType::BlockSampled),
            "TRANSACTIONS_IN_BLOCK" => Ok(DatalakeType::TransactionsInBlock),
            _ => bail!("Unknown datalake type"),
        }
    }
}

impl From<DatalakeType> for u8 {
    fn from(value: DatalakeType) -> Self {
        match value {
            DatalakeType::BlockSampled => BLOCK_SAMPLED_DATALAKE_TYPE_ID,
            DatalakeType::TransactionsInBlock => TRANSACTIONS_IN_BLOCK_DATALAKE_TYPE_ID,
        }
    }
}

impl DatalakeType {
    pub fn variants() -> Vec<String> {
        vec!["BLOCK_SAMPLED", "TRANSACTIONS_IN_BLOCK"]
            .into_iter()
            .map(String::from)
            .collect()
    }

    pub fn to_u8(self) -> u8 {
        self.into()
    }

    pub fn from_index(value: u8) -> Result<Self> {
        match value {
            BLOCK_SAMPLED_DATALAKE_TYPE_ID => Ok(DatalakeType::BlockSampled),
            TRANSACTIONS_IN_BLOCK_DATALAKE_TYPE_ID => Ok(DatalakeType::TransactionsInBlock),
            _ => bail!("Invalid datalake type"),
        }
    }
}
