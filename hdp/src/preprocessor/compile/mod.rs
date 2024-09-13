use alloy::primitives::U256;
use config::CompilerConfig;
use std::hash::Hash;

use std::collections::{HashMap, HashSet};
use thiserror::Error;

use crate::primitives::processed_types::block_proofs::{
    convert_to_mmr_with_headers, mmr_with_header_vec_to_map, MMRWithHeader, ProcessedBlockProofs,
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
    /// results of tasks
    pub task_results: Vec<U256>,
    /// mmr_with_headers related to the datalake
    pub mmr_with_headers: HashMap<u128, HashSet<MMRWithHeader>>,
    /// Accounts related to the datalake
    pub accounts: HashMap<u128, HashSet<ProcessedAccount>>,
    /// Storages related to the datalake
    pub storages: HashMap<u128, HashSet<ProcessedStorage>>,
    /// Transactions related to the datalake
    pub transactions: HashMap<u128, HashSet<ProcessedTransaction>>,
    /// Transaction receipts related to the datalake
    pub transaction_receipts: HashMap<u128, HashSet<ProcessedReceipt>>,
}

impl CompilationResult {
    pub fn new(
        task_results: Vec<U256>,
        mmr_with_headers: HashMap<u128, HashSet<MMRWithHeader>>,
        accounts: HashMap<u128, HashSet<ProcessedAccount>>,
        storages: HashMap<u128, HashSet<ProcessedStorage>>,
        transactions: HashMap<u128, HashSet<ProcessedTransaction>>,
        transaction_receipts: HashMap<u128, HashSet<ProcessedReceipt>>,
    ) -> Self {
        Self {
            task_results,
            mmr_with_headers,
            accounts,
            storages,
            transactions,
            transaction_receipts,
        }
    }

    pub fn from_single_chain(
        chain_id: u128,
        task_results: Vec<U256>,

        mmr_with_headers: HashSet<MMRWithHeader>,
        accounts: HashSet<ProcessedAccount>,
        storages: HashSet<ProcessedStorage>,
        transactions: HashSet<ProcessedTransaction>,
        transaction_receipts: HashSet<ProcessedReceipt>,
    ) -> Self {
        Self {
            task_results,
            mmr_with_headers: HashMap::from_iter(vec![(chain_id, mmr_with_headers)]),
            accounts: HashMap::from_iter(vec![(chain_id, accounts)]),
            storages: HashMap::from_iter(vec![(chain_id, storages)]),
            transactions: HashMap::from_iter(vec![(chain_id, transactions)]),
            transaction_receipts: HashMap::from_iter(vec![(chain_id, transaction_receipts)]),
        }
    }

    pub fn extend(&mut self, other: CompilationResult) {
        self.task_results.extend(other.task_results);

        // Merge mmr_with_headers
        merge_header_mmr_maps(&mut self.mmr_with_headers, other.mmr_with_headers);

        // Merge accounts
        merge_hash_maps(&mut self.accounts, other.accounts);

        // Merge storages
        merge_hash_maps(&mut self.storages, other.storages);

        // Merge transactions
        merge_hash_maps(&mut self.transactions, other.transactions);

        // Merge transaction_receipts
        merge_hash_maps(&mut self.transaction_receipts, other.transaction_receipts);
    }

    pub fn to_processed_block_vec(self) -> Vec<ProcessedBlockProofs> {
        let mut processed_block_vec = Vec::new();

        for (chain_id, mmr_with_headers) in self.mmr_with_headers {
            let accounts = self.accounts.get(&chain_id).cloned().unwrap_or_default();
            let storages = self.storages.get(&chain_id).cloned().unwrap_or_default();
            let transactions = self
                .transactions
                .get(&chain_id)
                .cloned()
                .unwrap_or_default();
            let transaction_receipts = self
                .transaction_receipts
                .get(&chain_id)
                .cloned()
                .unwrap_or_default();

            let processed_block = ProcessedBlockProofs {
                chain_id,
                mmr_with_headers: mmr_with_headers.into_iter().collect(),
                accounts: accounts.into_iter().collect(),
                storages: storages.into_iter().collect(),
                transactions: transactions.into_iter().collect(),
                transaction_receipts: transaction_receipts.into_iter().collect(),
            };

            processed_block_vec.push(processed_block);
        }

        processed_block_vec
    }
}

// Helper function to merge HashMaps with HashSet values
fn merge_hash_maps<T>(base: &mut HashMap<u128, HashSet<T>>, other: HashMap<u128, HashSet<T>>)
where
    T: Eq + Hash + Clone,
{
    for (key, value) in other {
        base.entry(key).or_default().extend(value);
    }
}

// TODO too complicated. refactor with method in MMRWithHeader
fn merge_header_mmr_maps(
    base: &mut HashMap<u128, HashSet<MMRWithHeader>>,
    other: HashMap<u128, HashSet<MMRWithHeader>>,
) {
    for (key, other_headers) in other {
        base.entry(key)
            .and_modify(|base_headers| {
                // Merge using the extend method from MMRWithHeader
                let mut new_headers =
                    mmr_with_header_vec_to_map(base_headers.iter().cloned().collect::<Vec<_>>());
                for item in other_headers.clone() {
                    new_headers
                        .entry(item.mmr_meta)
                        .and_modify(|existing_headers| {
                            existing_headers.extend(item.headers.iter().cloned());
                        })
                        .or_insert_with(|| item.headers.into_iter().collect());
                }
                let new_headers_vec = convert_to_mmr_with_headers(new_headers);
                *base_headers = HashSet::from_iter(new_headers_vec);
            })
            .or_insert(other_headers);
    }
}
