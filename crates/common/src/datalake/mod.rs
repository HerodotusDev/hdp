use self::{block_sampled::BlockSampledDatalake, dynamic_layout::DynamicLayoutDatalake};

pub mod base;
pub mod block_sampled;
pub mod dynamic_layout;

#[derive(Debug, Clone, PartialEq)]
pub enum Datalake {
    BlockSampled(BlockSampledDatalake),
    DynamicLayout(DynamicLayoutDatalake),
    Unknown,
}
