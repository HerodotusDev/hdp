use alloy_primitives::FixedBytes;
use serde::{Deserialize, Serialize};
use starknet::core::types::FieldElement;

//==============================================================================
// for int type, use uint type
// for string type, if formatted, use chunk[] to store field elements

//TODO:
// Vec<FieldElement> => def bytes_to_8_bytes_chunks_little(input_bytes):
// # Split the input_bytes into 8-byte chunks
// byte_chunks = [input_bytes[i : i + 8] for i in range(0, len(input_bytes), 8)]
// # Convert each chunk to little-endian integers
// little_endian_ints = [
//     int.from_bytes(chunk, byteorder="little") for chunk in byte_chunks
// return hex(little_endian_ints)

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
    // mmr_path is encoded with poseidon
    pub mmr_path: Vec<FieldElement>,
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
    pub proof: HeaderProofFormatted,
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
    pub proofs: Vec<MPTProofFormatted>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct MPTProof {
    pub block_number: u64,
    pub proof: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MPTProofFormatted {
    pub block_number: u64,
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
    pub root: FieldElement,
    pub size: u64,
    // Peaks are encoded with poseidon
    pub peaks: Vec<FieldElement>,
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
    // storage key == storage slot
    pub storage_key: Uint256,
    pub proofs: Vec<MPTProofFormatted>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub computational_task: String,
    pub task_commitment: String,
    pub result: String,
    pub task_proof: Vec<FixedBytes<32>>,
    pub result_proof: Vec<FixedBytes<32>>,
    pub datalake: String,
    // ex. dynamic datalake / block sampled datalake
    pub datalake_type: u8,
    // ex. "header", "account", "storage"
    pub property_id: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskFormatted {
    pub computational_bytes_len: u64,
    pub computational_task: Vec<FieldElement>,
    pub datalake_bytes_len: u64,
    pub datalake: Vec<FieldElement>,
    pub datalake_type: u8,
    pub property_id: u8,
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
