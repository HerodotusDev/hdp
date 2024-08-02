use alloy::hex;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{
    primitives::block::header::RlpBlockHeader,
    primitives::serde::{deserialize_hex, serialize_hex},
};

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
#[serde_as]
pub struct ProcessedHeader {
    #[serde(serialize_with = "serialize_hex", deserialize_with = "deserialize_hex")]
    pub rlp: Vec<u8>,
    pub proof: ProcessedHeaderProof,
}

impl ProcessedHeader {
    pub fn new(rlp: RlpBlockHeader, leaf_idx: u64, mmr_path: Vec<String>) -> Self {
        let rlp = hex::decode(rlp.value).expect("Cannot decode RLP block header to bytes");
        let proof = ProcessedHeaderProof::new(leaf_idx, mmr_path);
        ProcessedHeader { rlp, proof }
    }
}
