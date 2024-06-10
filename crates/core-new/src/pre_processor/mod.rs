//!  Preprocessor is reponsible for identifying the required values.
//!  This will be most abstract layer of the preprocessor.

use crate::cairo_runner::pre_run::PreRunner;
use crate::module::Module;
use crate::pre_processor::input::PreProcessorInput;
use anyhow::Result;
use hdp_provider::key::FetchKey;
use std::vec;

pub mod input;

pub struct PreProcessResult<T: FetchKey> {
    /// Fetch points are the values that are required to run the module
    pub fetch_keys: Vec<T>,
    /// Module hash is the hash of the module that is being processed
    module: Module,
}

pub struct PreProcessor<T> {
    pre_runner: PreRunner<T>,
}

impl<T: FetchKey> Default for PreProcessor<T>
where
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T: FetchKey> PreProcessor<T>
where
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    pub fn new() -> Self {
        let pre_runner = PreRunner::new();
        Self { pre_runner }
    }

    /// User request is pass as input of this function,
    /// First it will generate input structure for preprocessor that need to pass to runner
    /// Then it will run the preprocessor and return the result, fetch points
    /// Fetch points are the values that are required to run the module
    pub fn process(&self, module: Module) -> Result<PreProcessResult<T>> {
        // 1. generate input data required for preprocessor
        let input = self.generate_input(module);
        let input_bytes = input.to_bytes();
        // 2. run the preprocessor and get the fetch points
        let points = self.pre_runner.run(input_bytes.to_vec())?;
        Ok(PreProcessResult {
            fetch_keys: points,
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
