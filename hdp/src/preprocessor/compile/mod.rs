use alloy::primitives::U256;

use config::CompilerConfig;

use std::collections::HashSet;
use thiserror::Error;

use crate::primitives::processed_types::block_proofs::{
    convert_to_mmr_meta_set, convert_to_mmr_with_headers, MMRWithHeader,
};
use crate::primitives::processed_types::{
    account::ProcessedAccount, receipt::ProcessedReceipt, storage::ProcessedStorage,
    transaction::ProcessedTransaction,
};

use crate::provider::error::ProviderError;
use crate::{cairo_runner, preprocessor::module_registry::ModuleRegistryError};

pub mod config;
pub mod datalake;
pub mod module;
pub mod task;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Class hash mismatch")]
    ClassHashMismatch,

    #[error("Cairo Runner Error: {0}")]
    CairoRunnerError(#[from] cairo_runner::CairoRunnerError),

    #[error("Error from provider: {0}")]
    ProviderError(#[from] ProviderError),

    #[error("Invalid MMR meta data")]
    InvalidMMR,

    #[error("General error: {0}")]
    GeneralError(#[from] anyhow::Error),

    #[error("Error from module registry: {0}")]
    ModuleRegistryError(#[from] ModuleRegistryError),

    #[error("Compilation failed")]
    CompilationFailed,
}

/// Compile vector of tasks into compilation results
pub trait Compilable {
    fn compile(
        &self,
        compile_config: &CompilerConfig,
    ) -> impl std::future::Future<Output = Result<CompilationResult, CompileError>> + Send;
}

#[derive(Debug, Default, PartialEq)]
pub struct CompilationResult {
    pub chain_id: u128,
    /// results of tasks
    pub task_results: Vec<U256>,
    /// mmr_with_headers related to the datalake
    pub mmr_with_headers: HashSet<MMRWithHeader>,
    /// Accounts related to the datalake
    pub accounts: HashSet<ProcessedAccount>,
    /// Storages related to the datalake
    pub storages: HashSet<ProcessedStorage>,
    /// Transactions related to the datalake
    pub transactions: HashSet<ProcessedTransaction>,
    /// Transaction receipts related to the datalake
    pub transaction_receipts: HashSet<ProcessedReceipt>,
}

impl CompilationResult {
    pub fn new(
        chain_id: u128,
        task_results: Vec<U256>,
        mmr_with_headers: HashSet<MMRWithHeader>,
        accounts: HashSet<ProcessedAccount>,
        storages: HashSet<ProcessedStorage>,
        transactions: HashSet<ProcessedTransaction>,
        transaction_receipts: HashSet<ProcessedReceipt>,
    ) -> Self {
        Self {
            chain_id,
            task_results,
            mmr_with_headers,
            accounts,
            storages,
            transactions,
            transaction_receipts,
        }
    }

    /// Extend the current compilation results with another compilation results
    pub fn extend(&mut self, other: CompilationResult) {
        let others_mmr_with_headers_set =
            convert_to_mmr_meta_set(Vec::from_iter(other.mmr_with_headers));
        let mut self_mmr_with_headers_set =
            convert_to_mmr_meta_set(Vec::from_iter(self.mmr_with_headers.clone()));
        for (mmr_meta, headers) in others_mmr_with_headers_set {
            self_mmr_with_headers_set
                .entry(mmr_meta)
                .or_default()
                .extend(headers);
        }
        self.mmr_with_headers =
            HashSet::from_iter(convert_to_mmr_with_headers(self_mmr_with_headers_set));
        self.accounts.extend(other.accounts);
        self.storages.extend(other.storages);
        self.transactions.extend(other.transactions);
        self.transaction_receipts.extend(other.transaction_receipts);
        self.task_results.extend(other.task_results);
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct DatalakeCompileResult {
    pub chain_id: u128,
    /// results of tasks
    pub task_results: Vec<U256>,
    /// mmr_with_headers related to the datalake
    pub mmr_with_headers: HashSet<MMRWithHeader>,
    /// Accounts related to the datalake
    pub accounts: HashSet<ProcessedAccount>,
    /// Storages related to the datalake
    pub storages: HashSet<ProcessedStorage>,
    /// Transactions related to the datalake
    pub transactions: HashSet<ProcessedTransaction>,
    /// Transaction receipts related to the datalake
    pub transaction_receipts: HashSet<ProcessedReceipt>,
}

impl DatalakeCompileResult {
    pub fn new(
        chain_id: u128,
        task_results: Vec<U256>,
        mmr_with_headers: HashSet<MMRWithHeader>,
        accounts: HashSet<ProcessedAccount>,
        storages: HashSet<ProcessedStorage>,
        transactions: HashSet<ProcessedTransaction>,
        transaction_receipts: HashSet<ProcessedReceipt>,
    ) -> Self {
        Self {
            chain_id,
            task_results,
            mmr_with_headers,
            accounts,
            storages,
            transactions,
            transaction_receipts,
        }
    }
}
