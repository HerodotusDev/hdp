use self::{
    base::{DatalakeBase, Derivable},
    block_sampled::BlockSampledDatalake,
    dynamic_layout::DynamicLayoutDatalake,
};
use anyhow::{bail, Result};

pub mod base;
pub mod block_sampled;
pub mod dynamic_layout;

/// Type of datalake
#[derive(Debug, Clone, PartialEq)]
pub enum Datalake {
    BlockSampled(BlockSampledDatalake),
    DynamicLayout(DynamicLayoutDatalake),
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
            Datalake::Unknown => panic!("Unknown datalake type"),
        }
    }
}

impl Datalake {
    pub fn encode(&self) -> Result<String> {
        match self {
            Datalake::BlockSampled(datalake) => datalake.encode(),
            Datalake::DynamicLayout(_) => bail!("Unsupported datalake type"),
            Datalake::Unknown => bail!("Unknown datalake type"),
        }
    }

    pub fn get_datalake_type(&self) -> u8 {
        match self {
            Datalake::BlockSampled(_) => 0,
            Datalake::DynamicLayout(_) => 1,
            Datalake::Unknown => panic!("Unknown datalake type"),
        }
    }

    pub fn get_property_type(&self) -> u8 {
        match self {
            Datalake::BlockSampled(datalake) => datalake.get_property_type(),
            Datalake::DynamicLayout(_) => panic!("Unsupported datalake type"),
            Datalake::Unknown => panic!("Unknown datalake type"),
        }
    }
}
