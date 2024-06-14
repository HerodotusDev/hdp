//!  Preprocessor is reponsible for identifying the required values.
//!  This will be most abstract layer of the preprocessor.

use std::collections::HashSet;
use std::path::PathBuf;

use crate::compiler::module::ModuleCompilerConfig;
use crate::compiler::Compiler;

use anyhow::{Ok, Result};
use hdp_primitives::task::{ExtendedTask, TaskEnvelope};
use hdp_provider::key::FetchKeyEnvelope;

use starknet::providers::Url;
use tracing::info;

pub struct PreProcessor {
    /// compiler
    compiler: Compiler,
}

pub struct PreProcessorConfig {
    // rpc url to fetch the module class from starknet
    pub module_registry_rpc_url: Url,
    // pre-run program path
    pub program_path: PathBuf,
}

pub struct PreProcessResult {
    /// Fetch points are the values that are required to run the module
    pub fetch_keys: HashSet<FetchKeyEnvelope>,
    /// Tasks that are extended with relevant information for processor
    pub tasks: Vec<ExtendedTask>,
}

impl PreProcessor {
    pub fn new_with_config(config: PreProcessorConfig) -> Self {
        let rpc_url = config.module_registry_rpc_url;
        let program_path = config.program_path;
        let compiler = Compiler::new(ModuleCompilerConfig {
            module_registry_rpc_url: rpc_url,
            program_path: program_path.clone(),
        });
        Self { compiler }
    }

    /// User request is pass as input of this function,
    /// First it will generate input structure for preprocessor that need to pass to runner
    /// Then it will run the preprocessor and return the result, fetch points
    /// Fetch points are the values that are required to run the module
    pub async fn process(&self, tasks: Vec<TaskEnvelope>) -> Result<PreProcessResult> {
        let (fetch_keys, extended_tasks) = self.compiler.compile(tasks).await?;
        info!("Preprocessor completed successfully");
        Ok(PreProcessResult {
            fetch_keys,
            tasks: extended_tasks,
        })
    }
}
