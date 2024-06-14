//!  Preprocessor is reponsible for identifying the required values.
//!  This will be most abstract layer of the preprocessor.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::cairo_runner::input::{pre_run::PreRunnerInput, types::InputModule};
use crate::cairo_runner::pre_run::PreRunner;
use crate::module_registry::ModuleRegistry;

use anyhow::{Ok, Result};
use futures::future::join_all;
use hdp_primitives::{module::Module, task::ExtendedModule};
use hdp_provider::key::FetchKeyEnvelope;

use starknet::providers::Url;
use tempfile::NamedTempFile;
use tokio::task;
use tracing::info;

pub(crate) struct ModuleCompiler {
    pre_runner: PreRunner,
    /// Registery provider
    module_registry: Arc<ModuleRegistry>,
}

pub struct ModuleCompilerConfig {
    // rpc url to fetch the module class from starknet
    pub module_registry_rpc_url: Url,
    // pre-run program path
    pub program_path: PathBuf,
}

pub struct PreProcessResult {
    /// Fetch points are the values that are required to run the module
    pub fetch_keys: Vec<FetchKeyEnvelope>,
    /// Module hash is the hash of the module that is being processed
    pub modules: Vec<Module>,
}

impl ModuleCompiler {
    pub fn new_with_config(config: ModuleCompilerConfig) -> Self {
        let rpc_url = config.module_registry_rpc_url;
        let program_path = config.program_path;
        let module_registry = ModuleRegistry::new(rpc_url);
        let pre_runner = PreRunner::new(program_path);
        Self {
            pre_runner,
            module_registry: Arc::new(module_registry),
        }
    }

    /// User request is pass as input of this function,
    /// First it will generate input structure for preprocessor that need to pass to runner
    /// Then it will run the preprocessor and return the result, fetch points
    /// Fetch points are the values that are required to run the module
    pub async fn compile(
        &self,
        modules: Vec<Module>,
    ) -> Result<(HashSet<FetchKeyEnvelope>, Vec<ExtendedModule>)> {
        // 1. generate input data required for preprocessor
        info!("Generating input data for preprocessor...");

        // fetch module class
        let extended_modules = self.fetch_modules_class(modules.clone()).await?;

        // generate temp file
        let identified_keys_file = NamedTempFile::new().unwrap().path().to_path_buf();
        let input = self
            .generate_input(extended_modules.clone(), identified_keys_file)
            .await?;
        let input_string =
            serde_json::to_string_pretty(&input).expect("Failed to serialize module class");

        // //save into file
        fs::write("input.json", input_string.clone()).expect("Unable to write file");
        // 2. run the preprocessor and get the fetch points
        info!("Running preprocessor...");
        info!("Preprocessor completed successfully");
        // hashset from vector
        let keys: HashSet<FetchKeyEnvelope> =
            self.pre_runner.run(input_string)?.into_iter().collect();
        Ok((keys, extended_modules))
    }

    async fn fetch_modules_class(&self, modules: Vec<Module>) -> Result<Vec<ExtendedModule>> {
        let registry: Arc<ModuleRegistry> = Arc::clone(&self.module_registry);
        // Map each module to an asynchronous task
        let module_futures: Vec<_> = modules
            .into_iter()
            .map(|module| {
                let module_registry = Arc::clone(&registry);
                task::spawn(async move {
                    // create input_module
                    let module_hash = module.class_hash;
                    let module_class = module_registry.get_module_class(module_hash).await.unwrap();
                    Ok(ExtendedModule {
                        task: module,
                        module_class,
                    })
                })
            })
            .collect();

        // Join all tasks and collect their results
        let results: Vec<_> = join_all(module_futures).await;

        // Collect results, filter out any errors
        let mut collected_results = Vec::new();
        for result in results {
            let module_with_class = result??;
            collected_results.push(module_with_class);
        }

        Ok(collected_results)
    }

    /// Generate input structure for preprocessor that need to pass to runner
    async fn generate_input(
        &self,
        extended_modules: Vec<ExtendedModule>,
        identified_keys_file: PathBuf,
    ) -> Result<PreRunnerInput> {
        // Collect results, filter out any errors
        let mut collected_results = Vec::new();
        for module in extended_modules {
            let input_module = InputModule {
                inputs: module.task.inputs,
                module_class: module.module_class,
            };
            collected_results.push(input_module);
        }

        Ok(PreRunnerInput {
            identified_keys_file,
            modules: collected_results,
        })
    }
}

#[cfg(test)]
mod tests {
    use hdp_primitives::module::ModuleTag;
    use starknet::macros::felt;

    use super::*;

    #[tokio::test]
    async fn test_pre_processor() {
        let url: &str =
            "https://starknet-sepolia.g.alchemy.com/v2/lINonYKIlp4NH9ZI6wvqJ4HeZj7T4Wm6";
        let program_path = "../../build/compiled_cairo/hdp.json";
        let pre_processor = ModuleCompiler::new_with_config(ModuleCompilerConfig {
            module_registry_rpc_url: url.parse().unwrap(),
            program_path: PathBuf::from(program_path),
        });
        let module = Module::from_tag(ModuleTag::TEST, vec![felt!("1"), felt!("2")]);
        let module2 = Module::from_tag(ModuleTag::TEST, vec![felt!("1"), felt!("2")]);
        let _ = pre_processor
            .compile(vec![module.clone(), module2.clone()])
            .await
            .unwrap();

        // TODO: check fetch point
    }
}
