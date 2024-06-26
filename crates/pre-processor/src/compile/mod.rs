use alloy::primitives::{B256, U256};
use hdp_primitives::processed_types::{
    account::ProcessedAccount, header::ProcessedHeader, mmr::MMRMeta, receipt::ProcessedReceipt,
    storage::ProcessedStorage, transaction::ProcessedTransaction,
};
use hdp_provider::evm::provider::EvmProviderConfig;
use module::ModuleCompilerConfig;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

use crate::module_registry::ModuleRegistryError;

pub mod datalake;
pub mod module;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Cairo Runner Error: {0}")]
    CairoRunnerError(#[from] hdp_cairo_runner::CairoRunnerError),

    #[error("Invalid provider")]
    ProviderError(#[from] hdp_provider::evm::provider::ProviderError),

    #[error("Failed to fetch datalake: {0}")]
    FetchError(#[from] datalake::fetchable::FetchError),

    #[error("Invalid MMR meta data")]
    InvalidMMR,

    #[error("General error: {0}")]
    GeneralError(#[from] anyhow::Error),

    #[error("Error from module registry: {0}")]
    ModuleRegistryError(#[from] ModuleRegistryError),
}

pub struct CompileConfig {
    pub provider: EvmProviderConfig,
    pub module: ModuleCompilerConfig,
}

/// Compile vector of tasks into compilation results
pub trait Compilable {
    fn compile(
        &self,
        compile_config: &CompileConfig,
    ) -> impl std::future::Future<Output = Result<CompilationResults, CompileError>> + Send;
}

#[derive(Debug)]
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
    pub fn new_without_result(
        headers: HashSet<ProcessedHeader>,
        accounts: HashSet<ProcessedAccount>,
        storages: HashSet<ProcessedStorage>,
        transactions: HashSet<ProcessedTransaction>,
        transaction_receipts: HashSet<ProcessedReceipt>,
        mmr_meta: MMRMeta,
    ) -> Self {
        Self {
            pre_processable: false,
            commit_results_maps: HashMap::new(),
            headers,
            accounts,
            storages,
            transactions,
            transaction_receipts,
            mmr_meta,
        }
    }

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
