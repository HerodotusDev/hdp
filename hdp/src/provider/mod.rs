use alloy::primitives::BlockNumber;
use alloy::rpc::types::EIP1186AccountProofResponse;
use envelope::error::ProviderError;
use envelope::evm::datalake::FetchedDatalake;
use envelope::evm::from_keys::CategorizedFetchKeys;
use std::collections::HashMap;
use std::pin::Pin;
use std::{collections::HashSet, future::Future};

pub mod config;
pub mod envelope;
pub mod indexer;
pub mod key;
pub mod types;

use crate::primitives::processed_types::block_proofs::ProcessedBlockProofs;
use crate::primitives::{block::header::MMRProofFromNewIndexer, processed_types::mmr::MMRMeta};

use self::types::{FetchedTransactionProof, FetchedTransactionReceiptProof};

type HeaderProofsResult = Result<
    (
        HashSet<MMRMeta>,
        HashMap<BlockNumber, MMRProofFromNewIndexer>,
    ),
    ProviderError,
>;
type AccountProofsResult = Result<HashMap<BlockNumber, EIP1186AccountProofResponse>, ProviderError>;
type StorageProofsResult = Result<HashMap<BlockNumber, EIP1186AccountProofResponse>, ProviderError>;
type TxProofsResult = Result<Vec<FetchedTransactionProof>, ProviderError>;
type TxReceiptProofsResult = Result<Vec<FetchedTransactionReceiptProof>, ProviderError>;
type FetchProofsResult = Result<FetchedDatalake, ProviderError>;
type FetchProofsFromKeysResult = Result<ProcessedBlockProofs, ProviderError>;

type AsyncResult<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait ProofProvider: Send + Sync {
    fn fetch_proofs<'a>(
        &'a self,
        datalake: &'a crate::primitives::task::datalake::DatalakeCompute,
    ) -> AsyncResult<crate::provider::FetchProofsResult>;

    fn fetch_proofs_from_keys(
        &self,
        keys: CategorizedFetchKeys,
    ) -> AsyncResult<FetchProofsFromKeysResult>;
}
