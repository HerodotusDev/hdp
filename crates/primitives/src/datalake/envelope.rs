use alloy::primitives::B256;
use anyhow::Result;

use super::{
    block_sampled::BlockSampledDatalake,
    datalake_type::{
        DatalakeType, BLOCK_SAMPLED_DATALAKE_TYPE_ID, TRANSACTIONS_IN_BLOCK_DATALAKE_TYPE_ID,
    },
    transactions::TransactionsInBlockDatalake,
    Datalake, DatalakeCollection,
};

/// Type of datalake
#[derive(Debug, Clone, PartialEq)]
pub enum DatalakeEnvelope {
    BlockSampled(BlockSampledDatalake),
    Transactions(TransactionsInBlockDatalake),
}

impl DatalakeEnvelope {
    pub fn to_index(&self) -> u8 {
        match self {
            DatalakeEnvelope::BlockSampled(_) => BLOCK_SAMPLED_DATALAKE_TYPE_ID,
            DatalakeEnvelope::Transactions(_) => TRANSACTIONS_IN_BLOCK_DATALAKE_TYPE_ID,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        match self {
            DatalakeEnvelope::BlockSampled(datalake) => datalake.encode(),
            DatalakeEnvelope::Transactions(datalake) => datalake.encode(),
        }
    }

    pub fn get_collection_type(&self) -> Box<dyn DatalakeCollection> {
        match self {
            DatalakeEnvelope::BlockSampled(datalake) => Box::new(datalake.sampled_property.clone()),
            DatalakeEnvelope::Transactions(datalake) => Box::new(datalake.sampled_property.clone()),
        }
    }

    pub fn get_commitment(&self) -> B256 {
        match self {
            DatalakeEnvelope::BlockSampled(datalake) => datalake.commit(),
            DatalakeEnvelope::Transactions(datalake) => datalake.commit(),
        }
    }

    pub fn from_index(value: u8, data: &[u8]) -> Result<Self> {
        match DatalakeType::from_index(value)? {
            DatalakeType::BlockSampled => Ok(DatalakeEnvelope::BlockSampled(
                BlockSampledDatalake::decode(data)?,
            )),
            DatalakeType::TransactionsInBlock => Ok(DatalakeEnvelope::Transactions(
                TransactionsInBlockDatalake::decode(data)?,
            )),
        }
    }

    pub fn get_datalake_type(&self) -> DatalakeType {
        match self {
            DatalakeEnvelope::BlockSampled(_) => DatalakeType::BlockSampled,
            DatalakeEnvelope::Transactions(_) => DatalakeType::TransactionsInBlock,
        }
    }
}
