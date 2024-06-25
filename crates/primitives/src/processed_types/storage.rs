//! This module defines the `ProcessedStorage` struct and its corresponding `ProcessedStorageInFelts` struct.

use super::mpt::ProcessedMPTProof;
use alloy::primitives::{keccak256, Address, StorageKey, B256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedStorage {
    pub address: Address,
    pub slot: B256,
    pub storage_key: StorageKey,
    pub proofs: Vec<ProcessedMPTProof>,
}

impl ProcessedStorage {
    pub fn new(address: Address, slot: B256, proofs: Vec<ProcessedMPTProof>) -> Self {
        let storage_key = keccak256(slot);
        ProcessedStorage {
            address,
            slot,
            storage_key,
            proofs,
        }
    }
}
