use anyhow::Result;
use hdp_provider::key::FetchKeyEnvelope;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;
use tracing::info;

use anyhow::bail;
use regex::Regex;

pub struct PreRunner {
    program_path: PathBuf,
}

impl PreRunner {
    pub fn new(program_path: PathBuf) -> Self {
        Self { program_path }
    }

    fn _run(&self, input_file_path: &Path) -> Result<String> {
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

    /// Pre run to return requested values
    pub fn run(&self, input_string: String) -> Result<Vec<FetchKeyEnvelope>> {
        if input_string.is_empty() {
            bail!("Input file is empty");
        }
        let input_file = NamedTempFile::new()?;
        let input_file_path = input_file.path();
        fs::write(input_file_path, input_string).expect("Failed to write input file");
        info!("Running pre-runner on cairo-vm...");
        let output = self._run(input_file_path)?;

        // parse output to return dry run result
        let dry_run_result = self.parse_run(output)?;
        info!("Pre-runner executed successfully");
        Ok(dry_run_result)
    }

    /// Parse the output of the dry run command
    ///!wip
    fn parse_run(&self, output: String) -> Result<Vec<FetchKeyEnvelope>> {
        let task_result_re = Regex::new(r"Task Result\((\d+)\): (\S+)").unwrap();
        let mut task_results = vec![];
        for caps in task_result_re.captures_iter(&output) {
            let _ = &caps[1];
            let value: FetchKeyEnvelope = caps[2]
                .parse()
                .expect("Failed to parse Fetch Key from output");
            // from_str is implemented for FetchKey
            task_results.push(value);
        }
        Ok(task_results)
    }
}
