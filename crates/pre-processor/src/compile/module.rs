//!  THIS IS WIP, NOT READY FOR USE
//!  Preprocessor is reponsible for identifying the required values.
//!  This will be most abstract layer of the preprocessor.

#![allow(dead_code)]

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use crate::{module_registry::ModuleRegistry, ExtendedModule};

use futures::future::join_all;
use hdp_cairo_runner::{dry_run::DryRunner, input::dry_run::DryRunnerInput};
use hdp_primitives::{processed_types::module::ProcessedModule, task::module::Module};
use hdp_provider::{evm::provider::EvmProvider, key::FetchKeyEnvelope};

use starknet::providers::Url;
use tempfile::NamedTempFile;
use tokio::task;
use tracing::info;

use super::{Compilable, CompilationResults, CompileConfig, CompileError};

pub struct ModuleCompilerConfig {
    // rpc url to fetch the module class from starknet
    pub module_registry_rpc_url: Url,
    // pre-run program path
    pub program_path: PathBuf,
}

impl Compilable for Vec<Module> {
    async fn compile(
        &self,
        compile_config: &CompileConfig,
    ) -> Result<CompilationResults, CompileError> {
        let rpc_url = compile_config.module.module_registry_rpc_url.clone();
        let program_path = compile_config.module.program_path.clone();
        let module_registry = ModuleRegistry::new(rpc_url);
        let pre_runner = DryRunner::new(program_path);

        // 1. generate input data required for preprocessor
        info!("Generating input data for preprocessor...");

        // fetch module class
        let extended_modules = fetch_modules_class(module_registry, self.clone()).await?;

        // generate temp file
        let identified_keys_file = NamedTempFile::new().unwrap().path().to_path_buf();
        let input = generate_input(extended_modules.clone(), identified_keys_file).await?;
        let input_string =
            serde_json::to_string_pretty(&input).expect("Failed to serialize module class");

        //save into file
        // fs::write("input.json", input_string.clone()).expect("Unable to write file");
        // 2. run the preprocessor and get the fetch points
        info!("Running preprocessor...");
        info!("Preprocessor completed successfully");
        // hashset from vector
        let keys: HashSet<FetchKeyEnvelope> = pre_runner.run(input_string)?.into_iter().collect();

        // 3. call provider using keys
        let provider = EvmProvider::new(compile_config.provider.clone());
        let results = provider.fetch_proofs_from_keys(keys).await?;
        Ok(CompilationResults::new_without_result(
            results.headers.into_iter().collect(),
            results.accounts.into_iter().collect(),
            results.storages.into_iter().collect(),
            results.transactions.into_iter().collect(),
            results.transaction_receipts.into_iter().collect(),
            results.mmr_meta,
        ))
    }
}

pub async fn fetch_modules_class(
    module_registry: ModuleRegistry,
    modules: Vec<Module>,
) -> Result<Vec<ExtendedModule>, CompileError> {
    let registry = Arc::new(module_registry);
    // Map each module to an asynchronous task
    let module_futures: Vec<_> = modules
        .into_iter()
        .map(|module| {
            let module_registry = Arc::clone(&registry);
            task::spawn(async move {
                let module_hash = module.class_hash;
                match module_registry.get_module_class(module_hash).await {
                    Ok(module_class) => Ok(ExtendedModule {
                        task: module,
                        module_class,
                    }),
                    Err(e) => Err(e),
                }
            })
        })
        .collect();

    // Join all tasks and collect their results
    let results = join_all(module_futures).await;

    // Collect results, filter out any errors
    let mut collected_results = Vec::new();
    for result in results {
        match result {
            Ok(Ok(module_with_class)) => collected_results.push(module_with_class),
            Ok(Err(e)) => return Err(CompileError::AnyhowError(e)),
            Err(e) => return Err(CompileError::AnyhowError(e.into())),
        }
    }

    Ok(collected_results)
}

/// Generate input structure for preprocessor that need to pass to runner
async fn generate_input(
    extended_modules: Vec<ExtendedModule>,
    identified_keys_file: PathBuf,
) -> Result<DryRunnerInput, CompileError> {
    // Collect results, filter out any errors
    let mut collected_results = Vec::new();
    for module in extended_modules {
        let input_module = ProcessedModule::new(module.task.inputs, module.module_class);
        collected_results.push(input_module);
    }

    Ok(DryRunnerInput {
        identified_keys_file,
        modules: collected_results,
    })
}

pub struct PreProcessResult {
    /// Fetch points are the values that are required to run the module
    pub fetch_keys: Vec<FetchKeyEnvelope>,
    /// Module hash is the hash of the module that is being processed
    pub modules: Vec<Module>,
}

#[cfg(test)]
mod tests {
    use hdp_primitives::task::module::ModuleTag;
    use hdp_provider::evm::provider::EvmProviderConfig;
    use starknet::macros::felt;

    use super::*;
    const SEPOLIA_RPC_URL: &str =
        "https://eth-sepolia.g.alchemy.com/v2/xar76cftwEtqTBWdF4ZFy9n8FLHAETDv";

    #[ignore = "ignore for now"]
    #[tokio::test]
    async fn test_pre_processor() {
        let url: &str =
            "https://starknet-sepolia.g.alchemy.com/v2/lINonYKIlp4NH9ZI6wvqJ4HeZj7T4Wm6";
        let program_path = "../../build/compiled_cairo/hdp.json";

        let module = Module::from_tag(ModuleTag::TEST, vec![felt!("1"), felt!("2")]);
        let module2 = Module::from_tag(ModuleTag::TEST, vec![felt!("1"), felt!("2")]);

        let module_config = ModuleCompilerConfig {
            module_registry_rpc_url: Url::parse(url).unwrap(),
            program_path: PathBuf::from(program_path),
        };

        let provider_config = EvmProviderConfig {
            rpc_url: Url::parse(SEPOLIA_RPC_URL).unwrap(),
            chain_id: 11155111,
            max_requests: 100,
        };
        let _ = vec![module.clone(), module2.clone()]
            .compile(&CompileConfig {
                provider: provider_config,
                module: module_config,
            })
            .await
            .unwrap();

        // TODO: check fetch point
    }
}
