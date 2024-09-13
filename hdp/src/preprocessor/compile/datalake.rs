use crate::{
    primitives::task::datalake::DatalakeCompute, provider::traits::new_provider_from_config,
};
use tracing::{debug, info};

use super::{config::CompilerConfig, Compilable, CompilationResult, CompileError};

pub async fn compile_datalake(
    datalake: &DatalakeCompute,
    compile_config: &CompilerConfig,
) -> Result<CompilationResult, CompileError> {
    info!("target task: {:#?}", datalake);
    // ========== datalake ==============
    let target_provider_config = compile_config
        .provider_config
        .get(&datalake.datalake.get_chain_id())
        .expect("target task's chain had not been configured.");
    let provider = new_provider_from_config(target_provider_config);
    let compiled_block_sampled = provider.fetch_proofs(datalake).await?;
    debug!("values to aggregate : {:#?}", compiled_block_sampled.values);

    // ========== compute ==============
    let aggregation_fn = &datalake.compute.aggregate_fn_id;
    let fn_context = &datalake.compute.aggregate_fn_ctx;
    let aggregated_result =
        aggregation_fn.operation(&compiled_block_sampled.values, Some(fn_context.clone()))?;

    Ok(CompilationResult::from_single_chain(
        datalake.datalake.get_chain_id().to_numeric_id(),
        vec![aggregated_result],
        compiled_block_sampled.mmr_with_headers,
        compiled_block_sampled.accounts,
        compiled_block_sampled.storages,
        compiled_block_sampled.transactions,
        compiled_block_sampled.transaction_receipts,
    ))
}

pub type DatalakeComputeVec = Vec<DatalakeCompute>;

impl Compilable for DatalakeComputeVec {
    async fn compile(
        &self,
        compile_config: &CompilerConfig,
    ) -> Result<CompilationResult, CompileError> {
        let mut final_results = CompilationResult::default();

        for datalake_compute in self {
            let current_results = compile_datalake(datalake_compute, compile_config).await?;
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
        ChainId,
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
                    chain_id: ChainId::EthereumSepolia,
                    block_range_start: 10001,
                    block_range_end: 10005,
                    increment: 1,
                    sampled_property: BlockSampledCollection::Header(HeaderField::Number),
                }),
            },
            DatalakeCompute {
                compute: Computation::new(AggregationFunction::AVG, None),
                datalake: DatalakeEnvelope::BlockSampled(BlockSampledDatalake {
                    chain_id: ChainId::EthereumSepolia,
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
                    chain_id: ChainId::EthereumSepolia,
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
        // assert_eq!(results.mmr_with_headers[0].headers.len(), 16);
        let account_proofs = results.accounts.iter().next().unwrap();
        assert_eq!(account_proofs.1.len(), 2);
        let storage_proofs = results.storages.iter().next().unwrap();
        assert_eq!(storage_proofs.1.len(), 1);
        let storage_proofs = storage_proofs.1.iter().next().unwrap();
        assert_eq!(storage_proofs.proofs.len(), 6);
        assert_eq!(results.transactions.len(), 0);
        assert_eq!(results.transaction_receipts.len(), 0);
        // assert_eq!(results.mmr_metas.len(), 1);
    }

    #[tokio::test]
    async fn test_compile_transactions_datalake_compute_vec() {
        let program_path = "../../build/compiled_cairo/contract_dry_run.json";

        let datalake_compute_vec = vec![
            DatalakeCompute {
                compute: Computation::new(AggregationFunction::MIN, None),
                datalake: DatalakeEnvelope::TransactionsInBlock(TransactionsInBlockDatalake {
                    chain_id: ChainId::EthereumSepolia,
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
                    chain_id: ChainId::EthereumSepolia,
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
        // assert_eq!(results.headers.len(), 2);
        assert_eq!(results.accounts.len(), 0);
        assert_eq!(results.storages.len(), 0);
        assert_eq!(results.transactions.len(), 10);
        assert_eq!(results.transaction_receipts.len(), 11);
        // assert_eq!(results.mmr_metas.len(), 1);
    }
}
