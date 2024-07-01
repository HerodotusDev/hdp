//!  THIS IS WIP, NOT READY FOR USE

use hdp_provider::key::FetchKeyEnvelope;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;
use tracing::info;

use crate::CairoRunnerError;

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
    pub fn run(&self, input_string: String) -> Result<Vec<FetchKeyEnvelope>, CairoRunnerError> {
        if input_string.is_empty() {
            return Err(CairoRunnerError::EmptyInput);
        }
        let input_file = NamedTempFile::new()?;
        let input_file_path = input_file.path();
        fs::write(input_file_path, input_string).expect("Failed to write input file");
        info!("Running dry-runner on cairo-vm...");
        let _ = self._run(input_file_path)?;

        // parse output to return dry run result
        let dry_run_result = self.parse_run(input_file_path)?;
        info!("Dry-runner executed successfully");
        Ok(dry_run_result)
    }

    /// Parse the output of the dry run command
    fn parse_run(&self, input_file_path: &Path) -> Result<Vec<FetchKeyEnvelope>, CairoRunnerError> {
        let fetch_keys: Vec<FetchKeyEnvelope> =
            serde_json::from_str(&fs::read_to_string(input_file_path)?)?;
        Ok(fetch_keys)
    }
}
