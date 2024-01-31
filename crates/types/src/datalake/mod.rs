use self::{block_datalake::BlockDatalake, dynamic_layout_datalake::DynamicLayoutDatalake};

pub mod block_datalake;
pub mod compiler;
pub mod datalake_base;
pub mod dynamic_layout_datalake;

/// Datatype for decoded datalakes
#[derive(Debug, PartialEq)]
pub enum DatalakeType {
    Block(BlockDatalake),
    DynamicLayout(DynamicLayoutDatalake),
    Unknown,
}
