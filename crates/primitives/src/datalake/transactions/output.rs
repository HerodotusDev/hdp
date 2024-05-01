use serde::{Deserialize, Serialize};

use crate::datalake::output::{
    hex_to_8_byte_chunks_little_endian, split_big_endian_hex_into_parts,
    split_little_endian_hex_into_parts, CairoFormattedChunkResult, Uint256,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Transaction {
    // U256 type
    pub key: String,
    pub block_number: u64,
    pub proof: Vec<String>,
}

impl Transaction {
    pub(crate) fn to_cairo_format(&self) -> TransactionFormatted {
        let tx_key = split_big_endian_hex_into_parts(&self.key);
        let proof_chunk_result: Vec<CairoFormattedChunkResult> = self
            .proof
            .iter()
            .map(|proof| hex_to_8_byte_chunks_little_endian(proof))
            .collect();

        let proof_bytes_len = proof_chunk_result.iter().map(|x| x.chunks_len).collect();
        let proof_result: Vec<Vec<String>> = proof_chunk_result
            .iter()
            .map(|x| x.chunks.clone())
            .collect();
        TransactionFormatted {
            key: tx_key,
            block_number: self.block_number,
            proof_bytes_len,
            proof: proof_result,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub(crate) struct TransactionFormatted {
    // U256 type
    pub key: Uint256,
    pub block_number: u64,
    /// proof_bytes_len is the byte( 8 bit ) length from each proof string
    pub proof_bytes_len: Vec<u64>,
    pub proof: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct TransactionReceipt {
    // U256 type
    pub key: String,
    pub block_number: u64,
    pub proof: Vec<String>,
}

impl TransactionReceipt {
    pub(crate) fn to_cairo_format(&self) -> TransactionReceiptFormatted {
        let tx_key = split_little_endian_hex_into_parts(&self.key);
        let proof_chunk_result: Vec<CairoFormattedChunkResult> = self
            .proof
            .iter()
            .map(|proof| hex_to_8_byte_chunks_little_endian(proof))
            .collect();

        let proof_bytes_len = proof_chunk_result.iter().map(|x| x.chunks_len).collect();
        let proof_result: Vec<Vec<String>> = proof_chunk_result
            .iter()
            .map(|x| x.chunks.clone())
            .collect();
        TransactionReceiptFormatted {
            key: tx_key,
            block_number: self.block_number,
            proof_bytes_len,
            proof: proof_result,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub(crate) struct TransactionReceiptFormatted {
    // U256 type
    pub key: Uint256,
    pub block_number: u64,
    /// proof_bytes_len is the byte( 8 bit ) length from each proof string
    pub proof_bytes_len: Vec<u64>,
    pub proof: Vec<Vec<String>>,
}
