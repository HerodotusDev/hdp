//! Processed account type
//! This contains the processed account type and its conversion to cairo format.

use super::mpt::ProcessedMPTProof;
use alloy::primitives::{keccak256, Address};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedAccount {
    pub address: Address,
    pub account_key: String,
    pub proofs: Vec<ProcessedMPTProof>,
}

impl ProcessedAccount {
    pub fn new(address: Address, proofs: Vec<ProcessedMPTProof>) -> Self {
        let account_key = keccak256(address).to_string();
        ProcessedAccount {
            address,
            account_key,
            proofs,
        }
    }
}
