use std::path::PathBuf;

use crate::processed_types::query::ProcessorInput as BasedProcessorInput;
use ::serde::Serialize;
use alloy::primitives::B256;

use super::{AsCairoFormat, ProcessedBlockProofs, ProcessedTask};

impl AsCairoFormat for BasedProcessorInput {
    type Output = ProcessorInput;

    fn as_cairo_format(&self) -> Self::Output {
        ProcessorInput {
            cairo_run_output_path: self.cairo_run_output_path.clone(),
            task_root: self.tasks_root,
            result_root: self.results_root,
            proofs: self.proofs.as_cairo_format(),
            tasks: self
                .tasks
                .iter()
                .map(|task| task.as_cairo_format())
                .collect(),
        }
    }
}

#[derive(Serialize)]
pub struct ProcessorInput {
    /// Path to the directory where the cairo-run output will be stored.
    pub cairo_run_output_path: PathBuf,
    /// Batched tasks root of all tasks.
    pub task_root: B256,
    /// if every tasks are pre computable, this can be Some
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_root: Option<B256>,
    /// Fetched proofs per each fetch point.
    pub proofs: ProcessedBlockProofs,
    /// tasks to be executed.
    pub tasks: Vec<ProcessedTask>,
}
