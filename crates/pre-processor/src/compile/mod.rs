use std::collections::{HashMap, HashSet};

use alloy::primitives::{B256, U256};
use datalake::fetchable::FetchError;
use hdp_primitives::processed_types::{
    account::ProcessedAccount, header::ProcessedHeader, mmr::MMRMeta, receipt::ProcessedReceipt,
    storage::ProcessedStorage, transaction::ProcessedTransaction,
};
use hdp_provider::evm::provider::EvmProviderConfig;
use thiserror::Error;

pub mod datalake;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Invalid provider")]
    ProviderError(#[from] hdp_provider::evm::provider::ProviderError),
    #[error("Failed to fetch datalake: {0}")]
    FetchError(#[from] FetchError),
    #[error("Invalid MMR meta data")]
    InvalidMMR,
    #[error("Failed from anyhow")]
    AnyhowError(#[from] anyhow::Error),
}

pub trait Compilable {
    fn compile(
        &self,
        provider_config: &EvmProviderConfig,
    ) -> impl std::future::Future<Output = Result<CompilationResults, CompileError>> + Send;
}

pub struct CompilationResults {
    /// flag to check if the aggregation function is pre-processable
    pub pre_processable: bool,
    /// task_commitment -> value
    pub commit_results_maps: HashMap<B256, U256>,
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
    pub mmr_meta: MMRMeta,
}

impl CompilationResults {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pre_processable: bool,
        commit_results_maps: HashMap<B256, U256>,
        headers: HashSet<ProcessedHeader>,
        accounts: HashSet<ProcessedAccount>,
        storages: HashSet<ProcessedStorage>,
        transactions: HashSet<ProcessedTransaction>,
        transaction_receipts: HashSet<ProcessedReceipt>,
        mmr_meta: MMRMeta,
    ) -> Self {
        Self {
            pre_processable,
            commit_results_maps,
            headers,
            accounts,
            storages,
            transactions,
            transaction_receipts,
            mmr_meta,
        }
    }
}
