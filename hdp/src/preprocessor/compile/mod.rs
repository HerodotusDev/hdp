use alloy::primitives::U256;

use config::CompilerConfig;

use std::collections::HashSet;
use thiserror::Error;

use crate::primitives::processed_types::{
    account::ProcessedAccount, header::ProcessedHeader, mmr::MMRMeta, receipt::ProcessedReceipt,
    storage::ProcessedStorage, transaction::ProcessedTransaction,
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
    /// results of tasks
    pub task_results: Vec<U256>,
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

impl CompilationResult {
    pub fn new(
        task_results: Vec<U256>,
        headers: HashSet<ProcessedHeader>,
        accounts: HashSet<ProcessedAccount>,
        storages: HashSet<ProcessedStorage>,
        transactions: HashSet<ProcessedTransaction>,
        transaction_receipts: HashSet<ProcessedReceipt>,
        mmr_metas: HashSet<MMRMeta>,
    ) -> Self {
        Self {
            task_results,
            headers,
            accounts,
            storages,
            transactions,
            transaction_receipts,
            mmr_metas,
        }
    }

    /// Extend the current compilation results with another compilation results
    pub fn extend(&mut self, other: CompilationResult) {
        self.headers.extend(other.headers);
        self.accounts.extend(other.accounts);
        self.storages.extend(other.storages);
        self.transactions.extend(other.transactions);
        self.transaction_receipts.extend(other.transaction_receipts);
        self.task_results.extend(other.task_results);
        self.mmr_metas.extend(other.mmr_metas);
    }
}
