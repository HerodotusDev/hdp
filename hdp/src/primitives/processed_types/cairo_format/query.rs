use std::path::PathBuf;

use crate::primitives::processed_types::query::ProcessorInput as BasedProcessorInput;
use ::serde::Serialize;
use alloy::primitives::B256;
use serde::Deserialize;

use super::{AsCairoFormat, ProcessedBlockProofs, ProcessedTask};

impl BasedProcessorInput {
    pub fn as_cairo_format(&self, cairo_run_output_path: PathBuf) -> ProcessorInput {
        ProcessorInput {
            cairo_run_output_path: cairo_run_output_path.clone(),
            task_root: self.tasks_root,
            result_root: self.results_root,
            proofs: self
                .proofs
                .iter()
                .map(|proof| proof.as_cairo_format())
                .collect(),
            tasks: self
                .tasks
                .iter()
                .map(|task| task.as_cairo_format())
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ProcessorInput {
    /// Path to the directory where the cairo-run output will be stored.
    pub cairo_run_output_path: PathBuf,
    /// Batched tasks root of all tasks.
    pub task_root: B256,
    /// Batched results root of all tasks.
    pub result_root: B256,
    /// Fetched proofs per each fetch point.
    pub proofs: Vec<ProcessedBlockProofs>,
    /// tasks to be executed.
    pub tasks: Vec<ProcessedTask>,
}
