use self::{
    base::{DatalakeBase, Derivable},
    block_sampled::BlockSampledDatalake,
    dynamic_layout::DynamicLayoutDatalake,
    transactions::TransactionsDatalake,
};
use anyhow::{bail, Result};

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
    Unknown,
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
            Datalake::Unknown => panic!("Unknown datalake type"),
        }
    }
}

impl Datalake {
    pub fn encode(&self) -> Result<String> {
        match self {
            Datalake::BlockSampled(datalake) => datalake.encode(),
            Datalake::DynamicLayout(_) => bail!("Unsupported datalake type"),
            Datalake::Transactions(datalake) => datalake.encode(),
            Datalake::Unknown => bail!("Unknown datalake type"),
        }
    }

    pub fn get_datalake_type(&self) -> u8 {
        match self {
            Datalake::BlockSampled(_) => 0,
            Datalake::DynamicLayout(_) => 1,
            Datalake::Transactions(_) => 2,
            Datalake::Unknown => panic!("Unknown datalake type"),
        }
    }

    pub fn get_property_type(&self) -> u8 {
        match self {
            Datalake::BlockSampled(datalake) => datalake.get_property_type(),
            Datalake::DynamicLayout(_) => panic!("Unsupported datalake type"),
            Datalake::Transactions(datalake) => datalake.account_type.index() as u8,
            Datalake::Unknown => panic!("Unknown datalake type"),
        }
    }
}
