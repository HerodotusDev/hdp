use std::collections::{HashMap, HashSet};

use crate::compile::datalake::fetchable::Fetchable;
use hdp_primitives::{
    processed_types::{
        account::ProcessedAccount, header::ProcessedHeader, receipt::ProcessedReceipt,
        storage::ProcessedStorage, transaction::ProcessedTransaction,
    },
    solidity_types::traits::DatalakeComputeCodecs,
    task::datalake::{envelope::DatalakeEnvelope, DatalakeCompute},
};
use hdp_provider::evm::provider::EvmProvider;

use super::{Compilable, CompilationResults, CompileConfig, CompileError};

pub mod fetchable;

impl Compilable for Vec<DatalakeCompute> {
    async fn compile(
        &self,
        compile_config: &CompileConfig,
    ) -> Result<CompilationResults, CompileError> {
        let mut commit_results_maps = HashMap::new();
        let mut headers: HashSet<ProcessedHeader> = HashSet::new();
        let mut accounts: HashSet<ProcessedAccount> = HashSet::new();
        let mut storages: HashSet<ProcessedStorage> = HashSet::new();
        let mut transactions: HashSet<ProcessedTransaction> = HashSet::new();
        let mut transaction_receipts: HashSet<ProcessedReceipt> = HashSet::new();
        let mut mmr = None;
        let mut pre_processable = true;

        for datalake_compute in self {
            let task_commitment = datalake_compute.commit();
            let aggregation_fn = &datalake_compute.compute.aggregate_fn_id;
            let fn_context = datalake_compute.compute.aggregate_fn_ctx.clone();
            let provider = EvmProvider::new(compile_config.provider.clone());
            match datalake_compute.datalake {
                DatalakeEnvelope::BlockSampled(ref datalake) => {
                    let compiled_block_sampled = datalake.fetch(provider).await?;
                    headers.extend(compiled_block_sampled.headers);
                    accounts.extend(compiled_block_sampled.accounts);
                    storages.extend(compiled_block_sampled.storages);
                    if mmr.is_some() && mmr.unwrap() != compiled_block_sampled.mmr_meta {
                        return Err(CompileError::InvalidMMR);
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
                    let compiled_tx_datalake = datalake.fetch(provider).await?;
                    headers.extend(compiled_tx_datalake.headers);
                    transactions.extend(compiled_tx_datalake.transactions);
                    transaction_receipts.extend(compiled_tx_datalake.transaction_receipts);

                    if mmr.is_some() && mmr.unwrap() != compiled_tx_datalake.mmr_meta {
                        return Err(CompileError::InvalidMMR);
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

        Ok(CompilationResults::new(
            pre_processable,
            HashMap::new(),
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
