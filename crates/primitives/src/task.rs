//! Task is a unit of work that can be executed by the processor/pre-processor.

use alloy::primitives::B256;

use crate::datalake::task::DatalakeCompute;
use crate::module::Module;

/// [`TaskEnvelope`] is a structure that contains task itself
/// This structure is used to provide the task to the pre-processor
pub enum TaskEnvelope {
    DatalakeCompute(DatalakeCompute),
    Module(Module),
}

impl TaskEnvelope {
    pub fn commit(&self) -> B256 {
        match self {
            TaskEnvelope::DatalakeCompute(task) => task.commit(),
            TaskEnvelope::Module(module) => module.commit(),
        }
    }
}
