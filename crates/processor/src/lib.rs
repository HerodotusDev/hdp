//! Processor is reponsible for running the module.
//! This run is sound execution of the module.
//! This will be most abstract layer of the processor.

use alloy::dyn_abi::DynSolValue;
use alloy::primitives::{FixedBytes, Keccak256, B256, U256};
use alloy_merkle_tree::standard_binary_tree::StandardMerkleTree;
use anyhow::Result;
use hdp_cairo_runner::cairo_run;
use hdp_primitives::processed_types::cairo_format::AsCairoFormat;
use hdp_primitives::processed_types::query::ProcessedFullInput;
use serde::Serialize;
use std::path::PathBuf;
use tracing::info;

pub struct Processor {
    program_path: PathBuf,
}

#[derive(Debug, Serialize)]
pub struct ProcessorResult {
    /// leaf of result merkle tree
    pub task_results: Vec<B256>,
    /// leaf of task merkle tree
    pub task_commitments: Vec<B256>,
    /// tasks inclusion proofs
    pub task_inclusion_proofs: Vec<Vec<FixedBytes<32>>>,
    /// results inclusion proofs
    pub results_inclusion_proofs: Vec<Vec<FixedBytes<32>>>,
    /// root of the results merkle tree
    pub results_root: B256,
    /// root of the tasks merkle tree
    pub tasks_root: B256,
    /// mmr id
    pub used_mmr_id: u64,
    /// mmr size
    pub used_mmr_size: u64,
}

impl ProcessorResult {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        task_results: Vec<B256>,
        task_commitments: Vec<B256>,
        task_inclusion_proofs: Vec<Vec<FixedBytes<32>>>,
        results_inclusion_proofs: Vec<Vec<FixedBytes<32>>>,
        results_root: B256,
        tasks_root: B256,
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
        Self { program_path }
    }

    pub async fn process(
        &self,
        requset: ProcessedFullInput,
        pie_path: PathBuf,
    ) -> Result<ProcessorResult> {
        // 2. generate input struct with proofs and module bytes
        // let input = self.generate_input(requset).await?;
        // 3. pass the input file to the runner
        let input_string = serde_json::to_string_pretty(&requset.as_cairo_format())
            .expect("Failed to serialize module class");
        let result = cairo_run(self.program_path.clone(), input_string, pie_path)?;

        let task_commitments: Vec<B256> = requset
            .tasks
            .iter()
            .map(|task| task.get_task_commitment())
            .collect();
        let task_inclusion_proofs: Vec<Vec<B256>> = requset
            .tasks
            .iter()
            .map(|task| task.get_task_proof())
            .collect();
        let task_root = requset.tasks_root;

        let (results_tree, result_commitments) =
            self.build_result_merkle_tree(task_commitments.clone(), result.task_results.clone())?;
        let results_inclusion_proofs: Vec<_> = result_commitments
            .iter()
            .map(|rc| results_tree.get_proof(&DynSolValue::FixedBytes(*rc, 32)))
            .collect();
        let result_root = results_tree.root();
        let mmr = requset.proofs.mmr_meta.clone();
        let processor_result = ProcessorResult::new(
            result.task_results.iter().map(|x| B256::from(*x)).collect(),
            task_commitments,
            task_inclusion_proofs,
            results_inclusion_proofs,
            result_root,
            task_root,
            mmr.id,
            mmr.size,
        );
        info!("2️⃣ Processor completed successfully");
        Ok(processor_result)
    }

    fn build_result_merkle_tree(
        &self,
        task_commitments: Vec<B256>,
        task_results: Vec<U256>,
    ) -> Result<(StandardMerkleTree, Vec<FixedBytes<32>>)> {
        let mut results_leaves = Vec::new();
        let mut results_commitments = Vec::new();
        for (task_commitment, task_result) in task_commitments.iter().zip(task_results.iter()) {
            dbg!(
                "building result merkle tree | task_commitment: {:?}, task_result: {:?}",
                task_commitment,
                task_result
            );
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
