use super::{module::ProcessedModule, AsCairoFormat, ProcessedDatalakeCompute};
use crate::processed_types::task::ProcessedTask as BaseProcessedTask;
use ::serde::Serialize;

impl AsCairoFormat for BaseProcessedTask {
    type Output = ProcessedTask;

    fn as_cairo_format(&self) -> Self::Output {
        match self {
            BaseProcessedTask::DatalakeCompute(datalake_compute) => {
                ProcessedTask::DatalakeCompute(datalake_compute.as_cairo_format())
            }
            BaseProcessedTask::Module(module) => ProcessedTask::Module(module.as_cairo_format()),
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", content = "context")]
pub enum ProcessedTask {
    #[serde(rename = "datalake_compute")]
    DatalakeCompute(ProcessedDatalakeCompute),
    #[serde(rename = "module")]
    Module(ProcessedModule),
}
