//! Processor is reponsible for running the module.
//! This run is sound execution of the module.
//! This will be most abstract layer of the processor.

use alloy_dyn_abi::DynSolValue;
use alloy_merkle_tree::standard_binary_tree::StandardMerkleTree;
use alloy_primitives::B256;
use anyhow::Result;
use hdp_primitives::processed_types::{
    cairo_format::AsCairoFormat, datalake_compute::ProcessedDatalakeCompute,
    module::ProcessedModule,
};
use std::{fs, path::PathBuf, str::FromStr};

use hdp_provider::evm::{AbstractProvider, AbstractProviderConfig, ProcessedBlockProofs};
use tracing::info;

use crate::{
    cairo_runner::{
        input::run::{InputTask, RunnerInput},
        run::{RunResult, Runner},
    },
    pre_processor::{ExtendedTask, PreProcessResult},
};

pub struct Processor {
    runner: Runner,
    provider: AbstractProvider,
}

#[derive(Debug)]
pub struct ProcessorResult {
    /// leaf of result merkle tree
    task_results: Vec<String>,
    /// leaf of task merkle tree
    task_commitments: Vec<String>,
    /// tasks inclusion proofs
    task_inclusion_proofs: Vec<Vec<String>>,
    /// results inclusion proofs
    results_inclusion_proofs: Vec<Vec<String>>,
    /// root of the results merkle tree
    results_root: String,
    /// root of the tasks merkle tree
    tasks_root: String,
    /// mmr id
    used_mmr_id: u64,
    /// mmr size
    used_mmr_size: u64,
}

impl Processor {
    pub fn new(provider_config: AbstractProviderConfig, program_path: PathBuf) -> Self {
        let runner = Runner::new(program_path);
        let provider = AbstractProvider::new(provider_config);
        Self { runner, provider }
    }

    pub async fn process(&self, requset: PreProcessResult) -> Result<ProcessorResult> {
        // generate input file from fetch points
        // 1. fetch proofs from provider by using fetch points
        let proofs = self
            .provider
            .fetch_proofs_from_keys(requset.fetch_keys)
            .await?;

        println!("Proofs: {:?}", proofs);
        // TODO 2. pre-compute tasks
        // from requets.tasks.fetch keys -> value sets

        // 2. generate input struct with proofs and module bytes
        let input = self.generate_input(proofs, requset.tasks).await?;
        // 3. pass the input file to the runner
        let input_string =
            serde_json::to_string_pretty(&input).expect("Failed to serialize module class");
        fs::write("input_processor.json", input_string.clone()).expect("Unable to write file");
        let result = self.runner.run(input_string)?;
        info!("Processor executed successfully, PIE is generated");
        todo!("Return what execution store contract requires")
    }

    async fn generate_input(
        &self,
        proofs: ProcessedBlockProofs,
        tasks: Vec<ExtendedTask>,
    ) -> Result<RunnerInput> {
        let (task_tree, wrapped_tasks) = self.build_task_merkle_tree(tasks)?;
        let input_data = RunnerInput::new(proofs, task_tree.root().to_string(), wrapped_tasks);
        info!("Runner input is generated successfully");
        Ok(input_data)
    }

    fn build_task_merkle_tree(
        &self,
        tasks: Vec<ExtendedTask>,
    ) -> Result<(StandardMerkleTree, Vec<InputTask>)> {
        let mut task_wrapper: Vec<InputTask> = Vec::new();
        let task_commits = tasks
            .iter()
            .map(|task| task.get_task_commitment())
            .collect::<Vec<_>>();
        let tasks_leaves = task_commits
            .iter()
            .map(|commit| DynSolValue::FixedBytes(B256::from_str(commit).unwrap(), 32))
            .collect::<Vec<_>>();
        let tasks_merkle_tree = StandardMerkleTree::of(tasks_leaves);
        for (index, target_task) in tasks.into_iter().enumerate() {
            let task_commit = task_commits.get(index).unwrap();
            match target_task {
                ExtendedTask::DatalakeCompute(extended_datalake) => {
                    let task_proof = tasks_merkle_tree.get_proof(&DynSolValue::FixedBytes(
                        B256::from_str(task_commit).unwrap(),
                        32,
                    ));
                    let encoded_task = extended_datalake.task.encode()?;
                    let encoded_datalake = extended_datalake.task.datalake.encode()?;
                    let datalake_type = extended_datalake.task.datalake.get_datalake_type() as u8;
                    let property_type = extended_datalake
                        .task
                        .datalake
                        .get_collection_type()
                        .to_index();
                    let wrapped_task = ProcessedDatalakeCompute {
                        encoded_task,
                        task_commitment: task_commit.clone(),
                        compiled_result: None,
                        result_commitment: None,
                        task_proof,
                        result_proof: None,
                        encoded_datalake,
                        datalake_type,
                        property_type,
                    };
                    task_wrapper.push(InputTask::DatalakeCompute(wrapped_task.as_cairo_format()));
                }
                ExtendedTask::Module(extended_module) => {
                    let wrapped_task = ProcessedModule::new(
                        extended_module.task.inputs,
                        extended_module.module_class,
                    );
                    task_wrapper.push(InputTask::Module(wrapped_task));
                }
            };
        }
        info!("Task merkle tree is built successfully");
        Ok((tasks_merkle_tree, task_wrapper))
    }
}
