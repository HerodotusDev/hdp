use thiserror::Error;

use crate::provider::indexer::IndexerError;

use super::evm::rpc::RpcProviderError;

/// Error type for provider
#[derive(Error, Debug)]
pub enum ProviderError {
    /// Error when the query is invalid
    #[error("Transaction index out of bound: requested index: {0}, length: {1}")]
    OutOfBoundRequestError(u64, u64),

    /// Error when the MMR meta is mismatched among range of requested blocks
    #[error("MMR meta mismatch among range of requested blocks")]
    MismatchedMMRMeta,

    /// Error when the MMR is not found
    #[error("MMR not found")]
    MmrNotFound,

    /// Error from the [`Indexer`]
    #[error("Failed from indexer")]
    IndexerError(#[from] IndexerError),

    /// Error from [`RpcProvider`]
    #[error("Failed to get proofs: {0}")]
    EvmRpcProviderError(#[from] RpcProviderError),

    /// Error from [`eth_trie_proofs`]
    #[error("EthTrieError: {0}")]
    EthTrieError(#[from] eth_trie_proofs::EthTrieError),

    #[error("Fetch key error: {0}")]
    FetchKeyError(String),
}
