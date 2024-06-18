use anyhow::Result;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

use anyhow::bail;
use hdp_primitives::datalake::output::{combine_parts_into_big_endian_hex, Uint256};
use regex::Regex;

const RUN_CAIRO_PROGRAM: &str = "build/compiled_cairo/hdp.json";

/// Result of run
#[derive(Debug)]
pub struct RunResult {
    pie_path: PathBuf,
    task_results: Vec<String>,
    results_root: String,
}

pub struct Runner {}

impl Default for Runner {
    fn default() -> Self {
        Self::new()
    }
}

impl Runner {
    pub fn new() -> Self {
        Self {}
    }

    fn _run(&self, input_file_path: &Path, cairo_pie_file_path: &Path) -> Result<String> {
        let task = Command::new("cairo-run")
            .arg("--program")
            .arg(RUN_CAIRO_PROGRAM)
            .arg("--layout")
            .arg("starknet_with_keccak")
            .arg("--program_input")
            .arg(input_file_path)
            .arg("--cairo_pie_output")
            .arg(cairo_pie_file_path)
            .arg("--print_output")
            .stdout(Stdio::piped())
            .spawn()?;

        let output = task.wait_with_output().expect("Failed to read stdout");
        let output_str = String::from_utf8_lossy(&output.stdout);
        Ok(output_str.to_string())
    }

    /// Run the cairo program to return PIE object and results of process
    pub fn run(&self, input_bytes: Vec<u8>) -> Result<RunResult> {
        if input_bytes.is_empty() {
            bail!("Input file is empty");
        }
        let mut input_file = NamedTempFile::new()?;
        let pie_file = NamedTempFile::new()?;
        input_file.write_all(&input_bytes)?;
        let input_file_path = input_file.path();
        let pie_file_path = pie_file.path();
        let output = self._run(input_file_path, pie_file_path)?;

        let (task_results, results_root) = self.parse_run(output)?;

        Ok(RunResult {
            pie_path: pie_file_path.to_owned(),
            task_results,
            results_root,
        })
    }

    /// Parse the output of the run command
    fn parse_run(&self, output: String) -> Result<(Vec<String>, String)> {
        let task_result_re = Regex::new(r"Task Result\((\d+)\): (\S+)").unwrap();
        let mut task_results = vec![];
        for caps in task_result_re.captures_iter(&output) {
            let _ = &caps[1];
            let value = &caps[2];
            task_results.push(value.to_string());
        }
        let results_root_re = Regex::new(r"Results Root: (\S+) (\S+)").unwrap();
        if let Some(results_root_caps) = results_root_re.captures(&output) {
            let results_root_1 = &results_root_caps[1];
            let results_root_2 = &results_root_caps[2];
            let combined_results_root = combine_parts_into_big_endian_hex(&Uint256 {
                low: results_root_1.to_string(),
                high: results_root_2.to_string(),
            });
            Ok((task_results, combined_results_root))
        } else {
            bail!("Results Root not found");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_run() {
        let runner = Runner::new();
        let output = "Task Result(0): 0x01020304\nResults Root: 0x01020304 0x05060708";
        let (task_results, results_root) = runner.parse_run(output.to_string()).unwrap();
        assert_eq!(task_results.len(), 1);
        assert_eq!(task_results[0], "0x01020304");
        assert_eq!(
            results_root,
            "0x0000000000000000000000000506070800000000000000000000000001020304"
        );
    }
}
