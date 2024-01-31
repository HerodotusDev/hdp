use block_datalake::BlockDatalake;
use dynamic_layout_datalake::DynamicLayoutDatalake;

pub mod block_datalake;
pub mod compiler;
pub mod datalake_base;
pub mod dynamic_layout_datalake;
pub mod task;

#[derive(Debug, Clone, PartialEq)]
pub enum DatalakeType {
    Block(BlockDatalake),
    DynamicLayout(DynamicLayoutDatalake),
    Unknown,
}
