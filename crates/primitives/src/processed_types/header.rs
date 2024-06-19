use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

use crate::processed_types::felt_vec_unit::FieldElementVectorUnit;

use super::traits::IntoFelts;

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

impl IntoFelts for ProcessedHeader {
    type Output = ProcessedHeaderInFelts;

    fn to_felts(&self) -> Self::Output {
        let felts_unit = FieldElementVectorUnit::from_hex_str(&format!("0x{}", &self.rlp)).unwrap();
        let proof = self.proof.clone();
        ProcessedHeaderInFelts {
            rlp: felts_unit.felts,
            rlp_bytes_len: felts_unit.bytes_len,
            proof: ProcessedHeaderProof {
                leaf_idx: proof.leaf_idx,
                mmr_path: proof.mmr_path,
            },
        }
    }
}

/// HeaderFormatted is the formatted version of Header
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedHeaderInFelts {
    #[serde_as(as = "Vec<UfeHex>")]
    pub rlp: Vec<FieldElement>,
    /// rlp_bytes_len is the byte( 8 bit ) length from rlp string
    pub rlp_bytes_len: u64,
    pub proof: ProcessedHeaderProof,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_header_serde() {
        let processed_string = fs::read_to_string("fixtures/processed/header.json").unwrap();
        let headers: ProcessedHeader = serde_json::from_str(&processed_string).unwrap();
        let headers_in_felts: ProcessedHeaderInFelts = headers.to_felts();
        let string = serde_json::to_string_pretty(&headers_in_felts).unwrap();

        let json_file = fs::read_to_string("./fixtures/processed_in_felts/header.json").unwrap();
        let expected: ProcessedHeaderInFelts = serde_json::from_str(&json_file).unwrap();
        let expected_string = serde_json::to_string_pretty(&expected).unwrap();

        assert_eq!(string, expected_string);
    }
}
