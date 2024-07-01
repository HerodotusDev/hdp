//! The input for the dry-runner.
//! This serialized struct will be passed to the dry-runner(cairo-run) as input.json file.

use hdp_primitives::processed_types::module::ProcessedModule;
use serde::Serialize;
use serde_with::serde_as;
use std::path::PathBuf;

#[serde_as]
#[derive(Serialize)]
pub struct DryRunnerProgramInput {
    pub identified_keys_file: PathBuf,
    pub modules: Vec<ProcessedModule>,
}

impl DryRunnerProgramInput {
    pub fn new(identified_keys_file: PathBuf, modules: Vec<ProcessedModule>) -> Self {
        Self {
            identified_keys_file,
            modules,
        }
    }
}
