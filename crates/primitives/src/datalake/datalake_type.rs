use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

/// Identifier for a [`BlockSampledDatalake`] type.
pub const BLOCK_SAMPLED_DATALAKE_TYPE_ID: u8 = 0;

/// Identifier for a [`DynamicLayoutDatalake`] type.
pub const DYNAMIC_LAYOUT_DATALAKE_TYPE_ID: u8 = 1;

/// Identifier for an [`TransactionsDatalake`] type.
pub const TRANSACTIONS_DATALAKE_TYPE_ID: u8 = 2;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum DatalakeType {
    BlockSampled = 0,
    DynamicLayout = 1,
    Transactions = 2,
}

impl From<DatalakeType> for u8 {
    fn from(value: DatalakeType) -> Self {
        match value {
            DatalakeType::BlockSampled => BLOCK_SAMPLED_DATALAKE_TYPE_ID,
            DatalakeType::DynamicLayout => DYNAMIC_LAYOUT_DATALAKE_TYPE_ID,
            DatalakeType::Transactions => TRANSACTIONS_DATALAKE_TYPE_ID,
        }
    }
}

impl DatalakeType {
    pub fn from_index(value: u8) -> Result<Self> {
        match value {
            BLOCK_SAMPLED_DATALAKE_TYPE_ID => Ok(DatalakeType::BlockSampled),
            DYNAMIC_LAYOUT_DATALAKE_TYPE_ID => Ok(DatalakeType::DynamicLayout),
            TRANSACTIONS_DATALAKE_TYPE_ID => Ok(DatalakeType::Transactions),
            _ => bail!("Invalid datalake type"),
        }
    }
}
