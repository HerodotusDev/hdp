use alloy::primitives::B256;
use serde::Serialize;

use super::mmr::MMRMeta;

#[derive(Debug, Serialize)]
pub struct ProcessorOutput {
    /// raw results of the module
    pub raw_results: Vec<B256>,
    /// leaf of result merkle tree
    pub results_commitments: Vec<B256>,
    /// leaf of task merkle tree
    pub tasks_commitments: Vec<B256>,
    /// tasks inclusion proofs
    pub task_inclusion_proofs: Vec<Vec<B256>>,
    /// results inclusion proofs
    pub results_inclusion_proofs: Vec<Vec<B256>>,
    /// root of the results merkle tree
    pub results_root: B256,
    /// root of the tasks merkle tree
    pub tasks_root: B256,
    /// mmr metas related to processed tasks
    pub mmr_metas: Vec<MMRMeta>,
}

impl ProcessorOutput {
    pub fn new(
        raw_results: Vec<B256>,
        results_commitments: Vec<B256>,
        tasks_commitments: Vec<B256>,
        task_inclusion_proofs: Vec<Vec<B256>>,
        results_inclusion_proofs: Vec<Vec<B256>>,
        results_root: B256,
        tasks_root: B256,
        mmr_metas: Vec<MMRMeta>,
    ) -> Self {
        Self {
            raw_results,
            results_commitments,
            tasks_commitments,
            task_inclusion_proofs,
            results_inclusion_proofs,
            results_root,
            tasks_root,
            mmr_metas,
        }
    }
}
