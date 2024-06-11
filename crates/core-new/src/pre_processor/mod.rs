//!  Preprocessor is reponsible for identifying the required values.
//!  This will be most abstract layer of the preprocessor.

use std::path::PathBuf;

use crate::cairo_runner::pre_run::PreRunner;
use crate::module::Module;
use crate::module_registry::ModuleRegistry;
use crate::pre_processor::input::PreProcessorInput;
use anyhow::Result;
use hdp_provider::key::FetchKeyEnvelope;
use starknet::providers::Url;

pub mod input;

pub struct PreProcessor {
    pre_runner: PreRunner,
    /// Registery provider
    module_registry: ModuleRegistry,
}

pub struct PreProcessorConfig {
    // rpc url to fetch the module class from starknet
    pub module_registry_rpc_url: Url,
    // pre-run program path
    pub program_path: PathBuf,
}

pub struct PreProcessResult {
    /// Fetch points are the values that are required to run the module
    pub fetch_keys: Vec<FetchKeyEnvelope>,
    /// Module hash is the hash of the module that is being processed
    pub module: Module,
}

impl PreProcessor {
    pub fn new_with_config(config: PreProcessorConfig) -> Self {
        let rpc_url = config.module_registry_rpc_url;
        let program_path = config.program_path;
        let module_registry = ModuleRegistry::new(rpc_url);
        let pre_runner = PreRunner::new(program_path);
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
            serde_json::to_string_pretty(&input).expect("Failed to serialize module class");

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
        let module_class = self.module_registry.get_module_class(class_hash).await?;
        Ok(PreProcessorInput::new(module, module_class))
    }
}

#[cfg(test)]
mod tests {
    use starknet::macros::felt;

    use super::*;
    use crate::module::{Module, ModuleTag};

    #[tokio::test]
    async fn test_pre_processor() {
        let url: &str =
            "https://starknet-sepolia.g.alchemy.com/v2/lINonYKIlp4NH9ZI6wvqJ4HeZj7T4Wm6";
        let program_path = "../../build/compiled_cairo/hdp.json";
        let pre_processor = PreProcessor::new_with_config(PreProcessorConfig {
            module_registry_rpc_url: url.parse().unwrap(),
            program_path: PathBuf::from(program_path),
        });
        let module = Module::from_tag(ModuleTag::TEST, vec![felt!("1"), felt!("2")]);
        let res = pre_processor.process(module.clone()).await.unwrap();
        assert_eq!(module, res.module)
        // TODO: check fetch point
    }
}
