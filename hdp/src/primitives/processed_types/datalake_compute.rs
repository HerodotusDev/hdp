use alloy::primitives::{Bytes, B256, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedDatalakeCompute {
    /// encoded computational task
    pub encoded_task: Bytes,
    /// computational task commitment
    pub task_commitment: B256,
    /// raw evaluation result of target compiled task
    pub compiled_result: U256,
    /// results merkle tree's entry value
    pub result_commitment: B256,
    /// merkle proof for tasks
    pub task_proof: Vec<B256>,
    /// merkle proof for results
    pub result_proof: Vec<B256>,
    /// encoded datalake
    pub encoded_datalake: Bytes,
    // ex. block sampled datalake / transaction datalake
    pub datalake_type: u8,
    // ex. "header", "account", "storage"
    pub property_type: u8,
}

impl ProcessedDatalakeCompute {
    pub fn new(
        encoded_task: Bytes,
        task_commitment: B256,
        compiled_result: U256,
        result_commitment: B256,
        task_proof: Vec<B256>,
        result_proof: Vec<B256>,
        encoded_datalake: Bytes,
        datalake_type: u8,
        property_type: u8,
    ) -> Self {
        Self {
            encoded_task,
            task_commitment,
            compiled_result,
            result_commitment,
            task_proof,
            result_proof,
            encoded_datalake,
            datalake_type,
            property_type,
        }
    }
}
