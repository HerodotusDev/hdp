use alloy::primitives::{B256, U256};
use hdp_primitives::processed_types::uint256::Uint256;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;
use tempfile::NamedTempFile;
use tracing::info;

use crate::CairoRunnerError;

/// Result of run
#[derive(Debug)]
pub struct RunResult {
    pub pie_path: PathBuf,
    pub task_results: Vec<U256>,
    pub results_root: B256,
}

pub struct Runner {
    program_path: PathBuf,
}

impl Runner {
    pub fn new(program_path: PathBuf) -> Self {
        Self { program_path }
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
        pie_file_path: PathBuf,
    ) -> Result<RunResult, CairoRunnerError> {
        if input_string.is_empty() {
            return Err(CairoRunnerError::EmptyInput);
        }

        let input_file = NamedTempFile::new()?;
        let input_file_path = input_file.path();
        fs::write(input_file_path, input_string).expect("Failed to write input file");

        let output = self._run(input_file_path, &pie_file_path)?;
        let (task_results, results_root) = self.parse_run(output)?;
        info!("Final result: {:#?}", task_results);
        info!("Final result root: {:#?}", results_root);

        Ok(RunResult {
            pie_path: pie_file_path.to_owned(),
            task_results,
            results_root,
        })
    }

    /// Parse the output of the run command
    fn parse_run(&self, output: String) -> Result<(Vec<U256>, B256), CairoRunnerError> {
        let number_of_steps = Regex::new(r"Number of steps: (\d+)").unwrap();
        if let Some(number_of_steps_caps) = number_of_steps.captures(&output) {
            let number_of_steps = number_of_steps_caps[1].parse::<usize>()?;
            info!("Number of steps: {:#?}", number_of_steps);
        }
        let task_result_re = Regex::new(r"Task Result\((\d+)\): (\S+)").unwrap();
        let mut task_results = vec![];
        for caps in task_result_re.captures_iter(&output) {
            let _ = &caps[1];
            let value = &caps[2];
            task_results.push(U256::from_str(value)?);
        }
        let results_root_re = Regex::new(r"Results Root: (\S+) (\S+)").unwrap();
        if let Some(results_root_caps) = results_root_re.captures(&output) {
            let results_root_1 = &results_root_caps[1];
            let results_root_2 = &results_root_caps[2];
            let result_root = Uint256::from_strs(results_root_2, results_root_1)?;
            let combined_results_root = result_root.to_combined_string();
            Ok((task_results, combined_results_root))
        } else {
            Err(CairoRunnerError::ResultRootNotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_run() {
        let program_path = PathBuf::from("../build/compiled_cairo/hdp.json");
        let runner = Runner::new(program_path);
        let output = "Task Result(0): 0x01020304\nResults Root: 0x01020304 0x05060708";
        let (task_results, results_root) = runner.parse_run(output.to_string()).unwrap();
        assert_eq!(task_results.len(), 1);
        assert_eq!(
            task_results[0],
            U256::from_str_radix("01020304", 16).unwrap()
        );
        assert_eq!(
            results_root,
            Uint256::from_strs("05060708", "01020304")
                .unwrap()
                .to_combined_string()
        );
    }
}
