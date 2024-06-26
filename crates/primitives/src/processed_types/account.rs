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
        // TODO: actually this is account trie leaf to be more accurate
        let account_trie_leaf = keccak256(address).to_string();
        ProcessedAccount {
            address,
            account_key: account_trie_leaf,
            proofs,
        }
    }
}
