use hdp_primitives::processed_types::{
    block_proofs::ProcessedBlockProofs, cairo_format, module::ProcessedModule,
};
use serde::Serialize;

/*
    input.json file that will be passed to the processor, is generated by this struct.
*/

#[derive(Serialize)]
pub struct RunnerProgramInput {
    /// Batched tasks root of all tasks.
    pub task_root: String,
    /// if every tasks are pre computable, this can be Some
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_root: Option<String>,
    /// Fetched proofs per each fetch point.
    pub proofs: ProcessedBlockProofs,
    /// tasks compatible with v2
    pub tasks: Vec<InputTask>,
}

#[derive(Serialize)]
pub enum InputTask {
    #[serde(rename = "datalake_compute")]
    DatalakeCompute(cairo_format::ProcessedDatalakeCompute),
    #[serde(rename = "module")]
    Module(ProcessedModule),
}

impl RunnerProgramInput {
    pub fn new(proofs: ProcessedBlockProofs, task_root: String, tasks: Vec<InputTask>) -> Self {
        Self {
            task_root,
            result_root: None,
            tasks,
            proofs,
        }
    }
}
