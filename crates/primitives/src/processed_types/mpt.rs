use super::felt_vec_unit::FieldElementVectorUnit;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

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

    pub fn to_cairo_format(&self) -> ProcessedMPTProofInFelts {
        let proof_felts: Vec<FieldElementVectorUnit> = self
            .proof
            .iter()
            .map(|proof| FieldElementVectorUnit::from_hex_str(proof).unwrap())
            .collect();

        let proof_bytes_len = proof_felts.iter().map(|f| f.bytes_len).collect();
        let proof_result: Vec<Vec<FieldElement>> =
            proof_felts.iter().map(|f| f.felts.clone()).collect();
        ProcessedMPTProofInFelts {
            block_number: self.block_number,
            proof_bytes_len,
            proof: proof_result,
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedMPTProofInFelts {
    pub block_number: u64,
    /// proof_bytes_len is the byte( 8 bit ) length from each proof string
    pub proof_bytes_len: Vec<u64>,
    #[serde_as(as = "Vec<Vec<UfeHex>>")]
    pub proof: Vec<Vec<FieldElement>>,
}
