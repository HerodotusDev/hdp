use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;

use alloy::primitives::TxIndex;
use alloy::{
    primitives::{Address, BlockNumber, StorageKey},
    rpc::types::EIP1186AccountProofResponse,
};
use envelope::error::ProviderError;

pub mod config;
pub mod envelope;
pub mod indexer;
pub mod key;
pub mod types;

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

type AsyncResult<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait ProofProvider: Send + Sync {
    fn get_range_of_header_proofs(
        &self,
        from_block: BlockNumber,
        to_block: BlockNumber,
        increment: u64,
    ) -> AsyncResult<HeaderProofsResult>;

    fn get_range_of_account_proofs(
        &self,
        from_block: BlockNumber,
        to_block: BlockNumber,
        increment: u64,
        address: Address,
    ) -> AsyncResult<AccountProofsResult>;

    fn get_range_of_storage_proofs(
        &self,
        from_block: BlockNumber,
        to_block: BlockNumber,
        increment: u64,
        address: Address,
        storage_slot: StorageKey,
    ) -> AsyncResult<StorageProofsResult>;

    fn get_tx_with_proof_from_block(
        &self,
        target_block: BlockNumber,
        start_index: TxIndex,
        end_index: TxIndex,
        incremental: u64,
    ) -> AsyncResult<TxProofsResult>;

    fn get_tx_receipt_with_proof_from_block(
        &self,
        target_block: BlockNumber,
        start_index: TxIndex,
        end_index: TxIndex,
        incremental: u64,
    ) -> AsyncResult<TxReceiptProofsResult>;
}
