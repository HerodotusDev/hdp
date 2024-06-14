use crate::datalake::task::{DatalakeCompute, ExtendedDatalakeTask};
use crate::module::{ExtendedModuleTask, Module};

/// Most abstract structure that contains the task
pub enum TaskEnvelope {
    Datalake(DatalakeCompute),
    Module(Module),
}

impl TaskEnvelope {
    pub fn commit(&self) -> String {
        match self {
            TaskEnvelope::Datalake(task) => task.commit(),
            TaskEnvelope::Module(module) => module.commit(),
        }
    }
}

/// Extended task envelope that contains the information that processor requires
pub enum ExtendedTaskEnvelope {
    Datalake(ExtendedDatalakeTask),
    Module(ExtendedModuleTask),
}
