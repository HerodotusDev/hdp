//!  THIS IS WIP, NOT READY FOR USE
//!  Preprocessor is reponsible for identifying the required values.
//!  This will be most abstract layer of the preprocessor.

use alloy::dyn_abi::DynSolValue;
use alloy::primitives::{Bytes, Keccak256, B256, U256};
use alloy_merkle_tree::standard_binary_tree::StandardMerkleTree;
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use compile::{Compilable, CompilationResults, CompileConfig, CompileError};
use hdp_primitives::processed_types::block_proofs::ProcessedBlockProofs;
use hdp_primitives::processed_types::datalake_compute::ProcessedDatalakeCompute;
use hdp_primitives::processed_types::module::ProcessedModule;
use hdp_primitives::processed_types::query::ProcessedFullInput;
use hdp_primitives::processed_types::task::ProcessedTask;
use hdp_primitives::solidity_types::datalake_compute::BatchedDatalakeCompute;
use hdp_primitives::solidity_types::traits::{
    BatchedDatalakeComputeCodecs, DatalakeCodecs, DatalakeComputeCodecs,
};
use hdp_primitives::task::module::Module;

use hdp_primitives::task::TaskEnvelope;

use thiserror::Error;
use tracing::info;

pub mod compile;
pub mod module_registry;

#[derive(Error, Debug)]
pub enum PreProcessorError {
    #[error("Failed to compile the tasks")]
    CompileError(#[from] CompileError),
    #[error("Failed to decode the tasks")]
    DecodeError(#[from] anyhow::Error),
    #[error("Task commitment not found in compiled results")]
    TaskCommitmentNotFound,
}

pub struct PreProcessor {
    pub compile_config: CompileConfig,
}

#[derive(Clone, Debug)]
pub struct ExtendedModule {
    pub task: Module,
    pub module_class: CasmContractClass,
}

impl PreProcessor {
    pub fn new_with_config(compile_config: CompileConfig) -> Self {
        Self { compile_config }
    }

    pub async fn process_from_serialized(
        &self,
        batched_datalakes: Bytes,
        batched_tasks: Bytes,
    ) -> Result<ProcessedFullInput, PreProcessorError> {
        // 1. decode the tasks
        let tasks = BatchedDatalakeCompute::decode(&batched_datalakes, &batched_tasks)
            .map_err(PreProcessorError::DecodeError)?;
        info!("Target tasks: {:#?}", tasks);

        // wrap into TaskEnvelope
        // TODO: this is temporary, need to remove this
        let tasks = tasks
            .iter()
            .map(|task| TaskEnvelope::DatalakeCompute(task.clone()))
            .collect::<Vec<_>>();
        self.process(tasks).await
    }

    /// User request is pass as input of this function,
    /// First it will generate input structure for preprocessor that need to pass to runner
    /// Then it will run the preprocessor and return the result, fetch points
    /// Fetch points are the values that are required to run the module
    pub async fn process(
        &self,
        tasks: Vec<TaskEnvelope>,
    ) -> Result<ProcessedFullInput, PreProcessorError> {
        let task_commitments: Vec<B256> =
            tasks.iter().map(|task| task.commit()).collect::<Vec<_>>();

        // do compile with the tasks
        let compiled_results = tasks
            .compile(&self.compile_config)
            .await
            .map_err(PreProcessorError::CompileError)?;

        // do operation if possible
        let (tasks_merkle_tree, results_merkle_tree) =
            self.build_merkle_tree(&compiled_results, task_commitments)?;

        // 2. get roots of merkle tree
        let task_merkle_root = tasks_merkle_tree.root();
        let mut combined_tasks = Vec::new();

        for task in tasks {
            match task {
                TaskEnvelope::DatalakeCompute(datalake_compute) => {
                    let task_commitment = datalake_compute.commit();
                    let result = if results_merkle_tree.is_some() {
                        let compiled_result = compiled_results
                            .commit_results_maps
                            .get(&task_commitment)
                            .unwrap();
                        let result_commitment = self
                            ._raw_result_to_result_commitment(&task_commitment, *compiled_result);
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
                    let encoded_task = datalake_compute.encode()?;
                    let datalake_type = datalake_compute.datalake.get_datalake_type();
                    let property_type = datalake_compute.datalake.get_collection_type().to_index();

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
                                Bytes::from(datalake_compute.datalake.encode()?),
                                datalake_type.into(),
                                property_type,
                            )
                        }
                        None => ProcessedDatalakeCompute::new_without_result(
                            Bytes::from(encoded_task),
                            task_commitment,
                            task_proof,
                            Bytes::from(datalake_compute.datalake.encode()?),
                            datalake_type.into(),
                            property_type,
                        ),
                    };

                    // wrap into ProcessedTask
                    let task = ProcessedTask::DatalakeCompute(datalake_compute);
                    combined_tasks.push(task);
                }
                TaskEnvelope::Module(module) => {
                    let task_commitment = module.commit();
                    let compiled_result = compiled_results
                        .commit_results_maps
                        .get(&task_commitment)
                        .unwrap();
                    let module_class = compiled_results
                        .commit_casm_maps
                        .get(&task_commitment)
                        .unwrap();
                    let result_commitment =
                        self._raw_result_to_result_commitment(&task_commitment, *compiled_result);
                    let result_proof = results_merkle_tree
                        .as_ref()
                        .unwrap()
                        .get_proof(&DynSolValue::FixedBytes(result_commitment, 32));
                    let task_proof =
                        tasks_merkle_tree.get_proof(&DynSolValue::FixedBytes(task_commitment, 32));
                    let processed_module = ProcessedModule::new(
                        task_commitment,
                        result_commitment,
                        task_proof,
                        result_proof,
                        module.inputs,
                        module_class.clone(),
                    );

                    let task = ProcessedTask::Module(processed_module);
                    combined_tasks.push(task);
                }
            }
        }

        let proofs = ProcessedBlockProofs {
            mmr_meta: compiled_results.mmr_meta,
            headers: Vec::from_iter(compiled_results.headers),
            accounts: Vec::from_iter(compiled_results.accounts),
            storages: Vec::from_iter(compiled_results.storages),
            transactions: Vec::from_iter(compiled_results.transactions),
            transaction_receipts: Vec::from_iter(compiled_results.transaction_receipts),
        };
        let processed_result = ProcessedFullInput::new(
            results_merkle_tree.map(|tree| tree.root()),
            task_merkle_root,
            proofs,
            combined_tasks,
        );
        // TODO: from compiler result, generate batch for tree and final result that pass through cairo-runner
        info!("Preprocessor completed successfully");
        Ok(processed_result)
    }

    fn build_merkle_tree(
        &self,
        compiled_results: &CompilationResults,
        task_commitments: Vec<B256>,
    ) -> Result<(StandardMerkleTree, Option<StandardMerkleTree>), PreProcessorError> {
        let mut tasks_leaves = Vec::new();
        let mut results_leaves = Vec::new();

        for task_commitment in task_commitments {
            if compiled_results.pre_processable {
                let compiled_result =
                    match compiled_results.commit_results_maps.get(&task_commitment) {
                        Some(result) => result,
                        None => Err(PreProcessorError::TaskCommitmentNotFound)?,
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
