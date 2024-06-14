//! Task is a unit of work that can be executed by the processor/pre-processor.

use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;

use crate::datalake::task::DatalakeCompute;
use crate::module::Module;

/// [`TaskEnvelope`] is a structure that contains task itself
/// This structure is used to provide the task to the pre-processor
pub enum TaskEnvelope {
    DatalakeCompute(DatalakeCompute),
    Module(Module),
}

impl TaskEnvelope {
    pub fn commit(&self) -> String {
        match self {
            TaskEnvelope::DatalakeCompute(task) => task.commit(),
            TaskEnvelope::Module(module) => module.commit(),
        }
    }
}

/// [`ExtendedTask`] is a structure that contains the task commitment, aggregate values set, compute and module class
/// This structure is used to provide the task to the processor
pub enum ExtendedTask {
    DatalakeCompute(ExtendedDatalake),
    Module(ExtendedModule),
}

pub struct ExtendedDatalake {
    pub task: DatalakeCompute,
    pub aggregate_values_set: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ExtendedModule {
    pub task: Module,
    pub module_class: CasmContractClass,
}
