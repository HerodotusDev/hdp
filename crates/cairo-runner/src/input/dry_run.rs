//! The input for the dry-runner.
//! This serialized struct will be passed to the dry-runner(cairo-run) as input.json file.

use hdp_primitives::processed_types::cairo_format;
use serde::Serialize;
use serde_with::serde_as;
use std::path::PathBuf;

#[serde_as]
#[derive(Serialize)]
pub struct DryRunnerProgramInput {
    pub dry_run_output_path: PathBuf,
    pub modules: Vec<cairo_format::ProcessedModule>,
}

impl DryRunnerProgramInput {
    pub fn new(dry_run_output_path: PathBuf, modules: Vec<cairo_format::ProcessedModule>) -> Self {
        // TODO: temporary check to ensure only one module is passed
        if modules.len() != 1 {
            panic!("Currently DryRunnerProgramInput only supports a single module");
        }
        Self {
            dry_run_output_path,
            modules,
        }
    }
}
