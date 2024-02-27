use alloy_primitives::FixedBytes;
use serde::{Deserialize, Serialize};
use starknet::core::types::FieldElement;

//==============================================================================
// for int type, use uint type
// for string type, if formatted, use chunk[] to store field elements

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Uint256 {
    pub low: u128,
    pub high: u128,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct HeaderProof {
    pub leaf_idx: u64,
    pub mmr_path: Vec<String>,
}

/// HeaderProofFormatted is the formatted version of HeaderProof
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeaderProofFormatted {
    pub leaf_idx: u64,
    /// mmr_path_len is the length of mmr_path
    pub mmr_path_len: u64,
    pub mmr_path: Vec<Vec<FieldElement>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Header {
    pub rlp: String,
    pub proof: HeaderProof,
}

/// HeaderFormatted is the formatted version of Header
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeaderFormatted {
    pub rlp: Vec<FieldElement>,
    /// rlp_bytes_len is the byte( 8 bit ) length from rlp string
    pub rlp_bytes_len: u64,
    pub proof: HeaderProof,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Account {
    pub address: String,
    // U256 type
    pub account_key: String,
    pub proofs: Vec<MPTProof>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountFormatted {
    pub address: Vec<FieldElement>,
    pub account_key: Uint256,
    /// proofs_len is the length of proofs
    pub proofs_len: u64,
    pub proofs: Vec<MPTProof>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct MPTProof {
    pub block_number: u64,
    pub proof: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MPTProofFormatted {
    pub block_number: u64,
    /// proof_len is the length of proof
    pub proof_len: u64,
    /// proof_bytes_len is the byte( 8 bit ) length from each proof string
    pub proof_bytes_len: Vec<u64>,
    pub proof: Vec<Vec<FieldElement>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MMRMeta {
    pub id: u64,
    pub root: String,
    pub size: u64,
    // hex encoded
    pub peaks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MMRMetaFormatted {
    pub id: u64,
    pub root: Vec<FieldElement>,
    pub size: u64,
    /// peaks_len is the length of peaks
    pub peaks_len: u64,
    pub peaks: Vec<Vec<FieldElement>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Storage {
    pub address: String,
    // U256 type
    pub account_key: String,
    // U256 type
    pub storage_key: String,
    pub proofs: Vec<MPTProof>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageFormatted {
    pub address: Vec<FieldElement>,
    pub account_key: Uint256,
    pub storage_key: Uint256,
    /// proofs_len is the length of proofs
    pub proofs_len: u64,
    pub proofs: Vec<MPTProof>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub computational_task: String,
    pub task_commitment: String,
    pub result: String,
    pub task_proof: Vec<FixedBytes<32>>,
    pub result_proof: Vec<FixedBytes<32>>,
    pub datalake: String,
    pub datalake_type: u8,
    pub property: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskFormatted {
    computational_bytes_len: u64,
    computational_task: Vec<FieldElement>,
    datalake_bytes_len: u64,
    datalake: Vec<FieldElement>,
    datalake_type: u8,
    property_len: u8,
    pub property: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessedResult {
    // U256 type
    pub results_root: String,
    // U256 type
    pub tasks_root: String,
    pub headers: Vec<Header>,
    pub mmr: MMRMeta,
    pub accounts: Vec<Account>,
    pub storages: Vec<Storage>,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultFormatted {
    pub results_root: Uint256,
    pub tasks_root: Uint256,
    pub headers: Vec<HeaderFormatted>,
    pub mmr: MMRMetaFormatted,
    pub accounts: Vec<AccountFormatted>,
    pub storages: Vec<StorageFormatted>,
    pub tasks: Vec<TaskFormatted>,
}
