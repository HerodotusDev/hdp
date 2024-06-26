//! Indexer client for fetching MMR and headers proof from Herodotus Indexer
//!
//! For more information, see: https://rs-indexer.api.herodotus.cloud/swagger
//!
//! How to use:
//! ```rust
//! use hdp_provider::indexer::Indexer;
//! use hdp_provider::errors::IndexerError;
//!
//! async fn call_indexer(chain_id: u64, block_range_start: u64, block_range_end: u64) -> Result<(), IndexerError> {
//!     let indexer = Indexer::new(chain_id);
//!     let response = indexer.get_headers_proof(block_range_start, block_range_end).await?;
//!     Ok(())
//! }
//! ```

use alloy::primitives::BlockNumber;
use hdp_primitives::block::header::{
    MMRDataFromNewIndexer, MMRFromNewIndexer, MMRMetaFromNewIndexer, MMRProofFromNewIndexer,
};
use reqwest::Client;
use serde_json::{from_value, Value};
use std::collections::HashMap;
use thiserror::Error;
use tracing::error;

pub const HERODOTUS_RS_INDEXER_URL: &str =
    "https://rs-indexer.api.herodotus.cloud/accumulators/proofs";

/// Error from [`Indexer`]
#[derive(Error, Debug)]
pub enum IndexerError {
    /// The block range provided is invalid.
    #[error("Invalid block range")]
    InvalidBlockRange,

    /// Failed to send a request using [`reqwest`].
    #[error("Failed to send request")]
    ReqwestError(#[from] reqwest::Error),

    /// Failed to parse the response using [`serde_json`].
    #[error("Failed to parse response")]
    SerdeJsonError(#[from] serde_json::Error),

    /// Validation error with a detailed message.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Failed to get headers proof with a detailed message.
    #[error("Failed to get headers proof: {0}")]
    GetHeadersProofError(String),
}

#[derive(Clone)]
pub struct Indexer {
    client: Client,
    chain_id: u64,
}

#[derive(Debug)]
pub struct IndexerHeadersProofResponse {
    pub mmr_meta: MMRMetaFromNewIndexer,
    pub headers: HashMap<BlockNumber, MMRProofFromNewIndexer>,
}

impl IndexerHeadersProofResponse {
    pub fn new(mmr_data: MMRDataFromNewIndexer) -> Self {
        let mmr_meta = mmr_data.meta;
        let headers = mmr_data
            .proofs
            .into_iter()
            .map(|block| (block.block_number, block))
            .collect();
        Self { mmr_meta, headers }
    }
}

impl Indexer {
    pub fn new(chain_id: u64) -> Self {
        Self {
            client: Client::new(),
            chain_id,
        }
    }

    /// Fetch MMR and headers proof from Herodotus Indexer
    ///
    /// ## Parameters
    /// - `from_block`: The start block number (inclusive)
    /// - `to_block`: The end block number (inclusive)
    /// - `chain_id`: The chain id
    pub async fn get_headers_proof(
        &self,
        from_block: u64,
        to_block: u64,
    ) -> Result<IndexerHeadersProofResponse, IndexerError> {
        // validate from_block and to_block
        if from_block > to_block {
            return Err(IndexerError::InvalidBlockRange);
        }

        let response = self
            .client
            .get(HERODOTUS_RS_INDEXER_URL)
            .query(&self._query(from_block, to_block, self.chain_id))
            .send()
            .await
            .map_err(IndexerError::ReqwestError)?;

        // validate status
        if response.status().is_success() {
            let body: Value = response.json().await.map_err(IndexerError::ReqwestError)?;
            let parsed_mmr: MMRFromNewIndexer =
                from_value(body).map_err(IndexerError::SerdeJsonError)?;

            // validate MMR should be 1
            if parsed_mmr.data.is_empty() {
                Err(IndexerError::ValidationError("No MMR found".to_string()))
            } else if parsed_mmr.data.len() > 1 {
                return Err(IndexerError::ValidationError(
                    "MMR length should be 1".to_string(),
                ));
            } else {
                let mmr_data = parsed_mmr.data[0].clone();
                Ok(IndexerHeadersProofResponse::new(mmr_data))
            }
        } else {
            error!("Failed to get headers proof: {}", response.status());
            Err(IndexerError::GetHeadersProofError(
                response.text().await.map_err(IndexerError::ReqwestError)?,
            ))
        }
    }

    fn _query(&self, from_block: u64, to_block: u64, chain_id: u64) -> Vec<(String, String)> {
        vec![
            ("deployed_on_chain".to_string(), chain_id.to_string()),
            ("accumulates_chain".to_string(), chain_id.to_string()),
            ("hashing_function".to_string(), "poseidon".to_string()),
            ("contract_type".to_string(), "AGGREGATOR".to_string()),
            (
                "from_block_number_inclusive".to_string(),
                from_block.to_string(),
            ),
            (
                "to_block_number_inclusive".to_string(),
                to_block.to_string(),
            ),
            ("is_meta_included".to_string(), "true".to_string()),
            ("is_whole_tree".to_string(), "true".to_string()),
            ("is_rlp_included".to_string(), "true".to_string()),
            ("is_pure_rlp".to_string(), "true".to_string()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_headers_proof() -> Result<(), IndexerError> {
        let indexer = Indexer::new(11155111);
        let response = indexer.get_headers_proof(1, 1).await?;
        // check header length is 1
        assert!(response.headers.len() == 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_headers_proof_multiple_blocks() -> Result<(), IndexerError> {
        let indexer = Indexer::new(11155111);
        let response = indexer.get_headers_proof(0, 10).await?;
        // check header length is 11
        assert!(response.headers.len() == 11);
        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_query() -> Result<(), IndexerError> {
        let indexer = Indexer::new(11155111);
        let response = indexer.get_headers_proof(10, 1).await;
        assert!(matches!(response, Err(IndexerError::InvalidBlockRange)));
        Ok(())
    }
}
