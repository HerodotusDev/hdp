use self::{
    base::{DatalakeBase, Derivable},
    block_sampled::BlockSampledDatalake,
    dynamic_layout::DynamicLayoutDatalake,
};
use anyhow::{bail, Result};

pub mod base;
pub mod block_sampled;
pub mod dynamic_layout;

#[derive(Debug, Clone, PartialEq)]
pub enum Datalake {
    BlockSampled(BlockSampledDatalake),
    DynamicLayout(DynamicLayoutDatalake),
    Unknown,
}

impl Derivable for Datalake {
    fn derive(&self) -> DatalakeBase {
        match self {
            Datalake::BlockSampled(datalake) => DatalakeBase::new(
                &datalake.to_string(),
                Datalake::BlockSampled(datalake.clone()),
            ),
            Datalake::DynamicLayout(datalake) => DatalakeBase::new(
                &datalake.to_string(),
                Datalake::DynamicLayout(datalake.clone()),
            ),
            Datalake::Unknown => panic!("Unknown datalake type"),
        }
    }
}

impl Datalake {
    pub fn serialize(&self) -> Result<String> {
        match self {
            Datalake::BlockSampled(datalake) => datalake.serialize(),
            Datalake::DynamicLayout(datalake) => datalake.serialize(),
            Datalake::Unknown => bail!("Unknown datalake type"),
        }
    }

    pub fn get_type(&self) -> u8 {
        match self {
            Datalake::BlockSampled(_) => 0,
            Datalake::DynamicLayout(_) => 1,
            Datalake::Unknown => panic!("Unknown datalake type"),
        }
    }

    pub fn get_property(&self) -> Vec<u8> {
        match self {
            Datalake::BlockSampled(datalake) => datalake.get_property(),
            Datalake::DynamicLayout(_) => panic!("Unsupported datalake type"),
            Datalake::Unknown => panic!("Unknown datalake type"),
        }
    }
}
