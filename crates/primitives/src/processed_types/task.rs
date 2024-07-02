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
