use serde::{Deserialize, Serialize};
use starknet_crypto::Felt;

use crate::{primitives::ChainId, provider::starknet::types::GetProofOutput};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedStorage {
    pub chain_id: ChainId,
    pub block_number: u64,
    pub contract_address: Felt,
    pub storage_addresses: Vec<Felt>,
    pub proof: GetProofOutput,
}

impl ProcessedStorage {
    pub fn new(
        chain_id: ChainId,
        block_number: u64,
        contract_address: Felt,
        storage_addresses: Vec<Felt>,
        proof: GetProofOutput,
    ) -> Self {
        Self {
            chain_id,
            block_number,
            contract_address,
            storage_addresses,
            proof,
        }
    }
}
