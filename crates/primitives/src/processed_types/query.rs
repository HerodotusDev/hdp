use std::path::PathBuf;

use ::serde::{Deserialize, Serialize};
use alloy::primitives::B256;

use super::{block_proofs::ProcessedBlockProofs, task::ProcessedTask};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessedFullInput {
    /// Path to the directory where the cairo-run output will be stored.
    pub cairo_run_output_path: PathBuf,
    // U256 type
    pub tasks_root: B256,
    // U256 type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results_root: Option<B256>,
    pub proofs: ProcessedBlockProofs,
    pub tasks: Vec<ProcessedTask>,
}

impl ProcessedFullInput {
    pub fn new(
        cairo_run_output_path: PathBuf,
        results_root: Option<B256>,
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
}
