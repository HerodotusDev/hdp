use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedMPTProof {
    pub block_number: u64,
    pub proof: Vec<String>,
}

impl ProcessedMPTProof {
    pub fn new(block_number: u64, proof: Vec<String>) -> Self {
        ProcessedMPTProof {
            block_number,
            proof,
        }
    }
}
