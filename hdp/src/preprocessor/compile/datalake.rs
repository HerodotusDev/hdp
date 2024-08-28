use crate::{primitives::task::datalake::DatalakeCompute, provider::envelope::ProviderEnvelope};
use tracing::{debug, info};

use super::{config::CompilerConfig, Compilable, CompilationResult, CompileError};

impl Compilable for DatalakeCompute {
    async fn compile(
        &self,
        compile_config: &CompilerConfig,
    ) -> Result<CompilationResult, CompileError> {
        info!("target task: {:#?}", self);
        // ========== datalake ==============
        let target_provider_config = compile_config
            .provider_config
            .get(&self.datalake.get_chain_id())
            .expect("target task's chain had not been configured.");
        let provider = ProviderEnvelope::new(target_provider_config);
        let compiled_block_sampled = provider.fetch_proofs(self).await?;
        debug!("values to aggregate : {:#?}", compiled_block_sampled.values);

        // ========== compute ==============
        let aggregation_fn = &self.compute.aggregate_fn_id;
        let fn_context = &self.compute.aggregate_fn_ctx;
        let aggregated_result =
            aggregation_fn.operation(&compiled_block_sampled.values, Some(fn_context.clone()))?;

        Ok(CompilationResult::new(
            aggregation_fn.is_pre_processable(),
            vec![aggregated_result],
            compiled_block_sampled.headers,
            compiled_block_sampled.accounts,
            compiled_block_sampled.storages,
            compiled_block_sampled.transactions,
            compiled_block_sampled.transaction_receipts,
            compiled_block_sampled.mmr_metas,
        ))
    }
}

pub type DatalakeComputeVec = Vec<DatalakeCompute>;

impl Compilable for DatalakeComputeVec {
    async fn compile(
        &self,
        compile_config: &CompilerConfig,
    ) -> Result<CompilationResult, CompileError> {
        let mut final_results = CompilationResult::default();

        for datalake_compute in self {
            let current_results = datalake_compute.compile(compile_config).await?;
            final_results.extend(current_results);
        }

        Ok(final_results)
    }
}

#[cfg(test)]
#[cfg(feature = "test_utils")]
mod tests {
    use std::path::PathBuf;

    use crate::primitives::{
        aggregate_fn::AggregationFunction,
        task::datalake::{
            block_sampled::{
                AccountField, BlockSampledCollection, BlockSampledDatalake, HeaderField,
            },
            compute::Computation,
            envelope::DatalakeEnvelope,
            transactions::{
                IncludedTypes, TransactionField, TransactionReceiptField, TransactionsCollection,
                TransactionsInBlockDatalake,
            },
        },
    };
    use alloy::primitives::{address, B256, U256};

    use super::*;

    #[tokio::test]
    async fn test_compile_block_sampled_datalake_compute_vec() {
        let program_path = "../../build/compiled_cairo/contract_dry_run.json";

        let datalake_compute_vec = vec![
            DatalakeCompute {
                compute: Computation::new(AggregationFunction::MIN, None),
                datalake: DatalakeEnvelope::BlockSampled(BlockSampledDatalake {
                    chain_id: 11155111,
                    block_range_start: 10001,
                    block_range_end: 10005,
                    increment: 1,
                    sampled_property: BlockSampledCollection::Header(HeaderField::Number),
                }),
            },
            DatalakeCompute {
                compute: Computation::new(AggregationFunction::AVG, None),
                datalake: DatalakeEnvelope::BlockSampled(BlockSampledDatalake {
                    chain_id: 11155111,
                    block_range_start: 6127485,
                    block_range_end: 6127495,
                    increment: 1,
                    sampled_property: BlockSampledCollection::Account(
                        address!("7f2c6f930306d3aa736b3a6c6a98f512f74036d4"),
                        AccountField::Balance,
                    ),
                }),
            },
            DatalakeCompute {
                compute: Computation::new(AggregationFunction::AVG, None),
                datalake: DatalakeEnvelope::BlockSampled(BlockSampledDatalake {
                    chain_id: 11155111,
                    block_range_start: 6127485,
                    block_range_end: 6127490,
                    increment: 1,
                    sampled_property: BlockSampledCollection::Storage(
                        address!("75CeC1db9dCeb703200EAa6595f66885C962B920"),
                        B256::from(U256::from(1)),
                    ),
                }),
            },
        ];

        let compiler_config =
            CompilerConfig::default().with_dry_run_program_path(PathBuf::from(program_path));

        let results = datalake_compute_vec
            .compile(&compiler_config)
            .await
            .unwrap();
        assert_eq!(results.headers.len(), 16);
        assert_eq!(results.accounts.len(), 2);
        assert_eq!(results.storages.len(), 1);
        let storage_proofs = results.storages.iter().next().unwrap();
        assert_eq!(storage_proofs.proofs.len(), 6);
        assert_eq!(results.transactions.len(), 0);
        assert_eq!(results.transaction_receipts.len(), 0);
        assert_eq!(results.mmr_metas.len(), 1);
    }

    #[tokio::test]
    async fn test_compile_transactions_datalake_compute_vec() {
        let program_path = "../../build/compiled_cairo/contract_dry_run.json";

        let datalake_compute_vec = vec![
            DatalakeCompute {
                compute: Computation::new(AggregationFunction::MIN, None),
                datalake: DatalakeEnvelope::TransactionsInBlock(TransactionsInBlockDatalake {
                    chain_id: 11155111,
                    target_block: 6127486,
                    start_index: 0,
                    end_index: 10,
                    increment: 1,
                    included_types: IncludedTypes::from(&[1, 1, 1, 1]),
                    sampled_property: TransactionsCollection::Transactions(
                        TransactionField::GasLimit,
                    ),
                }),
            },
            DatalakeCompute {
                compute: Computation::new(AggregationFunction::MIN, None),
                datalake: DatalakeEnvelope::TransactionsInBlock(TransactionsInBlockDatalake {
                    chain_id: 11155111,
                    target_block: 6127485,
                    start_index: 0,
                    end_index: 11,
                    increment: 1,
                    included_types: IncludedTypes::from(&[1, 1, 1, 1]),
                    sampled_property: TransactionsCollection::TranasactionReceipts(
                        TransactionReceiptField::Success,
                    ),
                }),
            },
        ];

        let compiler_config =
            CompilerConfig::default().with_dry_run_program_path(PathBuf::from(program_path));
        let results = datalake_compute_vec
            .compile(&compiler_config)
            .await
            .unwrap();
        assert_eq!(results.headers.len(), 2);
        assert_eq!(results.accounts.len(), 0);
        assert_eq!(results.storages.len(), 0);
        assert_eq!(results.transactions.len(), 10);
        assert_eq!(results.transaction_receipts.len(), 11);
        assert_eq!(results.mmr_metas.len(), 1);
    }
}
