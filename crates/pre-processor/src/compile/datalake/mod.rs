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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use alloy::primitives::{address, B256, U256};
    use hdp_primitives::{
        aggregate_fn::AggregationFunction,
        task::datalake::{
            block_sampled::{
                AccountField, BlockSampledCollection, BlockSampledDatalake, HeaderField,
            },
            compute::Computation,
            transactions::{
                IncludedTypes, TransactionField, TransactionReceiptField, TransactionsCollection,
                TransactionsInBlockDatalake,
            },
        },
    };
    use hdp_provider::evm::provider::EvmProviderConfig;
    use starknet::providers::Url;

    use crate::compile::module::ModuleCompilerConfig;

    use super::*;

    const SEPOLIA_RPC_URL: &str =
        "https://eth-sepolia.g.alchemy.com/v2/xar76cftwEtqTBWdF4ZFy9n8FLHAETDv";
    const SN_SEPOLIA_RPC_URL: &str =
        "https://starknet-sepolia.g.alchemy.com/v2/lINonYKIlp4NH9ZI6wvqJ4HeZj7T4Wm6";

    #[tokio::test]
    async fn test_compile_block_sampled_datalake_compute_vec() {
        let program_path = "../../build/compiled_cairo/contract_dry_run.json";

        let datalake_compute_vec = vec![
            DatalakeCompute {
                compute: Computation::new(AggregationFunction::MIN, None),
                datalake: DatalakeEnvelope::BlockSampled(BlockSampledDatalake {
                    block_range_start: 10001,
                    block_range_end: 10005,
                    increment: 1,
                    sampled_property: BlockSampledCollection::Header(HeaderField::Number),
                }),
            },
            DatalakeCompute {
                compute: Computation::new(AggregationFunction::AVG, None),
                datalake: DatalakeEnvelope::BlockSampled(BlockSampledDatalake {
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

        let module_config = ModuleCompilerConfig {
            module_registry_rpc_url: Url::parse(SN_SEPOLIA_RPC_URL).unwrap(),
            program_path: PathBuf::from(program_path),
        };
        let provider_config = EvmProviderConfig {
            rpc_url: Url::parse(SEPOLIA_RPC_URL).unwrap(),
            chain_id: 11155111,
            max_requests: 100,
        };
        let compile_config = CompileConfig {
            provider: provider_config,
            module: module_config,
        };
        let results = datalake_compute_vec.compile(&compile_config).await.unwrap();
        assert_eq!(results.headers.len(), 16);
        assert_eq!(results.accounts.len(), 2);
        assert_eq!(results.storages.len(), 1);
        let storage_proofs = results.storages.iter().next().unwrap();
        assert_eq!(storage_proofs.proofs.len(), 6);
        assert_eq!(results.transactions.len(), 0);
        assert_eq!(results.transaction_receipts.len(), 0);
        assert_eq!(results.mmr_meta.id, 27);
    }

    #[tokio::test]
    async fn test_compile_transactions_datalake_compute_vec() {
        let program_path = "../../build/compiled_cairo/contract_dry_run.json";

        let datalake_compute_vec = vec![
            DatalakeCompute {
                compute: Computation::new(AggregationFunction::MIN, None),
                datalake: DatalakeEnvelope::Transactions(TransactionsInBlockDatalake {
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
                datalake: DatalakeEnvelope::Transactions(TransactionsInBlockDatalake {
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

        let module_config = ModuleCompilerConfig {
            module_registry_rpc_url: Url::parse(SN_SEPOLIA_RPC_URL).unwrap(),
            program_path: PathBuf::from(program_path),
        };
        let provider_config = EvmProviderConfig {
            rpc_url: Url::parse(SEPOLIA_RPC_URL).unwrap(),
            chain_id: 11155111,
            max_requests: 100,
        };
        let compile_config = CompileConfig {
            provider: provider_config,
            module: module_config,
        };
        let results = datalake_compute_vec.compile(&compile_config).await.unwrap();
        assert_eq!(results.headers.len(), 2);
        assert_eq!(results.accounts.len(), 0);
        assert_eq!(results.storages.len(), 0);
        assert_eq!(results.transactions.len(), 10);
        assert_eq!(results.transaction_receipts.len(), 11);
        assert_eq!(results.mmr_meta.id, 27);
    }
}
