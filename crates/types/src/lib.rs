use datalake::{block_sampled::BlockSampledDatalake, dynamic_layout::DynamicLayoutDatalake};

pub mod compiler;
pub mod datalake;
pub mod task;

#[derive(Debug, Clone, PartialEq)]
pub enum Datalake {
    BlockSampled(BlockSampledDatalake),
    DynamicLayout(DynamicLayoutDatalake),
    Unknown,
}
