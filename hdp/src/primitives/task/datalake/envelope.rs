use serde::{Deserialize, Serialize};

use crate::{
    preprocessor::compile::datalake::fetchable::{FetchError, Fetchable, FetchedDatalake},
    provider::ProofProvider,
};

use super::{
    block_sampled::BlockSampledDatalake, transactions::TransactionsInBlockDatalake,
    DatalakeCollection,
};

pub type BatchedDatalakes = Vec<DatalakeEnvelope>;

/// Envelope for datalake
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DatalakeEnvelope {
    BlockSampled(BlockSampledDatalake),
    TransactionsInBlock(TransactionsInBlockDatalake),
}

impl DatalakeEnvelope {
    pub fn get_collection_type(&self) -> Box<dyn DatalakeCollection> {
        match self {
            DatalakeEnvelope::BlockSampled(datalake) => Box::new(datalake.sampled_property.clone()),
            DatalakeEnvelope::TransactionsInBlock(datalake) => {
                Box::new(datalake.sampled_property.clone())
            }
        }
    }
}

impl<P> Fetchable<P> for DatalakeEnvelope
where
    P: ProofProvider,
{
    async fn fetch(&self, provider: P) -> Result<FetchedDatalake, FetchError> {
        match self {
            DatalakeEnvelope::BlockSampled(datalake) => datalake.fetch(provider).await,
            DatalakeEnvelope::TransactionsInBlock(datalake) => datalake.fetch(provider).await,
        }
    }
}

/// Default increment for datalake
pub fn default_increment() -> u64 {
    1
}
