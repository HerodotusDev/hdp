use self::{
    base::{DatalakeBase, Derivable},
    block_sampled::BlockSampledDatalake,
    dynamic_layout::DynamicLayoutDatalake,
    transactions::TransactionsDatalake,
};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

pub mod base;
pub mod block_sampled;
pub mod dynamic_layout;
pub mod transactions;

/// Type of datalake
#[derive(Debug, Clone, PartialEq)]
pub enum Datalake {
    BlockSampled(BlockSampledDatalake),
    DynamicLayout(DynamicLayoutDatalake),
    Transactions(TransactionsDatalake),
}

pub(crate) trait DatalakeCollection {
    fn to_index(&self) -> u8;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum DatalakeCode {
    BlockSampled = 0,
    DynamicLayout = 1,
    Transactions = 2,
}

impl DatalakeCode {
    pub fn index(&self) -> u8 {
        match self {
            DatalakeCode::BlockSampled => 0,
            DatalakeCode::DynamicLayout => 1,
            DatalakeCode::Transactions => 2,
        }
    }

    pub fn from_index(value: u8) -> Result<DatalakeCode> {
        match value {
            0 => Ok(DatalakeCode::BlockSampled),
            1 => Ok(DatalakeCode::DynamicLayout),
            2 => Ok(DatalakeCode::Transactions),
            _ => bail!("Invalid datalake code"),
        }
    }
}

/// Transform different datalake types into DatalakeBase
impl Derivable for Datalake {
    fn derive(&self) -> DatalakeBase {
        match self {
            Datalake::BlockSampled(datalake) => {
                DatalakeBase::new(&datalake.commit(), Datalake::BlockSampled(datalake.clone()))
            }
            Datalake::DynamicLayout(_) => panic!("Unsupported datalake type"),
            Datalake::Transactions(datalake) => {
                DatalakeBase::new(&datalake.commit(), Datalake::Transactions(datalake.clone()))
            }
        }
    }
}

impl Datalake {
    pub fn encode(&self) -> Result<String> {
        match self {
            Datalake::BlockSampled(datalake) => datalake.encode(),
            Datalake::DynamicLayout(_) => bail!("Unsupported datalake type"),
            Datalake::Transactions(datalake) => datalake.encode(),
        }
    }

    pub fn get_datalake_type(&self) -> DatalakeCode {
        match self {
            Datalake::BlockSampled(_) => DatalakeCode::BlockSampled,
            Datalake::DynamicLayout(_) => DatalakeCode::DynamicLayout,
            Datalake::Transactions(_) => DatalakeCode::Transactions,
        }
    }

    pub(crate) fn get_collection_type(&self) -> Box<dyn DatalakeCollection> {
        match self {
            Datalake::BlockSampled(datalake) => Box::new(datalake.get_collection_type()),
            Datalake::DynamicLayout(_) => panic!("Unsupported datalake type"),
            Datalake::Transactions(datalake) => Box::new(datalake.sampled_property.clone()),
        }
    }
}
