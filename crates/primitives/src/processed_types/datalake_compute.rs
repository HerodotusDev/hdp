use alloy_primitives::B256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedDatalakeCompute {
    /// encoded computational task
    pub encoded_task: String,
    /// computational task commitment
    pub task_commitment: String,
    /// raw evaluation result of target compiled task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiled_result: Option<String>,
    /// results merkle tree's entry value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_commitment: Option<String>,
    pub task_proof: Vec<B256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_proof: Option<Vec<B256>>,
    /// encoded datalake
    pub encoded_datalake: String,
    // ex. block sampled datalake / transaction datalake
    pub datalake_type: u8,
    // ex. "header", "account", "storage"
    pub property_type: u8,
}
