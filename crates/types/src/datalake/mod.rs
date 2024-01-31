use self::{block_datalake::BlockDatalake, dynamic_layout_datalake::DynamicLayoutDatalake};

pub mod block_datalake;
pub mod datalake_base;
pub mod dynamic_layout_datalake;
pub mod helpers;

/// Datatype for decoded datalakes
#[derive(Debug, PartialEq)]
pub enum DatalakeType {
    Block(BlockDatalake),
    DynamicLayout(DynamicLayoutDatalake),
    Unknown,
}
