//! This module defines the `ProcessedStorage` struct and its corresponding `ProcessedStorageInFelts` struct.

use super::mpt::ProcessedMPTProof;
use alloy_primitives::keccak256;
use serde::{Deserialize, Serialize};

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
