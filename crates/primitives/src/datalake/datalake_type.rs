use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

/// Identifier for a [`BlockSampledDatalake`] type.
pub const BLOCK_SAMPLED_DATALAKE_TYPE_ID: u8 = 0;

/// Identifier for an [`TransactionsDatalake`] type.
pub const TRANSACTIONS_IN_BLOCK_DATALAKE_TYPE_ID: u8 = 2;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum DatalakeType {
    BlockSampled = 0,
    TransactionsInBlock = 1,
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
