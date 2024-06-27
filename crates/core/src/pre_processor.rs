//!  THIS IS WIP, NOT READY FOR USE
//!  Preprocessor is reponsible for identifying the required values.
//!  This will be most abstract layer of the preprocessor.

use crate::compiler::datalake_compute::DatalakeComputeCompilationResults;
use crate::compiler::module::ModuleCompilerConfig;
use crate::compiler::Compiler;
use alloy::dyn_abi::DynSolValue;
use alloy::hex;
use alloy::primitives::{Bytes, Keccak256, B256, U256};
use alloy_merkle_tree::standard_binary_tree::StandardMerkleTree;
use anyhow::{bail, Ok, Result};
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use hdp_primitives::datalake::DatalakeCompute;
use hdp_primitives::module::Module;
use hdp_primitives::processed_types::datalake_compute::ProcessedDatalakeCompute;
use hdp_primitives::processed_types::v1_query::ProcessedResult;
use hdp_primitives::solidity_types::datalake_compute::BatchedDatalakeCompute;
use hdp_primitives::solidity_types::traits::{
    BatchedDatalakeComputeCodecs, DatalakeCodecs, DatalakeComputeCodecs,
};

use hdp_provider::evm::provider::EvmProviderConfig;
use hdp_provider::key::FetchKeyEnvelope;

use tracing::info;

pub struct PreProcessor {
    /// compiler
    compiler: Compiler,
}

pub struct PreProcessorConfig {
    pub datalake_config: EvmProviderConfig,
    pub module_config: ModuleCompilerConfig,
}

impl PreProcessorConfig {
    pub fn new(datalake_config: EvmProviderConfig, module_config: ModuleCompilerConfig) -> Self {
        Self {
            datalake_config,
            module_config,
        }
    }
}

pub struct ExtendedDatalake {
    pub task: DatalakeCompute,
    pub fetch_keys_set: Vec<FetchKeyEnvelope>,
}

#[derive(Clone, Debug)]
pub struct ExtendedModule {
    pub task: Module,
    pub module_class: CasmContractClass,
}

impl PreProcessor {
    pub fn new_with_config(config: PreProcessorConfig) -> Self {
        let compiler = Compiler::new(config);
        Self { compiler }
    }

    pub async fn process_from_serialized(
        &self,
        batched_datalakes: String,
        batched_tasks: String,
    ) -> Result<ProcessedResult> {
        let bytes_datalake = hex::decode(batched_datalakes)?;
        let bytes_tasks = hex::decode(batched_tasks)?;
        // 1. decode the tasks
        let tasks = BatchedDatalakeCompute::decode(&bytes_datalake, &bytes_tasks)?;
        self.process(tasks).await
    }

    /// User request is pass as input of this function,
    /// First it will generate input structure for preprocessor that need to pass to runner
    /// Then it will run the preprocessor and return the result, fetch points
    /// Fetch points are the values that are required to run the module
    pub async fn process(&self, tasks: Vec<DatalakeCompute>) -> Result<ProcessedResult> {
        let task_commitments: Vec<B256> =
            tasks.iter().map(|task| task.commit()).collect::<Vec<_>>();
        // do compile with the tasks
        let compiled_results = self.compiler.compile(&tasks).await?;
        // do operation if possible
        let (tasks_merkle_tree, results_merkle_tree) =
            self.build_merkle_tree(&compiled_results, task_commitments)?;

        // 2. get roots of merkle tree
        let task_merkle_root = tasks_merkle_tree.root();
        let mut combined_tasks = Vec::new();

        for task in tasks {
            let task_commitment = task.commit();
            let result = if results_merkle_tree.is_some() {
                let compiled_result = compiled_results
                    .commit_results_maps
                    .get(&task_commitment)
                    .unwrap();
                let result_commitment =
                    self._raw_result_to_result_commitment(&task_commitment, *compiled_result);
                let result_proof = results_merkle_tree
                    .as_ref()
                    .unwrap()
                    .get_proof(&DynSolValue::FixedBytes(result_commitment, 32));
                Some((compiled_result, result_commitment, result_proof))
            } else {
                None
            };
            let task_proof =
                tasks_merkle_tree.get_proof(&DynSolValue::FixedBytes(task_commitment, 32));
            let encoded_task = task.encode()?;
            let datalake_type = task.datalake.get_datalake_type();
            let property_type = task.datalake.get_collection_type().to_index();

            let datalake_compute = match result {
                Some(result_value) => {
                    let (compiled_result, result_commitment, result_proof) = result_value;
                    ProcessedDatalakeCompute::new_with_result(
                        Bytes::from(encoded_task),
                        task_commitment,
                        *compiled_result,
                        result_commitment,
                        task_proof,
                        result_proof,
                        Bytes::from(task.datalake.encode()?),
                        datalake_type.into(),
                        property_type,
                    )
                }
                None => ProcessedDatalakeCompute::new_without_result(
                    Bytes::from(encoded_task),
                    task_commitment,
                    task_proof,
                    Bytes::from(task.datalake.encode()?),
                    datalake_type.into(),
                    property_type,
                ),
            };

            combined_tasks.push(datalake_compute);
        }

        let processed_result = ProcessedResult::new(
            results_merkle_tree.map(|tree| tree.root().to_string()),
            task_merkle_root.to_string(),
            Vec::from_iter(compiled_results.headers),
            compiled_results.mmr_meta,
            Vec::from_iter(compiled_results.accounts),
            Vec::from_iter(compiled_results.storages),
            Vec::from_iter(compiled_results.transactions),
            Vec::from_iter(compiled_results.transaction_receipts),
            combined_tasks,
        );
        // TODO: from compiler result, generate batch for tree and final result that pass through cairo-runner
        info!("Preprocessor completed successfully");
        Ok(processed_result)
    }

    fn build_merkle_tree(
        &self,
        compiled_results: &DatalakeComputeCompilationResults,
        task_commitments: Vec<B256>,
    ) -> Result<(StandardMerkleTree, Option<StandardMerkleTree>)> {
        let mut tasks_leaves = Vec::new();
        let mut results_leaves = Vec::new();

        for task_commitment in task_commitments {
            if compiled_results.pre_processable {
                let compiled_result =
                    match compiled_results.commit_results_maps.get(&task_commitment) {
                        Some(result) => result,
                        None => bail!("Task commitment not found in compiled results"),
                    };
                let result_commitment =
                    self._raw_result_to_result_commitment(&task_commitment, *compiled_result);
                results_leaves.push(DynSolValue::FixedBytes(result_commitment, 32));
            }
            tasks_leaves.push(DynSolValue::FixedBytes(task_commitment, 32));
        }
        let tasks_merkle_tree = StandardMerkleTree::of(tasks_leaves);

        if compiled_results.pre_processable {
            let results_merkle_tree = StandardMerkleTree::of(results_leaves);
            Ok((tasks_merkle_tree, Some(results_merkle_tree)))
        } else {
            Ok((tasks_merkle_tree, None))
        }
    }

    fn _raw_result_to_result_commitment(
        &self,
        task_commitment: &B256,
        compiled_result: U256,
    ) -> B256 {
        let mut hasher = Keccak256::new();
        hasher.update(task_commitment);
        hasher.update(B256::from(compiled_result));
        hasher.finalize()
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use hdp_primitives::datalake::block_sampled::{
    //     BlockSampledCollection, BlockSampledDatalake, HeaderField,
    // };
    // use hdp_primitives::datalake::envelope::DatalakeEnvelope;
    // use hdp_primitives::datalake::task::Computation;
    // use hdp_primitives::module::{Module, ModuleTag};
    // use starknet::macros::felt;
    // use starknet::providers::Url;
    // use std::path::PathBuf;

    // const STARKNET_SEPOLIA_RPC: &str =
    //     "https://starknet-sepolia.g.alchemy.com/v2/lINonYKIlp4NH9ZI6wvqJ4HeZj7T4Wm6";
    // const PREPROCESS_PROGRAM_PATH: &str = "../build/compiled_cairo/hdp.json";

    // #[tokio::test]
    // async fn test_process_only_datalake() {
    //     let start_process = std::time::Instant::now();
    //     let config = PreProcessorConfig {
    //         module_registry_rpc_url: Url::parse(STARKNET_SEPOLIA_RPC).unwrap(),
    //         program_path: PathBuf::from("../build/compiled_cairo/hdp.json"),
    //     };
    //     let pre_processor = PreProcessor::new_with_config(config);

    //     let tasks = vec![
    //         TaskEnvelope::DatalakeCompute(DatalakeCompute {
    //             compute: Computation::new("min", None),
    //             datalake: DatalakeEnvelope::BlockSampled(BlockSampledDatalake {
    //                 block_range_start: 1000,
    //                 block_range_end: 10000,
    //                 increment: 1,
    //                 sampled_property: BlockSampledCollection::Header(HeaderField::Number),
    //             }),
    //         }),
    //         TaskEnvelope::DatalakeCompute(DatalakeCompute {
    //             compute: Computation::new("min", None),
    //             datalake: DatalakeEnvelope::BlockSampled(BlockSampledDatalake {
    //                 block_range_start: 1000,
    //                 block_range_end: 10000,
    //                 increment: 1,
    //                 sampled_property: BlockSampledCollection::Header(HeaderField::Number),
    //             }),
    //         }),
    //     ];

    //     let result = pre_processor.process(tasks).await.unwrap();

    //     let end_process = start_process.elapsed();
    //     println!("Process time: {:?}", end_process);
    //     assert_eq!(result.fetch_keys.len(), 9000);
    //     assert_eq!(result.tasks.len(), 2);
    //     assert!(matches!(&result.tasks[0], ExtendedTask::DatalakeCompute(_)));
    // }

    // #[tokio::test]
    // async fn test_process_only_module() {
    //     let start_process = std::time::Instant::now();
    //     let config = PreProcessorConfig {
    //         module_registry_rpc_url: Url::parse(STARKNET_SEPOLIA_RPC).unwrap(),
    //         program_path: PathBuf::from(PREPROCESS_PROGRAM_PATH),
    //     };
    //     let pre_processor = PreProcessor::new_with_config(config);

    //     let module = Module::from_tag(ModuleTag::TEST, vec![felt!("1"), felt!("2")]);
    //     let tasks = vec![TaskEnvelope::Module(module)];

    //     let result = pre_processor.process(tasks).await.unwrap();
    //     let end_process = start_process.elapsed();
    //     println!("Process time: {:?}", end_process);
    //     assert_eq!(result.fetch_keys.len(), 0);
    //     assert_eq!(result.tasks.len(), 1);
    //     assert!(matches!(&result.tasks[0], ExtendedTask::Module(_)));
    // }
}
