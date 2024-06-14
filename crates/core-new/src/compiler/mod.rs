use std::collections::HashSet;

use anyhow::Result;
use hdp_primitives::{
    datalake::task::DatalakeCompute,
    module::Module,
    task::{ExtendedTask, TaskEnvelope},
};
use hdp_provider::key::FetchKeyEnvelope;
use module::ModuleCompilerConfig;

pub mod datalake;
pub mod module;

pub struct Compiler {
    pub(crate) datalake: datalake::DatalakeCompiler,
    pub(crate) module: module::ModuleCompiler,
}

impl Compiler {
    pub fn new(config: ModuleCompilerConfig) -> Self {
        Self {
            datalake: datalake::DatalakeCompiler::new(),
            module: module::ModuleCompiler::new_with_config(config),
        }
    }

    pub async fn compile(
        &self,
        tasks: Vec<TaskEnvelope>,
    ) -> Result<(HashSet<FetchKeyEnvelope>, Vec<ExtendedTask>)> {
        let (datalake_tasks, module_tasks) = self.split_tasks(tasks);
        let (datalake_fetch_keys, datalake_result) = self.datalake.compile(datalake_tasks, 1)?;
        let (module_fetch_keys, module_result) = self.module.compile(module_tasks).await?;

        // combine fetch keys
        let mut fetch_keys = datalake_fetch_keys;
        fetch_keys.extend(module_fetch_keys);

        // combine results
        let mut result = Vec::new();
        result.extend(module_result.into_iter().map(ExtendedTask::Module));
        result.extend(
            datalake_result
                .into_iter()
                .map(ExtendedTask::DatalakeCompute),
        );
        Ok((fetch_keys, result))
    }

    fn split_tasks(&self, tasks: Vec<TaskEnvelope>) -> (Vec<DatalakeCompute>, Vec<Module>) {
        let mut datalake_tasks = Vec::new();
        let mut module_tasks = Vec::new();
        for task in tasks {
            match task {
                TaskEnvelope::DatalakeCompute(datalake_task) => datalake_tasks.push(datalake_task),
                TaskEnvelope::Module(module_task) => module_tasks.push(module_task),
            }
        }
        (datalake_tasks, module_tasks)
    }
}
