use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedHeaderProof {
    pub leaf_idx: u64,
    pub mmr_path: Vec<String>,
}

impl ProcessedHeaderProof {
    pub fn new(leaf_idx: u64, mmr_path: Vec<String>) -> Self {
        ProcessedHeaderProof { leaf_idx, mmr_path }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedHeader {
    pub rlp: String,
    pub proof: ProcessedHeaderProof,
}

impl ProcessedHeader {
    pub fn new(rlp: String, leaf_idx: u64, mmr_path: Vec<String>) -> Self {
        let proof = ProcessedHeaderProof::new(leaf_idx, mmr_path);
        ProcessedHeader { rlp, proof }
    }
}
