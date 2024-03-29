use self::datalake_type::DatalakeType;
use anyhow::Result;

pub mod block_sampled;
pub mod datalake_type;
pub mod envelope;
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
    fn encode(&self) -> Result<String>;
    fn commit(&self) -> String;
    fn decode(encoded: &str) -> Result<Self>
    where
        Self: Sized;
}

pub trait DatalakeField {
    fn from_index(index: u8) -> Result<Self>
    where
        Self: Sized;
    fn to_index(&self) -> u8;
    fn as_str(&self) -> &'static str;
    fn decode_field_from_rlp(&self, rlp: &str) -> String;
}
