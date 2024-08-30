use ::serde::{Deserialize, Serialize};
use alloy::primitives::B256;
use std::path::PathBuf;

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
    pub fn into_processor_output(self) -> ProcessorOutput {
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
        let task_results: Vec<B256> = self.tasks.iter().map(|task| task.get_result()).collect();
        let result_commitments: Vec<B256> = self
            .tasks
            .iter()
            .map(|task| task.get_result_commitment())
            .collect();
        let results_inclusion_proofs: Vec<Vec<B256>> = self
            .tasks
            .iter()
            .map(|task| task.get_result_proof())
            .collect();

        ProcessorOutput::new(
            task_results,
            result_commitments,
            tasks_commitments,
            task_inclusion_proofs,
            results_inclusion_proofs,
            self.results_root,
            self.tasks_root,
            self.proofs.mmr_metas,
        )
    }
}
