use alloy::primitives::B256;
use anyhow::Result;

use crate::primitives::task::datalake::datalake_type::DatalakeType;

/// Define the common trait for datalake
///
/// Common for both BlockSampled and TransactionsInBlock
pub trait DatalakeCodecs {
    fn get_datalake_type(&self) -> DatalakeType;
    fn encode(&self) -> Result<Vec<u8>>;
    fn commit(&self) -> B256;
    fn decode(encoded: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

pub trait Codecs {
    fn encode(&self) -> Result<Vec<u8>>;
    fn decode(encoded: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

/// Codecs for [`DatalakeCompute`]
pub trait DatalakeComputeCodecs {
    fn decode(encoded_datalake: &[u8], encoded_compute: &[u8]) -> Result<Self>
    where
        Self: std::marker::Sized;
    fn encode(&self) -> Result<Vec<u8>>;
    fn commit(&self) -> B256;
}

/// Codecs for [`BatchedDatalakeCompute`]
pub trait BatchedDatalakeComputeCodecs {
    fn decode(encoded_datalake: &[u8], encoded_compute: &[u8]) -> Result<Self>
    where
        Self: std::marker::Sized;
    fn encode(&self) -> Result<(Vec<u8>, Vec<u8>)>;
}
