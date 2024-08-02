//! This module defines the `ProcessedStorage` struct and its corresponding `ProcessedStorageInFelts` struct.

use alloy::primitives::StorageKey;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

use crate::primitives::processed_types::storage::ProcessedStorage as BaseProcessedStorage;

use super::{felt_vec_unit::FieldElementVectorUnit, mpt::ProcessedMPTProof, traits::AsCairoFormat};

impl AsCairoFormat for BaseProcessedStorage {
    type Output = ProcessedStorage;

    fn as_cairo_format(&self) -> Self::Output {
        let address_chunk_result =
            FieldElementVectorUnit::from_bytes(self.address.as_ref()).unwrap();
        let slot_chunk_result = FieldElementVectorUnit::from_bytes(self.slot.as_ref()).unwrap();
        let storage_key = self.storage_key;
        let proofs = self
            .proofs
            .iter()
            .map(|proof| proof.as_cairo_format())
            .collect();
        ProcessedStorage {
            address: address_chunk_result.felts,
            slot: slot_chunk_result.felts,
            storage_key,
            proofs,
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedStorage {
    // chunked address
    #[serde_as(as = "Vec<UfeHex>")]
    pub address: Vec<FieldElement>,
    // chunked storage slot
    #[serde_as(as = "Vec<UfeHex>")]
    pub slot: Vec<FieldElement>,
    pub storage_key: StorageKey,
    pub proofs: Vec<ProcessedMPTProof>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_serde() {
        let processed_string =
            include_str!("../../../../../fixtures/primitives/processed/storage.json");
        let storages: BaseProcessedStorage = serde_json::from_str(processed_string).unwrap();
        let storages_in_felts: ProcessedStorage = storages.as_cairo_format();
        let string = serde_json::to_string_pretty(&storages_in_felts).unwrap();

        let json_file =
            include_str!("../../../../../fixtures/primitives/processed_in_felts/storage.json");
        let expected: ProcessedStorage = serde_json::from_str(json_file).unwrap();
        let expected_string = serde_json::to_string_pretty(&expected).unwrap();

        assert_eq!(string, expected_string);
    }
}
