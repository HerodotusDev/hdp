use serde::{Deserialize, Serialize};

use crate::primitives::{task::datalake::envelope::default_increment, ChainId};

use super::collection::BlockSampledCollection;

/// [`BlockSampledDatalake`] is a struct that represents a block sampled datalake.
/// It contains chain id, block range, the sampled property, and the increment.
///
/// Inclusive block range: [block_range_start..block_range_end]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockSampledDatalake {
    /// Chain id of the datalake
    pub chain_id: ChainId,
    /// The start of the block range
    pub block_range_start: u64,
    /// The end of the block range
    pub block_range_end: u64,
    /// The increment. Defaults to 1 if not present.
    #[serde(default = "default_increment")]
    pub increment: u64,
    /// The sampled property
    pub sampled_property: BlockSampledCollection,
}

impl BlockSampledDatalake {
    pub fn new(
        chain_id: ChainId,
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
