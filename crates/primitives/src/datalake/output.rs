use alloy_primitives::{
    hex::{self, FromHex},
    FixedBytes,
};
use serde::{Deserialize, Serialize};

use crate::utils::bytes_to_hex_string;

//==============================================================================
// for int type, use uint type
// for string type, if formatted, use chunk[] to store field elements

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Uint256 {
    pub low: String,
    pub high: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct HeaderProof {
    pub leaf_idx: u64,
    pub mmr_path: Vec<String>,
}

/// HeaderProofFormatted is the formatted version of HeaderProof
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct HeaderProofFormatted {
    pub leaf_idx: u64,
    // mmr_path is encoded with poseidon
    pub mmr_path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Header {
    pub rlp: String,
    pub proof: HeaderProof,
}

impl Header {
    pub fn to_cairo_format(&self) -> HeaderFormatted {
        let chunk_result = hex_to_8_byte_chunks_little_endian(&self.rlp);
        let proof = self.proof.clone();
        HeaderFormatted {
            rlp: chunk_result.chunks,
            rlp_bytes_len: chunk_result.chunks_len,
            proof: HeaderProofFormatted {
                leaf_idx: proof.leaf_idx,
                mmr_path: proof.mmr_path,
            },
        }
    }
}

/// HeaderFormatted is the formatted version of Header
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct HeaderFormatted {
    pub rlp: Vec<String>,
    /// rlp_bytes_len is the byte( 8 bit ) length from rlp string
    pub rlp_bytes_len: u64,
    pub proof: HeaderProofFormatted,
}

pub fn hex_to_8_byte_chunks_little_endian(input_hex: &str) -> CairoFormattedChunkResult {
    // Convert hex string to bytes
    let bytes = hex::decode(input_hex).expect("Invalid hex input");
    let chunks_len = bytes.len() as u64;
    // Process bytes into 8-byte chunks and convert to little-endian u64, then to hex strings
    let chunks = bytes
        .chunks(8)
        .map(|chunk| {
            let mut arr = [0u8; 8];
            let len = chunk.len();
            arr[..len].copy_from_slice(chunk);
            let le_int = u64::from_le_bytes(arr);
            format!("0x{:x}", le_int)
        })
        .collect();

    CairoFormattedChunkResult { chunks, chunks_len }
}

pub struct CairoFormattedChunkResult {
    pub chunks: Vec<String>,
    pub chunks_len: u64,
}

pub fn split_little_endian_hex_into_parts(hex_str: &str) -> Uint256 {
    let clean_hex = hex_str.trim_start_matches("0x");
    let mut fix_hex: FixedBytes<32> = FixedBytes::from_hex(clean_hex).unwrap();
    fix_hex.reverse();

    let high_part = fix_hex[..16].to_vec();
    let low_part = fix_hex[16..].to_vec();

    Uint256 {
        high: bytes_to_hex_string(&high_part),
        low: bytes_to_hex_string(&low_part),
    }
}

pub fn split_big_endian_hex_into_parts(hex_str: &str) -> Uint256 {
    let clean_hex = hex_str.trim_start_matches("0x");
    let padded_hex = format!("{:0>64}", clean_hex);
    let (high_part, low_part) = padded_hex.split_at(32);
    Uint256 {
        high: format!("0x{}", high_part),
        low: format!("0x{}", low_part),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Task {
    /// encoded computational task
    pub encoded_task: String,
    /// computational task commitment
    pub task_commitment: String,
    /// raw evaluation result of target compiled task
    pub compiled_result: String,
    /// results merkle tree's entry value
    pub result_commitment: String,
    pub task_proof: Vec<FixedBytes<32>>,
    pub result_proof: Vec<FixedBytes<32>>,
    /// encoded datalake
    pub encoded_datalake: String,
    // ex. dynamic datalake / block sampled datalake
    pub datalake_type: u8,
    // ex. "header", "account", "storage"
    pub property_type: u8,
}

impl Task {
    pub fn to_cairo_format(&self) -> TaskFormatted {
        let computational_task_chunk_result =
            hex_to_8_byte_chunks_little_endian(&self.encoded_task);
        let datalake_chunk_result = hex_to_8_byte_chunks_little_endian(&self.encoded_datalake);
        TaskFormatted {
            task_bytes_len: computational_task_chunk_result.chunks_len,
            encoded_task: computational_task_chunk_result.chunks,
            datalake_bytes_len: datalake_chunk_result.chunks_len,
            encoded_datalake: datalake_chunk_result.chunks,
            datalake_type: self.datalake_type,
            property_type: self.property_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskFormatted {
    pub task_bytes_len: u64,
    pub encoded_task: Vec<String>,
    pub datalake_bytes_len: u64,
    pub encoded_datalake: Vec<String>,
    pub datalake_type: u8,
    pub property_type: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct MPTProof {
    pub block_number: u64,
    pub proof: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct MPTProofFormatted {
    pub block_number: u64,
    /// proof_bytes_len is the byte( 8 bit ) length from each proof string
    pub proof_bytes_len: Vec<u64>,
    pub proof: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct MMRMeta {
    pub id: u64,
    pub root: String,
    pub size: u64,
    // hex encoded
    pub peaks: Vec<String>,
}