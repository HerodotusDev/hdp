use alloy::primitives::B256;
use serde::{Deserialize, Serialize};

use super::{datalake_compute::ProcessedDatalakeCompute, module::ProcessedModule};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "context")]
pub enum ProcessedTask {
    #[serde(rename = "datalake_compute")]
    DatalakeCompute(ProcessedDatalakeCompute),
    #[serde(rename = "module")]
    Module(ProcessedModule),
}

impl ProcessedTask {
    pub fn get_task_commitment(&self) -> B256 {
        match self {
            ProcessedTask::DatalakeCompute(datalake_compute) => datalake_compute.task_commitment,
            ProcessedTask::Module(module) => module.task_commitment,
        }
    }

    pub fn get_task_proof(&self) -> Vec<B256> {
        match self {
            ProcessedTask::DatalakeCompute(datalake_compute) => datalake_compute.task_proof.clone(),
            ProcessedTask::Module(module) => module.task_proof.clone(),
        }
    }
}
