use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexerError {
    #[error("Invalid block range")]
    InvalidBlockRange,
    #[error("Failed to send request")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Failed to parse response")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Failed to get headers proof: {0}")]
    GetHeadersProofError(String),
}

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Failed to get proofs: {0}")]
    GetProofsError(String),
    #[error("Failed from indexer")]
    IndexerError(#[from] IndexerError),
    #[error("Failed to get transaction proof: {0}")]
    GetTransactionProofError(String),
    #[error("Failed to get transaction receipt proof: {0}")]
    GetTransactionReceiptProofError(String),
    #[error("EthTrieError: {0}")]
    EthTrieError(#[from] eth_trie_proofs::EthTrieError),
}
