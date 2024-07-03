use std::collections::{HashMap, HashSet};

use crate::compile::datalake::fetchable::Fetchable;
use hdp_primitives::{
    solidity_types::traits::DatalakeComputeCodecs,
    task::datalake::{envelope::DatalakeEnvelope, DatalakeCompute},
};
use hdp_provider::evm::provider::EvmProvider;

use super::{Compilable, CompilationResults, CompileConfig, CompileError};

pub mod fetchable;

impl Compilable for DatalakeCompute {
    async fn compile(
        &self,
        compile_config: &CompileConfig,
    ) -> Result<CompilationResults, CompileError> {
        let task_commitment = self.commit();
        let aggregation_fn = &self.compute.aggregate_fn_id;
        let fn_context = &self.compute.aggregate_fn_ctx;
        let provider = EvmProvider::new(compile_config.provider.clone());
        match self.datalake {
            DatalakeEnvelope::BlockSampled(ref datalake) => {
                let compiled_block_sampled = datalake.fetch(provider).await?;
                let aggregated_result = aggregation_fn
                    .operation(&compiled_block_sampled.values, Some(fn_context.clone()))?;
                Ok(CompilationResults::new(
                    aggregation_fn.is_pre_processable(),
                    HashMap::new(),
                    vec![(task_commitment, aggregated_result)]
                        .into_iter()
                        .collect(),
                    compiled_block_sampled.headers,
                    compiled_block_sampled.accounts,
                    compiled_block_sampled.storages,
                    HashSet::new(),
                    HashSet::new(),
                    compiled_block_sampled.mmr_meta,
                ))
            }
            DatalakeEnvelope::Transactions(ref datalake) => {
                let compiled_tx_datalake = datalake.fetch(provider).await?;
                let aggregated_result = aggregation_fn
                    .operation(&compiled_tx_datalake.values, Some(fn_context.clone()))?;
                Ok(CompilationResults::new(
                    aggregation_fn.is_pre_processable(),
                    HashMap::new(),
                    vec![(task_commitment, aggregated_result)]
                        .into_iter()
                        .collect(),
                    compiled_tx_datalake.headers,
                    HashSet::new(),
                    HashSet::new(),
                    compiled_tx_datalake.transactions,
                    compiled_tx_datalake.transaction_receipts,
                    compiled_tx_datalake.mmr_meta,
                ))
            }
        }
    }
}

pub type DatalakeComputeVec = Vec<DatalakeCompute>;

impl Compilable for DatalakeComputeVec {
    async fn compile(
        &self,
        compile_config: &CompileConfig,
    ) -> Result<CompilationResults, CompileError> {
        let mut final_results = CompilationResults::default();

        for datalake_compute in self {
            let current_results = datalake_compute.compile(compile_config).await?;
            final_results.extend(current_results);
        }

        Ok(final_results)
    }
}
