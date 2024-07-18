use serde::{Deserialize, Serialize};

use super::collection::BlockSampledCollection;

/// [`BlockSampledDatalake`] is a struct that represents a block sampled datalake.
/// It contains the block range, the sampled property, and the increment.
///
/// The block range is inclusive, so the block range is from `block_range_start` to `block_range_end`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockSampledDatalake {
    pub chain_id: u64,
    /// The start of the block range
    pub block_range_start: u64,
    /// The end of the block range
    pub block_range_end: u64,
    /// The increment
    pub increment: u64,
    /// The sampled property
    pub sampled_property: BlockSampledCollection,
}

impl BlockSampledDatalake {
    pub fn new(
        chain_id: u64,
        block_range_start: u64,
        block_range_end: u64,
        increment: u64,
        sampled_property: BlockSampledCollection,
    ) -> Self {
        Self {
            chain_id,
            block_range_start,
            block_range_end,
            increment,
            sampled_property,
        }
    }
}
