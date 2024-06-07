use std::vec;

use crate::{cairo_runner::pre_run::PreRunner, input_generator::r#type::PreProcessorInput};
use anyhow::Result;

pub struct PreProcessResult {
    /// Fetch points are the values that are required to run the module
    pub fetch_points: Vec<String>,
    /// Module hash is the hash of the module that is being processed
    pub module_hash: String,
}

/*
  Preprocessor is reponsible for identifying the required values.
  This will be most abstract layer of the preprocessor.
*/
pub struct PreProcessor {
    pre_runner: PreRunner,
}

impl Default for PreProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl PreProcessor {
    pub fn new() -> Self {
        let pre_runner = PreRunner::new();
        Self { pre_runner }
    }

    /// User request is pass as input of this function,
    /// First it will generate input structure for preprocessor that need to pass to runner
    /// Then it will run the preprocessor and return the result, fetch points
    /// Fetch points are the values that are required to run the module
    pub fn process(&self, module_hash: String, module_input: Vec<u8>) -> Result<PreProcessResult> {
        let input = self.generate_input(module_hash, module_input);
        let input_bytes = input.to_bytes();
        let points = self.pre_runner.run(input_bytes.to_vec())?;
        Ok(PreProcessResult {
            fetch_points: points,
            module_hash: input.module_hash.to_string(),
        })
    }

    /// Generate input structure for preprocessor that need to pass to runner
    pub fn generate_input(&self, module_hash: String, module_input: Vec<u8>) -> PreProcessorInput {
        // TODO: get module bytes from registry by module_hash
        let module_bytes = vec![];
        PreProcessorInput::new(module_hash, module_bytes, module_input)
    }
}
