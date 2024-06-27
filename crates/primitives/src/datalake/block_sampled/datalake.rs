use std::str::FromStr;

use super::collection::BlockSampledCollection;
use anyhow::Result;

/// [`BlockSampledDatalake`] is a struct that represents a block sampled datalake.
/// It contains the block range, the sampled property, and the increment.
///
/// The block range is inclusive, so the block range is from `block_range_start` to `block_range_end`
#[derive(Debug, Clone, PartialEq)]
pub struct BlockSampledDatalake {
    /// The start of the block range
    pub block_range_start: u64,
    /// The end of the block range
    pub block_range_end: u64,
    /// The sampled property
    pub sampled_property: BlockSampledCollection,
    /// The increment
    pub increment: u64,
}

impl BlockSampledDatalake {
    pub fn new(
        block_range_start: u64,
        block_range_end: u64,
        sampled_property: String,
        increment: u64,
    ) -> Result<Self> {
        Ok(Self {
            block_range_start,
            block_range_end,
            sampled_property: BlockSampledCollection::from_str(&sampled_property)?,
            increment,
        })
    }
}
