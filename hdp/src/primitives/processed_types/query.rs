use std::path::PathBuf;

use crate::primitives::merkle_tree::build_result_merkle_tree;
use ::serde::{Deserialize, Serialize};
use alloy::{
    dyn_abi::DynSolValue,
    primitives::{B256, U256},
};

use super::{
    block_proofs::ProcessedBlockProofs, processor_output::ProcessorOutput, task::ProcessedTask,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessorInput {
    /// Path to the directory where the cairo-run output will be stored.
    pub cairo_run_output_path: PathBuf,
    // U256 type
    pub tasks_root: B256,
    // U256 type
    pub results_root: B256,
    pub proofs: ProcessedBlockProofs,
    pub tasks: Vec<ProcessedTask>,
}

impl ProcessorInput {
    pub fn new(
        cairo_run_output_path: PathBuf,
        results_root: B256,
        tasks_root: B256,
        proofs: ProcessedBlockProofs,
        tasks: Vec<ProcessedTask>,
    ) -> Self {
        Self {
            cairo_run_output_path,
            results_root,
            tasks_root,
            proofs,
            tasks,
        }
    }

    /// Turn [`ProcessorInput`] into [`ProcessorOutput`] by provided task results
    pub fn into_processor_output(self, task_results: Vec<U256>) -> ProcessorOutput {
        let tasks_commitments: Vec<B256> = self
            .tasks
            .iter()
            .map(|task| task.get_task_commitment())
            .collect();
        let task_inclusion_proofs: Vec<Vec<B256>> = self
            .tasks
            .iter()
            .map(|task| task.get_task_proof())
            .collect();
        let (results_tree, result_commitments) =
            build_result_merkle_tree(&tasks_commitments, &task_results);
        let results_inclusion_proofs: Vec<Vec<B256>> = result_commitments
            .iter()
            .map(|rc| results_tree.get_proof(&DynSolValue::FixedBytes(*rc, 32)))
            .collect();
        let result_root = results_tree.root();

        ProcessorOutput::new(
            task_results.iter().map(|x| B256::from(*x)).collect(),
            result_commitments,
            tasks_commitments,
            task_inclusion_proofs,
            results_inclusion_proofs,
            result_root,
            self.tasks_root,
            self.proofs.mmr_metas,
        )
    }
}
