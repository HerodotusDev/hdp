use super::{
    block_sampled::BlockSampledDatalake, transactions::TransactionsInBlockDatalake,
    DatalakeCollection,
};

pub type BatchedDatalakes = Vec<DatalakeEnvelope>;

/// Envelope for datalake
#[derive(Debug, Clone, PartialEq)]
pub enum DatalakeEnvelope {
    BlockSampled(BlockSampledDatalake),
    Transactions(TransactionsInBlockDatalake),
}

impl DatalakeEnvelope {
    pub fn get_collection_type(&self) -> Box<dyn DatalakeCollection> {
        match self {
            DatalakeEnvelope::BlockSampled(datalake) => Box::new(datalake.sampled_property.clone()),
            DatalakeEnvelope::Transactions(datalake) => Box::new(datalake.sampled_property.clone()),
        }
    }
}
