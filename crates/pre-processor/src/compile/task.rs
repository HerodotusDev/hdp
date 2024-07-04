use hdp_primitives::task::TaskEnvelope;

use super::{Compilable, CompilationResults, CompileConfig, CompileError};

impl Compilable for Vec<TaskEnvelope> {
    async fn compile(
        &self,
        compile_config: &CompileConfig,
    ) -> Result<CompilationResults, CompileError> {
        let (datalakes, modules) = TaskEnvelope::divide_tasks(self.to_vec());
        let mut compiled_result = if !datalakes.is_empty() {
            datalakes.compile(compile_config).await?
        } else {
            CompilationResults::default()
        };

        let module_compile_result = if !modules.is_empty() {
            modules.compile(compile_config).await?
        } else {
            CompilationResults::default()
        };
        compiled_result.extend(module_compile_result);
        if compiled_result == CompilationResults::default() {
            Err(CompileError::CompilationFailed)
        } else {
            Ok(compiled_result)
        }
    }
}
