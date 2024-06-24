use anyhow::Result;
use datalake_compute::DatalakeComputeCompilationResults;
use hdp_primitives::datalake::task::DatalakeCompute;
use hdp_provider::evm::AbstractProviderConfig;
use module::ModuleCompilerConfig;

pub mod datalake_compute;
pub mod module;

pub struct Compiler {
    pub(crate) datalake: datalake_compute::DatalakeCompiler,
    // pub(crate) module: module::ModuleCompiler,
}

pub struct CompilerConfig {
    datalake_config: AbstractProviderConfig,
    #[allow(dead_code)]
    module_config: ModuleCompilerConfig,
}

impl CompilerConfig {
    pub fn new(
        datalake_config: AbstractProviderConfig,
        module_config: ModuleCompilerConfig,
    ) -> Self {
        Self {
            datalake_config,
            module_config,
        }
    }
}

impl Compiler {
    pub fn new(config: CompilerConfig) -> Self {
        Self {
            datalake: datalake_compute::DatalakeCompiler::new_from_config(config.datalake_config),
            // module: module::ModuleCompiler::new_with_config(config),
        }
    }

    // TODO: later turn result into generic for both datalake and module
    pub async fn compile(
        &self,
        datalake_tasks: &[DatalakeCompute],
    ) -> Result<DatalakeComputeCompilationResults> {
        //   let (datalake_tasks, _module_tasks) = self.split_tasks(tasks);
        let datalake_result = self.datalake.compile(datalake_tasks).await?;
        Ok(datalake_result)
    }

    // fn split_tasks(&self, tasks: &[TaskEnvelope]) -> (&[DatalakeCompute], &[Module]) {
    //     let mut datalake_tasks = Vec::new();
    //     let mut module_tasks = Vec::new();
    //     for task in tasks {
    //         match task {
    //             TaskEnvelope::DatalakeCompute(datalake_task) => datalake_tasks.push(datalake_task),
    //             TaskEnvelope::Module(module_task) => module_tasks.push(module_task),
    //         }
    //     }
    //     (datalake_tasks, module_tasks)
    // }
}
