use std::{fmt::Display, str::FromStr};

use self::datalake_type::DatalakeType;
use alloy::primitives::{B256, U256};
use anyhow::Result;

pub mod block_sampled;
pub mod datalake_type;
pub mod envelope;
pub mod task;
pub mod transactions;

pub trait DatalakeCollection {
    fn to_index(&self) -> u8;
    fn serialize(&self) -> Result<Vec<u8>>;
    fn deserialize(encoded: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

/// Define the common trait for all datalakes
pub trait Datalake {
    fn get_datalake_type(&self) -> DatalakeType;
    fn encode(&self) -> Result<Vec<u8>>;
    fn commit(&self) -> B256;
    fn decode(encoded: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

pub trait DatalakeField: FromStr + Display {
    fn from_index(index: u8) -> Result<Self>
    where
        Self: Sized;
    fn to_index(&self) -> u8;
    fn decode_field_from_rlp(&self, rlp: &[u8]) -> U256;
}
