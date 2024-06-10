use anyhow::Result;
use hdp_provider::key::FetchKey;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

use anyhow::bail;
use regex::Regex;

const PRE_RUN_CAIRO_PROGRAM: &str = "build/compiled_cairo/hdp.json";

pub struct PreRunner<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: FetchKey> Default for PreRunner<T>
where
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T: FetchKey> PreRunner<T>
where
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    fn _run(&self, input_file_path: &Path) -> Result<String> {
        let task = Command::new("cairo-run")
            .arg("--program")
            .arg(PRE_RUN_CAIRO_PROGRAM)
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
    pub fn run(&self, input_bytes: Vec<u8>) -> Result<Vec<T>> {
        if input_bytes.is_empty() {
            bail!("Input file is empty");
        }
        let mut input_file = NamedTempFile::new()?;
        input_file.write_all(&input_bytes)?;
        let input_file_path = input_file.path();
        let output = self._run(input_file_path)?;

        // parse output to return dry run result
        let dry_run_result = self.parse_run(output)?;
        Ok(dry_run_result)
    }

    /// Parse the output of the dry run command
    fn parse_run(&self, output: String) -> Result<Vec<T>> {
        let task_result_re = Regex::new(r"Task Result\((\d+)\): (\S+)").unwrap();
        let mut task_results = vec![];
        for caps in task_result_re.captures_iter(&output) {
            let _ = &caps[1];
            let value: T = caps[2]
                .parse()
                .expect("Failed to parse Fetch Key from output");
            // from_str is implemented for FetchKey
            task_results.push(value);
        }
        Ok(task_results)
    }
}
