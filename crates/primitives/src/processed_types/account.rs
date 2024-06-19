//! Processed account type
//! This contains the processed account type and its conversion to cairo format.

use super::mpt::ProcessedMPTProof;
use alloy_primitives::keccak256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedAccount {
    pub address: String,
    pub account_key: String,
    pub proofs: Vec<ProcessedMPTProof>,
}

impl ProcessedAccount {
    pub fn new(address: String, proofs: Vec<ProcessedMPTProof>) -> Self {
        let account_key = keccak256(&address).to_string();
        ProcessedAccount {
            address,
            account_key,
            proofs,
        }
    }
}
