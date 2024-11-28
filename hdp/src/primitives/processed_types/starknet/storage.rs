use alloy::hex;
use serde::{Deserialize, Serialize, Serializer};
use starknet_crypto::Felt;

use crate::{primitives::ChainId, provider::starknet::types::GetProofOutput};

fn serialize_chain_id<S>(chain_id: &ChainId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let bytes = chain_id.to_be_bytes();
    let hex_string = format!("0x{}", hex::encode(bytes));
    serializer.serialize_str(&hex_string)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedStorage {
    #[serde(serialize_with = "serialize_chain_id")]
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
