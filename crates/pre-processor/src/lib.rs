//!  Preprocessor is reponsible for identifying the required values.
//!  This will be most abstract layer of the preprocessor.

use alloy::dyn_abi::DynSolValue;
use alloy::primitives::{Bytes, B256};
use compile::config::CompilerConfig;
use compile::{Compilable, CompileError};
use hdp_primitives::constant::SOUND_CAIRO_RUN_OUTPUT_FILE;
use hdp_primitives::merkle_tree::{build_result_merkle_tree, build_task_merkle_tree};
use hdp_primitives::processed_types::block_proofs::ProcessedBlockProofs;
use hdp_primitives::processed_types::datalake_compute::ProcessedDatalakeCompute;
use hdp_primitives::processed_types::module::ProcessedModule;
use hdp_primitives::processed_types::query::ProcessorInput;
use hdp_primitives::processed_types::task::ProcessedTask;
use hdp_primitives::solidity_types::traits::{DatalakeCodecs, DatalakeComputeCodecs};
use hdp_primitives::task::TaskEnvelope;

use thiserror::Error;
use tracing::info;

pub mod compile;
pub mod constant;
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
    pub compile_config: CompilerConfig,
}

impl PreProcessor {
    pub fn new_with_config(compile_config: CompilerConfig) -> Self {
        Self { compile_config }
    }

    /// User request is pass as input of this function,
    /// First it will generate input structure for preprocessor that need to pass to runner
    /// Then it will run the preprocessor and return the result, fetch points
    /// Fetch points are the values that are required to run the module
    pub async fn process(
        &self,
        tasks: Vec<TaskEnvelope>,
    ) -> Result<ProcessorInput, PreProcessorError> {
        // 1. compile the given tasks
        let compiled_results = tasks
            .compile(&self.compile_config)
            .await
            .map_err(PreProcessorError::CompileError)?;

        let tasks_commitments: Vec<B256> =
            tasks.iter().map(|task| task.commit()).collect::<Vec<_>>();
        let tasks_merkle_tree = build_task_merkle_tree(&tasks_commitments);
        let results_merkle_tree_result = if compiled_results.pre_processable {
            Some(build_result_merkle_tree(
                &tasks_commitments,
                &compiled_results.task_results,
            ))
        } else {
            None
        };

        let task_merkle_root = tasks_merkle_tree.root();
        let mut combined_tasks = Vec::new();

        for (i, task) in tasks.into_iter().enumerate() {
            match task {
                TaskEnvelope::DatalakeCompute(datalake_compute) => {
                    let task_commitment = datalake_compute.commit();
                    let result = if results_merkle_tree_result.is_some() {
                        let (result_merkle_tree, results_commitments) =
                            results_merkle_tree_result.as_ref().unwrap();
                        let compiled_result = compiled_results.task_results[i];
                        let result_commitment = results_commitments[i];
                        let result_proof = result_merkle_tree
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
                                compiled_result,
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
                    let task_commitment = module.task.commit();
                    let encoded_task = module.task.encode_task();
                    let (result_merkle_tree, results_commitments) =
                        results_merkle_tree_result.as_ref().unwrap();
                    let result_commitment = results_commitments[i];
                    let result_proof = result_merkle_tree
                        .get_proof(&DynSolValue::FixedBytes(result_commitment, 32));
                    let task_proof =
                        tasks_merkle_tree.get_proof(&DynSolValue::FixedBytes(task_commitment, 32));
                    let processed_module = ProcessedModule::new(
                        Bytes::from(encoded_task),
                        task_commitment,
                        result_commitment,
                        task_proof,
                        result_proof,
                        module.task.inputs,
                        module.module_class,
                    );

                    let task = ProcessedTask::Module(processed_module);
                    combined_tasks.push(task);
                }
            }
        }

        let proofs = ProcessedBlockProofs {
            mmr_metas: Vec::from_iter(compiled_results.mmr_metas),
            headers: Vec::from_iter(compiled_results.headers),
            accounts: Vec::from_iter(compiled_results.accounts),
            storages: Vec::from_iter(compiled_results.storages),
            transactions: Vec::from_iter(compiled_results.transactions),
            transaction_receipts: Vec::from_iter(compiled_results.transaction_receipts),
        };
        let processed_result = ProcessorInput::new(
            SOUND_CAIRO_RUN_OUTPUT_FILE.into(),
            results_merkle_tree_result
                .as_ref()
                .map(|tree| tree.0.root()),
            task_merkle_root,
            proofs,
            combined_tasks,
        );
        info!("1️⃣  Preprocessor completed successfully");
        Ok(processed_result)
    }
}
