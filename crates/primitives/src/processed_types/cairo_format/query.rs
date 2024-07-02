use crate::processed_types::query::ProcessedFullInput as BasedProcessedFullInput;
use ::serde::Serialize;

use super::{AsCairoFormat, ProcessedBlockProofs, ProcessedTask};

impl AsCairoFormat for BasedProcessedFullInput {
    type Output = ProcessedFullInput;

    fn as_cairo_format(&self) -> Self::Output {
        ProcessedFullInput {
            task_root: self.tasks_root.clone(),
            result_root: self.results_root.clone(),
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
pub struct ProcessedFullInput {
    /// Batched tasks root of all tasks.
    pub task_root: String,
    /// if every tasks are pre computable, this can be Some
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_root: Option<String>,
    /// Fetched proofs per each fetch point.
    pub proofs: ProcessedBlockProofs,
    /// tasks to be executed.
    pub tasks: Vec<ProcessedTask>,
}
