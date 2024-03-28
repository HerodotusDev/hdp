use self::datalake_type::DatalakeType;
use anyhow::Result;

pub mod block_sampled;
pub mod datalake_type;
pub mod envelope;
pub mod transactions;

pub trait DatalakeCollection {
    fn to_index(&self) -> u8;
}

/// Define the common trait for all datalakes
pub trait Datalake {
    fn get_datalake_type(&self) -> DatalakeType;
    fn encode(&self) -> Result<String>;
    fn commit(&self) -> String;
    fn decode(encoded: &str) -> Result<Self>
    where
        Self: Sized;
}
