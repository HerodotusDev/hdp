//! This module defines the `ProcessedReceipt` struct and its corresponding `ProcessedReceiptInFelts` struct.

use crate::utils::tx_index_to_tx_key;
use alloy::primitives::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedReceipt {
    pub key: String,
    pub block_number: u64,
    pub proof: Vec<Bytes>,
}

impl ProcessedReceipt {
    pub fn new(index: u64, block_number: u64, proof: Vec<Bytes>) -> Self {
        let key = tx_index_to_tx_key(index);
        Self {
            key,
            block_number,
            proof,
        }
    }
}
