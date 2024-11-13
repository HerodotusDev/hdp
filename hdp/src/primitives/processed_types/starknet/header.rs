use serde::{Deserialize, Serialize};
use starknet_crypto::Felt;

use crate::primitives::processed_types::header::ProcessedHeaderProof;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedHeader {
    pub fields: Vec<Felt>,
    pub proof: ProcessedHeaderProof,
}

impl ProcessedHeader {
    pub fn new(fields: Vec<Felt>, leaf_idx: u64, mmr_path: Vec<String>) -> Self {
        let proof = ProcessedHeaderProof::new(leaf_idx, mmr_path);
        Self { fields, proof }
    }
}
