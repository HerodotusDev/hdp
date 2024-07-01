//! The input for the dry-runner.
//! This serialized struct will be passed to the dry-runner(cairo-run) as input.json file.

use hdp_primitives::processed_types::module::ProcessedModule;
use serde::Serialize;
use serde_with::serde_as;
use std::path::PathBuf;

#[serde_as]
#[derive(Serialize)]
pub struct DryRunnerProgramInput {
    pub fetch_keys_file_path: PathBuf,
    pub module: ProcessedModule,
}

impl DryRunnerProgramInput {
    pub fn new(fetch_keys_file_path: PathBuf, modules: Vec<ProcessedModule>) -> Self {
        // TODO: temporary check to ensure only one module is passed
        if modules.len() != 1 {
            panic!("DryRunnerProgramInput only supports a single module");
        }
        let first_module = modules[0].clone();
        Self {
            fetch_keys_file_path,
            module: first_module,
        }
    }
}
