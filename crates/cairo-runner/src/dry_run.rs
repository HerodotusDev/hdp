//!  THIS IS WIP, NOT READY FOR USE

use hdp_primitives::constant::DRY_RUN_OUTPUT_FILE;
use hdp_primitives::processed_types::uint256::Uint256;
use hdp_provider::key::FetchKeyEnvelope;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;
use tracing::info;

use crate::CairoRunnerError;

pub type DryRunResult = Vec<DryRunnedModule>;

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct DryRunnedModule {
    pub fetch_keys: Vec<FetchKeyEnvelope>,
    pub result: Uint256,
    #[serde_as(as = "UfeHex")]
    pub class_hash: FieldElement,
}

pub struct DryRunner {
    program_path: PathBuf,
}

impl DryRunner {
    pub fn new(program_path: PathBuf) -> Self {
        Self { program_path }
    }

    fn _run(&self, input_file_path: &Path) -> Result<String, CairoRunnerError> {
        let task = Command::new("cairo-run")
            .arg("--program")
            .arg(&self.program_path)
            .arg("--layout")
            .arg("starknet_with_keccak")
            .arg("--program_input")
            .arg(input_file_path)
            .arg("--print_output")
            .stdout(Stdio::piped())
            .spawn()?;

        let output = task.wait_with_output().expect("Failed to read stdout");
        let output_str = String::from_utf8_lossy(&output.stdout);
        Ok(output_str.to_string())
    }

    /// Dry run to return requested values
    pub fn run(&self, input_string: String) -> Result<DryRunResult, CairoRunnerError> {
        if input_string.is_empty() {
            return Err(CairoRunnerError::EmptyInput);
        }
        let input_file = NamedTempFile::new()?;
        let input_file_path = input_file.path();
        fs::write(input_file_path, input_string).expect("Failed to write input file");
        let _ = self._run(input_file_path)?;

        // parse output to return dry run result
        let dry_run_result = self.parse_run(&PathBuf::from(DRY_RUN_OUTPUT_FILE))?;
        info!("Dry-runner executed successfully");
        Ok(dry_run_result)
    }

    /// Parse the output of the dry run command
    fn parse_run(&self, input_file_path: &Path) -> Result<DryRunResult, CairoRunnerError> {
        let output = fs::read_to_string(input_file_path)?;
        let fetch_keys: Vec<DryRunnedModule> = serde_json::from_str(&output)?;
        fs::remove_file(input_file_path)?;
        Ok(fetch_keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_dry_runner() -> DryRunner {
        let program_path = PathBuf::from("tests/programs/cairo_runner_test.cairo");
        DryRunner::new(program_path)
    }

    #[test]
    fn test_parse_run() {
        let dry_runner = init_dry_runner();

        let path: &Path = Path::new("./fixtures/dry_run_output.json");

        let fetch_keys = dry_runner.parse_run(path).unwrap();
        assert_eq!(fetch_keys.len(), 10);
    }
}
