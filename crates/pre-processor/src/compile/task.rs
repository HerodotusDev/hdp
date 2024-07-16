use hdp_primitives::task::TaskEnvelope;

use super::{config::CompilerConfig, Compilable, CompilationResult, CompileError};

impl Compilable for Vec<TaskEnvelope> {
    async fn compile(
        &self,
        compile_config: &CompilerConfig,
    ) -> Result<CompilationResult, CompileError> {
        let (datalakes, modules) = TaskEnvelope::divide_tasks(self.to_vec());
        let mut compiled_result = if !datalakes.is_empty() {
            datalakes.compile(compile_config).await?
        } else {
            CompilationResult::default()
        };

        let module_compile_result = if !modules.is_empty() {
            modules.compile(compile_config).await?
        } else {
            CompilationResult::default()
        };
        compiled_result.extend(module_compile_result);
        if compiled_result == CompilationResult::default() {
            Err(CompileError::CompilationFailed)
        } else {
            Ok(compiled_result)
        }
    }
}
