//!  Preprocessor is reponsible for identifying the required values.
//!  This will be most abstract layer of the preprocessor.

use std::fs;

use crate::cairo_runner::pre_run::PreRunner;
use crate::module::Module;
use crate::module_registry::ModuleRegistry;
use crate::pre_processor::input::PreProcessorInput;
use anyhow::Result;
use hdp_provider::key::FetchKeyEnvelope;
use starknet::providers::Url;

pub mod input;

pub struct PreProcessResult {
    /// Fetch points are the values that are required to run the module
    pub fetch_keys: Vec<FetchKeyEnvelope>,
    /// Module hash is the hash of the module that is being processed
    module: Module,
}

pub struct PreProcessor {
    pre_runner: PreRunner,
    /// Registery provider
    module_registry: ModuleRegistry,
}

impl PreProcessor {
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
    pub async fn process(&self, module: Module) -> Result<PreProcessResult> {
        // 1. generate input data required for preprocessor
        let input = self.generate_input(module).await?;
        let input_string =
            serde_json::to_string_pretty(&input).expect("Failed to serialize module_casm");

        // //save into file
        // fs::write("input.json", input_string.clone()).expect("Unable to write file");
        // 2. run the preprocessor and get the fetch points
        let keys = self.pre_runner.run(input_string)?;
        Ok(PreProcessResult {
            fetch_keys: keys,
            module: input.get_module(),
        })
    }

    /// Generate input structure for preprocessor that need to pass to runner
    pub async fn generate_input(&self, module: Module) -> Result<PreProcessorInput> {
        let class_hash = module.get_class_hash();
        let module_casm = self.module_registry.get_module(class_hash).await?;

        // TODO: generate input data and make it ready to seialize as bytes
        Ok(PreProcessorInput::new(module, module_casm))
    }
}
