//! Processor is reponsible for running the module.
//! This run is sound execution of the module.
//! This will be most abstract layer of the processor.

use alloy_dyn_abi::DynSolValue;
use alloy_merkle_tree::standard_binary_tree::StandardMerkleTree;
use alloy_primitives::{hex::FromHex, FixedBytes, Keccak256, B256, U256};
use anyhow::Result;
use hdp_primitives::processed_types::{cairo_format::AsCairoFormat, v1_query::ProcessedResult};
use serde::Serialize;
use std::{path::PathBuf, str::FromStr};

use hdp_provider::evm::{AbstractProvider, AbstractProviderConfig};

use crate::cairo_runner::run::Runner;

pub struct Processor {
    runner: Runner,
    _provider: AbstractProvider,
}

#[derive(Debug, Serialize)]
pub struct ProcessorResult {
    /// leaf of result merkle tree
    pub task_results: Vec<String>,
    /// leaf of task merkle tree
    pub task_commitments: Vec<String>,
    /// tasks inclusion proofs
    pub task_inclusion_proofs: Vec<Vec<FixedBytes<32>>>,
    /// results inclusion proofs
    pub results_inclusion_proofs: Vec<Vec<FixedBytes<32>>>,
    /// root of the results merkle tree
    pub results_root: String,
    /// root of the tasks merkle tree
    pub tasks_root: String,
    /// mmr id
    pub used_mmr_id: u64,
    /// mmr size
    pub used_mmr_size: u64,
}

impl ProcessorResult {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        task_results: Vec<String>,
        task_commitments: Vec<String>,
        task_inclusion_proofs: Vec<Vec<FixedBytes<32>>>,
        results_inclusion_proofs: Vec<Vec<FixedBytes<32>>>,
        results_root: String,
        tasks_root: String,
        used_mmr_id: u64,
        used_mmr_size: u64,
    ) -> Self {
        Self {
            task_results,
            task_commitments,
            task_inclusion_proofs,
            results_inclusion_proofs,
            results_root,
            tasks_root,
            used_mmr_id,
            used_mmr_size,
        }
    }
}

impl Processor {
    pub fn new(provider_config: AbstractProviderConfig, program_path: PathBuf) -> Self {
        let runner = Runner::new(program_path);
        let provider = AbstractProvider::new(provider_config);
        Self {
            runner,
            _provider: provider,
        }
    }

    pub async fn process(
        &self,
        requset: ProcessedResult,
        pie_path: String,
    ) -> Result<ProcessorResult> {
        // generate input file from fetch points
        // 1. fetch proofs from provider by using fetch points
        // TODO: only for module
        // let proofs = self
        //     .provider
        //     .fetch_proofs_from_keys(requset.fetch_keys)
        //     .await?;

        // 2. generate input struct with proofs and module bytes
        // let input = self.generate_input(requset).await?;
        // 3. pass the input file to the runner
        let input_string = serde_json::to_string_pretty(&requset.as_cairo_format())
            .expect("Failed to serialize module class");
        let result = self.runner.run(input_string, PathBuf::from(pie_path))?;

        let task_commitments: Vec<String> = requset
            .tasks
            .iter()
            .map(|task| task.task_commitment.clone())
            .collect();
        let task_inclusion_proofs: Vec<_> = requset
            .tasks
            .iter()
            .map(|task| task.task_proof.clone())
            .collect();

        let task_root = requset.tasks_root.clone();

        let (results_tree, result_commitments) =
            self.build_result_merkle_tree(task_commitments.clone(), result.task_results.clone())?;
        let results_inclusion_proofs: Vec<_> = result_commitments
            .iter()
            .map(|rc| results_tree.get_proof(&DynSolValue::FixedBytes(*rc, 32)))
            .collect();
        let result_root = results_tree.root().to_string();
        let mmr = requset.mmr.clone();

        Ok(ProcessorResult::new(
            result.task_results,
            task_commitments,
            task_inclusion_proofs,
            results_inclusion_proofs,
            result_root,
            task_root,
            mmr.id,
            mmr.size,
        ))
    }

    fn build_result_merkle_tree(
        &self,
        task_commitments: Vec<String>,
        task_results: Vec<String>,
    ) -> Result<(StandardMerkleTree, Vec<FixedBytes<32>>)> {
        let mut results_leaves = Vec::new();
        let mut results_commitments = Vec::new();
        for (task_commitment, task_result) in task_commitments.iter().zip(task_results.iter()) {
            let result_commitment =
                self._raw_result_to_result_commitment(task_commitment, task_result);
            results_commitments.push(result_commitment);
            results_leaves.push(DynSolValue::FixedBytes(result_commitment, 32));
        }
        let tree = StandardMerkleTree::of(results_leaves);
        Ok((tree, results_commitments))
    }

    fn _raw_result_to_result_commitment(
        &self,
        task_commitment: &str,
        compiled_result: &str,
    ) -> FixedBytes<32> {
        let mut hasher = Keccak256::new();
        hasher.update(Vec::from_hex(task_commitment).unwrap());
        hasher.update(B256::from(U256::from_str(compiled_result).unwrap()));
        hasher.finalize()
    }
}