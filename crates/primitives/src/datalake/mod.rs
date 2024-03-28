pub mod block_sampled;
pub mod datalake_type;
pub mod envelope;
pub mod transactions;

pub trait DatalakeCollection {
    fn to_index(&self) -> u8;
}
