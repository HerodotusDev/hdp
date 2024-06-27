use alloy::primitives::B256;
use anyhow::Result;

use crate::datalake::datalake_type::DatalakeType;

/// Define the common trait for all datalakes
pub trait DatalakeCodecs {
    fn get_datalake_type(&self) -> DatalakeType;
    fn encode(&self) -> Result<Vec<u8>>;
    fn commit(&self) -> B256;
    fn decode(encoded: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

pub trait DatalakeBatchCodecs {
    fn encode(&self) -> Result<Vec<u8>>;
    fn decode(encoded: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

pub trait ComputeCodecs {
    fn encode(&self) -> Result<Vec<u8>>;
    fn decode(encoded: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

pub trait DatalakeComputeCodecs {
    fn decode(encoded_datalake: &[u8], encoded_compute: &[u8]) -> Result<Self>
    where
        Self: std::marker::Sized;
    fn encode(&self) -> Result<Vec<u8>>;
    fn commit(&self) -> B256;
}

// we need this for now because datalake and compute is seperate bytes
pub trait DatalakeComputeBatchCodecs {
    fn decode(encoded_datalake: &[u8], encoded_compute: &[u8]) -> Result<Self>
    where
        Self: std::marker::Sized;
    fn encode(&self) -> Result<(Vec<u8>, Vec<u8>)>;
    fn commit(&self) -> B256;
}
