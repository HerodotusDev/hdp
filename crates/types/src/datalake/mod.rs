use self::{block_datalake::BlockDatalake, dynamic_layout_datalake::DynamicLayoutDatalake};

pub mod block_datalake;
pub mod datalake_base;
pub mod dynamic_layout_datalake;

/// Datatype for decoded datalakes
#[derive(Debug)]
pub enum DatalakeType {
    Block(BlockDatalake),
    DynamicLayout(DynamicLayoutDatalake),
    Unknown,
}
