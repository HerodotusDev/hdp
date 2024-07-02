use ::serde::{Deserialize, Serialize};

use super::{block_proofs::ProcessedBlockProofs, task::ProcessedTask};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessedFullInput {
    // U256 type
    pub tasks_root: String,
    // U256 type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results_root: Option<String>,
    pub proofs: ProcessedBlockProofs,
    pub tasks: Vec<ProcessedTask>,
}

impl ProcessedFullInput {
    pub fn new(
        results_root: Option<String>,
        tasks_root: String,
        proofs: ProcessedBlockProofs,
        tasks: Vec<ProcessedTask>,
    ) -> Self {
        Self {
            results_root,
            tasks_root,
            proofs,
            tasks,
        }
    }

    pub fn update_results_root(&mut self, results_root: String) {
        self.results_root = Some(results_root);
    }
}
