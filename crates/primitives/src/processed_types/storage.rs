//! This module defines the `ProcessedStorage` struct and its corresponding `ProcessedStorageInFelts` struct.

use alloy_primitives::keccak256;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

use super::{
    felt_vec_unit::FieldElementVectorUnit,
    mpt::{ProcessedMPTProof, ProcessedMPTProofInFelts},
    traits::IntoFelts,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedStorage {
    pub address: String,
    pub slot: String,
    pub storage_key: String,
    pub proofs: Vec<ProcessedMPTProof>,
}

impl ProcessedStorage {
    pub fn new(address: String, slot: String, proofs: Vec<ProcessedMPTProof>) -> Self {
        let storage_key = keccak256(&slot).to_string();
        ProcessedStorage {
            address,
            slot,
            storage_key,
            proofs,
        }
    }
}

impl IntoFelts for ProcessedStorage {
    type Output = ProcessedStorageInFelts;

    fn to_felts(&self) -> Self::Output {
        let address_chunk_result = FieldElementVectorUnit::from_hex_str(&self.address).unwrap();
        let slot_chunk_result = FieldElementVectorUnit::from_hex_str(&self.slot).unwrap();
        let storage_key = self.storage_key.clone();
        let proofs = self
            .proofs
            .iter()
            .map(|proof| proof.to_cairo_format())
            .collect();
        ProcessedStorageInFelts {
            address: address_chunk_result.felts,
            slot: slot_chunk_result.felts,
            storage_key,
            proofs,
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedStorageInFelts {
    // chunked address
    #[serde_as(as = "Vec<UfeHex>")]
    pub address: Vec<FieldElement>,
    // chunked storage slot
    #[serde_as(as = "Vec<UfeHex>")]
    pub slot: Vec<FieldElement>,
    pub storage_key: String,
    pub proofs: Vec<ProcessedMPTProofInFelts>,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::processed_types::{
        storage::{ProcessedStorage, ProcessedStorageInFelts},
        traits::IntoFelts,
    };

    #[test]
    fn test_storage_serde() {
        let processed_string = fs::read_to_string("fixtures/processed/storage.json").unwrap();
        let storages: ProcessedStorage = serde_json::from_str(&processed_string).unwrap();
        let storages_in_felts: ProcessedStorageInFelts = storages.to_felts();
        let string = serde_json::to_string_pretty(&storages_in_felts).unwrap();

        let json_file = fs::read_to_string("./fixtures/processed_in_felts/storage.json").unwrap();
        let expected: ProcessedStorageInFelts = serde_json::from_str(&json_file).unwrap();
        let expected_string = serde_json::to_string_pretty(&expected).unwrap();

        assert_eq!(string, expected_string);
    }
}
