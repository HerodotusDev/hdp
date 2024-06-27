use anyhow::Result;

use crate::solidity_types::traits::DatalakeCodecs;

use super::{
    block_sampled::BlockSampledDatalake,
    datalake_type::{
        DatalakeType, BLOCK_SAMPLED_DATALAKE_TYPE_ID, TRANSACTIONS_IN_BLOCK_DATALAKE_TYPE_ID,
    },
    transactions::TransactionsInBlockDatalake,
    DatalakeCollection,
};

pub type BatchedDatalakes = Vec<DatalakeEnvelope>;

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

    pub fn get_collection_type(&self) -> Box<dyn DatalakeCollection> {
        match self {
            DatalakeEnvelope::BlockSampled(datalake) => Box::new(datalake.sampled_property.clone()),
            DatalakeEnvelope::Transactions(datalake) => Box::new(datalake.sampled_property.clone()),
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
}
