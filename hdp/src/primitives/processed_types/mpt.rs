use alloy::primitives::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedMPTProof {
    pub block_number: u64,
    pub proof: Vec<Bytes>,
}

impl ProcessedMPTProof {
    pub fn new(block_number: u64, proof: Vec<Bytes>) -> Self {
        ProcessedMPTProof {
            block_number,
            proof,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_mpt_proof() {
        let processed_string = include_str!("../../../../fixtures/primitives/processed/mpt.json");
        let processed_mpt: ProcessedMPTProof = serde_json::from_str(processed_string).unwrap();
        assert_eq!(processed_mpt.block_number, 5244634);
        assert_eq!(processed_mpt.proof.len(), 8);
    }
}
