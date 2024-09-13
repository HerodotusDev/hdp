use crate::primitives::task::TaskEnvelope;

use super::{
    config::CompilerConfig, module::compile_modules, Compilable, CompilationResult, CompileError,
};

pub async fn compile_tasks(
    tasks: &[TaskEnvelope],
    compile_config: &CompilerConfig,
) -> Result<CompilationResult, CompileError> {
    let (datalakes, modules) = TaskEnvelope::divide_tasks(tasks.to_vec());
    let mut compiled_result = if !datalakes.is_empty() {
        datalakes.compile(compile_config).await?
    } else {
        CompilationResult::default()
    };

    let module_compile_result = if !modules.is_empty() {
        compile_modules(modules, compile_config).await?
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
