//! Task is a unit of work that can be executed by the processor/pre-processor.
use crate::solidity_types::traits::DatalakeComputeCodecs;
use alloy::primitives::B256;
use datalake::DatalakeCompute;
use module::Module;

pub mod datalake;
pub mod module;

/// [`TaskEnvelope`] is a structure that contains task itself
/// This structure is used to provide the task to the pre-processor
#[derive(Clone, Debug)]
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

    pub fn divide_tasks(tasks: Vec<TaskEnvelope>) -> (Vec<DatalakeCompute>, Vec<Module>) {
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
}
