use self::{
    base::{DatalakeBase, Derivable},
    block_sampled::BlockSampledDatalake,
    dynamic_layout::DynamicLayoutDatalake,
};

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
