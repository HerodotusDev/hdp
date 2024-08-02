//! Task is a unit of work that can be executed by the processor/pre-processor.
use crate::primitives::solidity_types::traits::DatalakeComputeCodecs;
use alloy::primitives::B256;
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use datalake::DatalakeCompute;
use module::Module;

pub mod datalake;
pub mod module;

/// [`TaskEnvelope`] is a structure that contains task itself
/// This structure is used to provide the task to the pre-processor
#[derive(Clone, Debug)]
pub enum TaskEnvelope {
    DatalakeCompute(DatalakeCompute),
    Module(ExtendedModule),
}

#[derive(Clone, Debug)]
pub struct ExtendedModule {
    pub task: Module,
    pub module_class: CasmContractClass,
}

impl TaskEnvelope {
    pub fn commit(&self) -> B256 {
        match self {
            TaskEnvelope::DatalakeCompute(task) => task.commit(),
            TaskEnvelope::Module(module) => module.task.commit(),
        }
    }

    pub fn divide_tasks(tasks: Vec<TaskEnvelope>) -> (Vec<DatalakeCompute>, Vec<ExtendedModule>) {
        // Partition the tasks into datalake and module tasks
        let (datalake_envelopes, module_envelopes): (Vec<_>, Vec<_>) = tasks
            .into_iter()
            .partition(|task| matches!(task, TaskEnvelope::DatalakeCompute(_)));

        let datalake_tasks = datalake_envelopes
            .into_iter()
            .filter_map(|task| {
                if let TaskEnvelope::DatalakeCompute(datalake_task) = task {
                    Some(datalake_task)
                } else {
                    None
                }
            })
            .collect();

        let module_tasks = module_envelopes
            .into_iter()
            .filter_map(|task| {
                if let TaskEnvelope::Module(module_task) = task {
                    Some(module_task)
                } else {
                    None
                }
            })
            .collect();

        (datalake_tasks, module_tasks)
    }

    pub fn variants() -> Vec<String> {
        vec!["DATALAKE_COMPUTE", "MODULE"]
            .into_iter()
            .map(String::from)
            .collect()
    }
}
