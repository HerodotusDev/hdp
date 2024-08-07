use alloy::primitives::U256;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

use self::{compute::Computation, envelope::DatalakeEnvelope};

pub mod block_sampled;
pub mod compute;
pub mod datalake_type;
pub mod envelope;
pub mod transactions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatalakeCompute {
    pub datalake: DatalakeEnvelope,
    pub compute: Computation,
}

impl DatalakeCompute {
    pub fn new(datalake: DatalakeEnvelope, compute: Computation) -> Self {
        Self { datalake, compute }
    }
}

pub trait DatalakeCollection {
    fn to_index(&self) -> u8;
    fn serialize(&self) -> Result<Vec<u8>>;
    fn deserialize(encoded: &[u8]) -> Result<Self>
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
