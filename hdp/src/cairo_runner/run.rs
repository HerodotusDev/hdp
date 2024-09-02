use crate::constant::SOUND_CAIRO_RUN_OUTPUT_FILE;
use alloy::primitives::{B256, U256};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;
use tracing::info;

use crate::cairo_runner::CairoRunnerError;

/// Result of run
#[derive(Debug)]
pub struct RunResult {
    pub pie_path: PathBuf,
    pub cairo_run_output: CairoRunOutput,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CairoRunOutput {
    pub tasks_root: B256,
    pub results_root: B256,
    pub results: Vec<U256>,
}

pub struct Runner {
    program_path: PathBuf,
}

impl Runner {
    pub fn new(program_path: &Path) -> Self {
        Self {
            program_path: program_path.to_path_buf(),
        }
    }

    fn _run(
        &self,
        input_file_path: &Path,
        cairo_pie_file_path: &Path,
    ) -> Result<String, CairoRunnerError> {
        let task = Command::new("cairo-run")
            .arg("--program")
            .arg(&self.program_path)
            .arg("--layout")
            .arg("starknet_with_keccak")
            .arg("--program_input")
            .arg(input_file_path)
            .arg("--cairo_pie_output")
            .arg(cairo_pie_file_path)
            .arg("--print_output")
            .arg("--print_info")
            .stdout(Stdio::piped())
            .spawn()?;

        let output = task.wait_with_output().expect("Failed to read stdout");
        let output_str = String::from_utf8_lossy(&output.stdout);
        Ok(output_str.to_string())
    }

    /// Run the cairo program to return PIE object and results of process
    pub fn run(
        &self,
        input_string: String,
        pie_file_path: &PathBuf,
    ) -> Result<RunResult, CairoRunnerError> {
        if input_string.is_empty() {
            return Err(CairoRunnerError::EmptyInput);
        }

        let input_file = NamedTempFile::new()?;
        let input_file_path = input_file.path();
        fs::write(input_file_path, input_string).expect("Failed to write input file");

        let output = self._run(input_file_path, pie_file_path)?;
        let cairo_run_output =
            self.parse_run(output, &PathBuf::from(SOUND_CAIRO_RUN_OUTPUT_FILE))?;
        info!("cairo run output: {:#?}", cairo_run_output);

        fs::remove_file(SOUND_CAIRO_RUN_OUTPUT_FILE)
            .expect("Failed to remove cairo run output file");

        Ok(RunResult {
            pie_path: pie_file_path.to_owned(),
            cairo_run_output,
        })
    }

    /// Parse the output of the run command
    fn parse_run(
        &self,
        output: String,
        cairo_run_output_path: &PathBuf,
    ) -> Result<CairoRunOutput, CairoRunnerError> {
        let number_of_steps = Regex::new(r"Number of steps: (\d+)").unwrap();
        if let Some(number_of_steps_caps) = number_of_steps.captures(&output) {
            let number_of_steps = number_of_steps_caps[1].parse::<usize>()?;
            info!("number of steps: {:#?}", number_of_steps);
            let cairo_run_output_from_file = fs::read_to_string(cairo_run_output_path)
                .expect("Failed to read cairo run output file");
            let cairo_run_output: CairoRunOutput =
                serde_json::from_str(&cairo_run_output_from_file)
                    .expect("Failed to parse cairo run output");
            Ok(cairo_run_output)
        } else {
            Err(CairoRunnerError::CairoRunError)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cairo_run_output() {
        let cairo_run_output_str = r#"{"tasks_root": "0x25bdd48e6c00a86eef6c08afb935635652d246bdf07f54e3ef7c81c29e763fe2", "results_root": "0xbe7bb3e8d053273c753c752107b0b528a24a03058ae989c1e0a9d920c96da753", "results": ["0x0000000000000000000000000000000000000000000000103557b1b802c24c17"]}"#;
        let cairo_run_output: CairoRunOutput =
            serde_json::from_str(cairo_run_output_str).expect("Failed to parse cairo run output");
        println!("Cairo run output: {:#?}", cairo_run_output);
    }
}
