//!  Preprocessor is reponsible for identifying the required values.
//!  This will be most abstract layer of the preprocessor.

use crate::cairo_runner::pre_run::PreRunner;
use crate::module::Module;
use crate::module_registry::ModuleRegistry;
use crate::pre_processor::input::PreProcessorInput;
use anyhow::Result;
use hdp_provider::key::FetchKey;
use starknet::providers::Url;

pub mod input;

pub struct PreProcessResult<T: FetchKey> {
    /// Fetch points are the values that are required to run the module
    pub fetch_keys: Vec<T>,
    /// Module hash is the hash of the module that is being processed
    module: Module,
}

pub struct PreProcessor<T> {
    pre_runner: PreRunner<T>,
    /// Registery provider
    module_registry: ModuleRegistry,
}

impl<T: FetchKey> PreProcessor<T>
where
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    pub fn new(url: &str) -> Self {
        let url = Url::parse(url).expect("Invalid url");
        let module_registry = ModuleRegistry::new(url);
        let pre_runner = PreRunner::new();
        Self {
            pre_runner,
            module_registry,
        }
    }

    /// User request is pass as input of this function,
    /// First it will generate input structure for preprocessor that need to pass to runner
    /// Then it will run the preprocessor and return the result, fetch points
    /// Fetch points are the values that are required to run the module
    pub async fn process(&self, module: Module) -> Result<PreProcessResult<T>> {
        // 1. generate input data required for preprocessor
        let input = self.generate_input(module).await?;
        let input_string = serde_json::to_string_pretty(&input.get_module_casm())
            .expect("Failed to serialize module_casm");
        // 2. run the preprocessor and get the fetch points
        let keys = self.pre_runner.run(input_string)?;
        Ok(PreProcessResult {
            fetch_keys: keys,
            module: input.get_module(),
        })
    }

    /// Generate input structure for preprocessor that need to pass to runner
    pub async fn generate_input(&self, module: Module) -> Result<PreProcessorInput> {
        let module_hash = module.get_module_hash();
        let module_casm = self.module_registry.get_module(module_hash).await?;

        // TODO: generate input data and make it ready to seialize as bytes
        Ok(PreProcessorInput::new(module, module_casm))
    }
}
