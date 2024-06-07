use std::vec;

use crate::cairo_runner::pre_run::PreRunner;
use crate::module::Module;
use crate::pre_processor::input::PreProcessorInput;
use anyhow::Result;

pub mod input;

pub struct PreProcessResult {
    /// Fetch points are the values that are required to run the module
    // TODO: fetch point design should sync with memoizer key-lookup design
    pub fetch_points: Vec<String>,
    /// Module hash is the hash of the module that is being processed
    module: Module,
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
    pub fn process(&self, module: Module) -> Result<PreProcessResult> {
        // 1. generate input data required for preprocessor
        let input = self.generate_input(module);
        let input_bytes = input.to_bytes();
        // 2. run the preprocessor and get the fetch points
        let points = self.pre_runner.run(input_bytes.to_vec())?;
        Ok(PreProcessResult {
            fetch_points: points,
            module: input.get_module(),
        })
    }

    /// Generate input structure for preprocessor that need to pass to runner
    pub fn generate_input(&self, module: Module) -> PreProcessorInput {
        // TODO: get module bytes from registry by module_hash
        let module_hash = module.get_module_hash();
        let module_input = module.get_module_input();
        let module_bytes = vec![];

        // TODO: generate input data and make it ready to seialize as bytes
        PreProcessorInput::new(module, module_bytes)
    }
}
