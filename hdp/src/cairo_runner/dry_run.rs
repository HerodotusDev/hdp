use crate::primitives::constant::DRY_CAIRO_RUN_OUTPUT_FILE;
use crate::primitives::processed_types::uint256::Uint256;
use crate::provider::key::FetchKeyEnvelope;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;
use tracing::info;

use crate::cairo_runner::CairoRunnerError;

pub type DryRunResult = Vec<DryRunnedModule>;

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct DryRunnedModule {
    pub fetch_keys: Vec<FetchKeyEnvelope>,
    pub result: Uint256,
    #[serde_as(as = "UfeHex")]
    pub program_hash: FieldElement,
}

pub struct DryRunner {
    program_path: PathBuf,
    output_file_path: Option<PathBuf>,
}

impl DryRunner {
    pub fn new(program_path: PathBuf, output_file_path: Option<PathBuf>) -> Self {
        Self {
            program_path,
            output_file_path,
        }
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

        let input_file_path = &NamedTempFile::new()?.path().to_path_buf();
        fs::write(input_file_path, input_string).expect("Failed to write input file");
        let output = self._run(input_file_path)?;
        if output.is_empty() {
            return Err(CairoRunnerError::CairoRunError);
        }

        // parse output to return dry run result
        let dry_run_result = self.parse_run(&PathBuf::from(DRY_CAIRO_RUN_OUTPUT_FILE))?;
        info!("Dry-runner executed successfully");
        Ok(dry_run_result)
    }

    /// Parse the output of the dry run command
    fn parse_run(&self, input_file_path: &Path) -> Result<DryRunResult, CairoRunnerError> {
        let output = fs::read_to_string(input_file_path)?;
        let fetch_keys: Vec<DryRunnedModule> = serde_json::from_str(&output)?;
        fs::remove_file(input_file_path).expect("Failed to remove input file");
        if let Some(ref output_path) = self.output_file_path {
            fs::write(output_path, &output).expect("Failed to write output file");
        }
        Ok(fetch_keys)
    }
}

#[cfg(test)]
mod tests {
    use starknet::macros::felt;

    use super::*;

    fn init_dry_runner() -> DryRunner {
        let program_path = PathBuf::from("../../build/contract_dry_run.json");
        DryRunner::new(program_path, None)
    }

    #[ignore = "ignore for now"]
    #[test]
    fn test_dry_run() {
        let dry_runner = init_dry_runner();
        let input = fs::read_to_string("./fixtures/dry_run_input.json").unwrap();
        let result = dry_runner.run(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].fetch_keys.len(), 3);
        assert_eq!(result[0].result, Uint256::from_strs("0x0", "0x0").unwrap());
        assert_eq!(
            result[0].program_hash,
            felt!("0x04df21eb479ae4416fbdc00abab6fab43bff0b8083be4d1fd8602c8fbfbd2274")
        );
    }

    #[test]
    fn test_parse_run() {
        let output = r#"
    [
        {
            "fetch_keys": [
            {
                "key": {
                "chain_id": 11155111,
                "block_number": 5186021
                },
                "type": "HeaderMemorizerKey"
            },
             {
                "key": {
                "chain_id": 11155111,
                "block_number": 5186023,
                "address": "0x13CB6AE34A13a0977F4d7101eBc24B87Bb23F0d5"
                },
                "type": "AccountMemorizerKey"
            },
            {
                "key": {
                "chain_id": 11155111,
                "block_number": 5186022,
                "address": "0x13CB6AE34A13a0977F4d7101eBc24B87Bb23F0d5",
                "key": "0x487ea7bf96eb1280f1075498855b55ec61ba7d354b5260e2504ef51140e0df63"
                },
                "type": "StorageMemorizerKey"
            }
            ],
            "result": { "low": "0x0", "high": "0x0" },
            "program_hash": "0xc8580f74b6e6e04d8073602ad0c0d55538b56bf8307fefebb6b65b1bbf2a27"
            }
        ]
        "#;
        let fetch_keys: Vec<DryRunnedModule> = serde_json::from_str(output).unwrap();
        assert_eq!(fetch_keys.len(), 1);
        let module = &fetch_keys[0];
        assert_eq!(module.fetch_keys.len(), 3);
        assert_eq!(module.result, Uint256::from_strs("0x0", "0x0").unwrap());
        assert_eq!(
            module.program_hash,
            felt!("0xc8580f74b6e6e04d8073602ad0c0d55538b56bf8307fefebb6b65b1bbf2a27")
        )
    }
}
