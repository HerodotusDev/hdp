use hdp_primitives::task::TaskEnvelope;

use super::{Compilable, CompilationResults, CompileConfig, CompileError};

impl Compilable for Vec<TaskEnvelope> {
    async fn compile(
        &self,
        compile_config: &CompileConfig,
    ) -> Result<CompilationResults, CompileError> {
        let (datalakes, modules) = TaskEnvelope::divide_tasks(self.to_vec());
        let mut datalake_compile_results = datalakes.compile(compile_config).await?;
        let module_compile_results = modules.compile(compile_config).await?;
        datalake_compile_results.extend(module_compile_results);

        Ok(datalake_compile_results)
    }
}
