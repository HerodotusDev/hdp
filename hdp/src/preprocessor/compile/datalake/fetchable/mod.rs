use std::collections::HashSet;

use crate::primitives::processed_types::{
    account::ProcessedAccount, header::ProcessedHeader, mmr::MMRMeta, receipt::ProcessedReceipt,
    storage::ProcessedStorage, transaction::ProcessedTransaction,
};
use crate::provider::evm::provider::EvmProvider;
use alloy::primitives::U256;
use thiserror::Error;

pub mod block_sampled;
pub mod transactions;

/// Fetchable trait for fetching target datalake related data and proofs from the provider
pub trait Fetchable {
    fn fetch(
        &self,
        provider: EvmProvider,
    ) -> impl std::future::Future<Output = Result<FetchedDatalake, FetchError>> + Send;
}

#[derive(Error, Debug)]
pub enum FetchError {
    #[error("provider error: {0}")]
    ProviderError(#[from] crate::provider::evm::provider::ProviderError),
}

pub struct FetchedDatalake {
    /// Targeted datalake's compiled results
    pub values: Vec<U256>,
    /// Headers related to the datalake
    pub headers: HashSet<ProcessedHeader>,
    /// Accounts related to the datalake
    pub accounts: HashSet<ProcessedAccount>,
    /// Storages related to the datalake
    pub storages: HashSet<ProcessedStorage>,
    /// Transactions related to the datalake
    pub transactions: HashSet<ProcessedTransaction>,
    /// Transaction receipts related to the datalake
    pub transaction_receipts: HashSet<ProcessedReceipt>,
    /// MMR meta data related to the headers
    pub mmr_metas: HashSet<MMRMeta>,
}
