//! Processor is reponsible for running the module.
//! This run is sound execution of the module.
//! This will be most abstract layer of the processor.

use alloy::dyn_abi::DynSolValue;
use alloy::primitives::{FixedBytes, Keccak256, B256, U256};
use alloy_merkle_tree::standard_binary_tree::StandardMerkleTree;
use anyhow::Result;
use hdp_primitives::processed_types::{
    cairo_format::AsCairoFormat, datalake_compute::ProcessedDatalakeCompute,
    v1_query::ProcessedResult,
};
use serde::Serialize;
use std::path::PathBuf;

use crate::cairo_runner::run::{RunResult, Runner};

pub struct Processor {
    runner: Runner,
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
    pub fn new(program_path: PathBuf) -> Self {
        let runner = Runner::new(program_path);
        Self { runner }
    }

    pub async fn process(
        &self,
        requset: ProcessedResult,
        pie_path: String,
    ) -> Result<ProcessedResult> {
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

        let pr = self.build_legacy_output_file(requset, result)?;

        // let task_commitments: Vec<String> = requset
        //     .tasks
        //     .iter()
        //     .map(|task| task.task_commitment.clone())
        //     .collect();
        // let task_inclusion_proofs: Vec<_> = requset
        //     .tasks
        //     .iter()
        //     .map(|task| task.task_proof.clone())
        //     .collect();

        // let task_root = requset.tasks_root.clone();

        // let (results_tree, result_commitments) =
        //     self.build_result_merkle_tree(task_commitments.clone(), result.task_results.clone())?;
        // let results_inclusion_proofs: Vec<_> = result_commitments
        //     .iter()
        //     .map(|rc| results_tree.get_proof(&DynSolValue::FixedBytes(*rc, 32)))
        //     .collect();
        // let result_root = results_tree.root().to_string();
        // let mmr = requset.mmr.clone();

        Ok(pr)
    }

    // TODO: for now, we are using the legacy output file format.
    fn build_legacy_output_file(
        &self,
        requset: ProcessedResult,
        result: RunResult,
    ) -> Result<ProcessedResult> {
        let task_commitments: Vec<B256> = requset
            .tasks
            .iter()
            .map(|task| task.task_commitment)
            .collect();
        // let task_inclusion_proofs: Vec<_> = requset
        //     .tasks
        //     .iter()
        //     .map(|task| task.task_proof.clone())
        //     .collect();

        let task_root = requset.tasks_root.clone();
        let (results_tree, result_commitments) =
            self.build_result_merkle_tree(task_commitments.clone(), result.task_results.clone())?;
        let results_inclusion_proofs: Vec<_> = result_commitments
            .iter()
            .map(|rc| results_tree.get_proof(&DynSolValue::FixedBytes(*rc, 32)))
            .collect();
        let result_root = results_tree.root().to_string();

        let mut new_tasks: Vec<ProcessedDatalakeCompute> = Vec::new();
        for (idx, mut task) in requset.tasks.into_iter().enumerate() {
            let compiled_result = result.task_results[idx];
            let result_commitment = result_commitments[idx];
            let result_proof = results_inclusion_proofs[idx].clone();
            task.update_results(compiled_result, result_commitment, result_proof);
            new_tasks.push(task.clone());
        }

        let new_final_processed_result = ProcessedResult {
            results_root: Some(result_root),
            tasks_root: task_root,
            headers: requset.headers,
            mmr: requset.mmr,
            accounts: requset.accounts,
            storages: requset.storages,
            transactions: requset.transactions,
            transaction_receipts: requset.transaction_receipts,
            tasks: new_tasks,
        };
        Ok(new_final_processed_result)
    }

    fn build_result_merkle_tree(
        &self,
        task_commitments: Vec<B256>,
        task_results: Vec<U256>,
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
        task_commitment: &B256,
        compiled_result: &U256,
    ) -> B256 {
        let mut hasher = Keccak256::new();
        hasher.update(task_commitment);
        hasher.update(B256::from(*compiled_result));
        hasher.finalize()
    }
}
