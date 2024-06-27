use alloy::primitives::{B256, U256};
use anyhow::{bail, Result};
use hdp_primitives::{
    datalake::{compute::DatalakeCompute, envelope::DatalakeEnvelope},
    processed_types::{
        account::ProcessedAccount, header::ProcessedHeader, mmr::MMRMeta,
        receipt::ProcessedReceipt, storage::ProcessedStorage, transaction::ProcessedTransaction,
    },
    solidity_types::traits::DatalakeComputeCodecs,
};
use hdp_provider::evm::provider::{EvmProvider, EvmProviderConfig};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::RwLock;

use self::{block_sampled::compile_block_sampled_datalake, transactions::compile_tx_datalake};

pub mod block_sampled;
pub mod transactions;

pub struct DatalakeComputeCompilationResults {
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

impl DatalakeComputeCompilationResults {
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

pub struct DatalakeCompiler {
    provider: Arc<RwLock<EvmProvider>>,
}

impl DatalakeCompiler {
    /// initialize DatalakeCompiler with commitment and datalake
    pub fn new_from_config(config: EvmProviderConfig) -> Self {
        let provider = EvmProvider::new(config);
        Self {
            provider: Arc::new(provider.into()),
        }
    }

    /// Compile the datalake meaning, fetching relevant headers, accounts, storages, and mmr_meta data.
    ///
    /// Plus, it will combine target datalake's datapoints in compiled_results.
    pub async fn compile(
        &self,
        datalake_computes: &[DatalakeCompute],
    ) -> Result<DatalakeComputeCompilationResults> {
        let mut commit_results_maps = HashMap::new();
        let mut headers: HashSet<ProcessedHeader> = HashSet::new();
        let mut accounts: HashSet<ProcessedAccount> = HashSet::new();
        let mut storages: HashSet<ProcessedStorage> = HashSet::new();
        let mut transactions: HashSet<ProcessedTransaction> = HashSet::new();
        let mut transaction_receipts: HashSet<ProcessedReceipt> = HashSet::new();
        let mut mmr = None;
        let mut pre_processable = true;

        for datalake_compute in datalake_computes {
            let task_commitment = datalake_compute.commit();
            let aggregation_fn = &datalake_compute.compute.aggregate_fn_id;
            let fn_context = datalake_compute.compute.aggregate_fn_ctx.clone();
            match datalake_compute.datalake {
                DatalakeEnvelope::BlockSampled(ref datalake) => {
                    let compiled_block_sampled =
                        compile_block_sampled_datalake(datalake.clone(), &self.provider).await?;
                    headers.extend(compiled_block_sampled.headers);
                    accounts.extend(compiled_block_sampled.accounts);
                    storages.extend(compiled_block_sampled.storages);
                    if mmr.is_some() && mmr.unwrap() != compiled_block_sampled.mmr_meta {
                        bail!("MMR meta data is not consistent");
                    } else {
                        mmr = Some(compiled_block_sampled.mmr_meta);
                    }

                    // Compute datalake over specified aggregation function to validate
                    let aggregated_result = aggregation_fn
                        .operation(&compiled_block_sampled.values, Some(fn_context))?;
                    // Save the datalake results
                    commit_results_maps.insert(task_commitment, aggregated_result);
                    if !aggregation_fn.is_pre_processable() {
                        pre_processable = false;
                    }
                }
                DatalakeEnvelope::Transactions(ref datalake) => {
                    let compiled_tx_datalake =
                        compile_tx_datalake(datalake.clone(), &self.provider).await?;
                    headers.extend(compiled_tx_datalake.headers);
                    transactions.extend(compiled_tx_datalake.transactions);
                    transaction_receipts.extend(compiled_tx_datalake.transaction_receipts);

                    if mmr.is_some() && mmr.unwrap() != compiled_tx_datalake.mmr_meta {
                        bail!("MMR meta data is not consistent");
                    } else {
                        mmr = Some(compiled_tx_datalake.mmr_meta);
                    }

                    // Compute datalake over specified aggregation function to validate
                    let aggregated_result =
                        aggregation_fn.operation(&compiled_tx_datalake.values, Some(fn_context))?;
                    // Save the datalake results
                    commit_results_maps.insert(task_commitment, aggregated_result);
                    if !aggregation_fn.is_pre_processable() {
                        pre_processable = false;
                    }
                }
            };
        }

        Ok(DatalakeComputeCompilationResults::new(
            pre_processable,
            commit_results_maps,
            headers,
            accounts,
            storages,
            transactions,
            transaction_receipts,
            mmr.unwrap(),
        ))
    }
}
