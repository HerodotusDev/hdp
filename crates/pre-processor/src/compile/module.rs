//!  THIS IS WIP, NOT READY FOR USE
//!  Preprocessor is reponsible for identifying the required values.
//!  This will be most abstract layer of the preprocessor.

#![allow(dead_code)]

use alloy::primitives::ChainId;
use core::panic;
use hdp_cairo_runner::dry_run::DryRunResult;
use hdp_cairo_runner::{cairo_dry_run, input::dry_run::DryRunnerProgramInput};
use hdp_primitives::constant::DRY_RUN_OUTPUT_FILE;
use hdp_primitives::processed_types::cairo_format;
use hdp_primitives::task::ExtendedModule;
use hdp_provider::{evm::provider::EvmProvider, key::FetchKeyEnvelope};
use starknet::providers::Url;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

use super::{Compilable, CompilationResults, CompileConfig, CompileError};

pub type ModuleVec = Vec<ExtendedModule>;

pub struct ModuleCompilerConfig {
    // rpc url to fetch the module class from starknet
    pub module_registry_rpc_url: Url,
    // pre-run program path
    pub program_path: PathBuf,
}

impl Compilable for ModuleVec {
    async fn compile(
        &self,
        compile_config: &CompileConfig,
    ) -> Result<CompilationResults, CompileError> {
        info!("target task: {:#?}", self[0].task);
        let program_path = compile_config.module.program_path.clone();
        let task_commitments = self
            .iter()
            .map(|module| module.task.commit())
            .collect::<Vec<_>>();

        let input = generate_input(self.to_vec(), PathBuf::from(DRY_RUN_OUTPUT_FILE)).await?;
        let input_string =
            serde_json::to_string_pretty(&input).expect("Failed to serialize module class");

        // 2. run the dry run and get the fetch points
        info!("2. Running dry-run... ");
        let keys: DryRunResult = cairo_dry_run(program_path, input_string)?;

        if keys[0].class_hash != self[0].task.class_hash {
            return Err(CompileError::ClassHashMismatch);
        }

        if keys.len() != 1 {
            // TODO: temporary solution. Need to handle multiple module in future
            panic!("Multiple Modules are not supported yet");
        }
        let dry_runned_module = keys.into_iter().next().unwrap();
        let mut commit_results_maps = HashMap::new();
        commit_results_maps.insert(
            task_commitments[0],
            dry_runned_module.result.to_combined_string().into(),
        );

        // 3. call provider using keys
        let keys_maps_chain = categrize_fetch_keys_by_chain_id(dry_runned_module.fetch_keys);
        if keys_maps_chain.len() > 1 {
            // TODO: This is temporary solution. Need to handle multiple chain id in future
            panic!("Multiple chain id is not supported yet");
        }

        let (_, keys) = keys_maps_chain.into_iter().next().unwrap();
        // TODO: later we can get chain id from the key. For now we just ignore as this not compatible with cairo
        // TODO: should spawn multiple provider base on batch of chain id. Probably need to change config around chain id and rpc url
        // This config cannot handle the situation when calling multiple chain data in one module
        // But as this have not used, for now we can just follow batch's chain id
        info!("3. Fetching proofs from provider...");
        let provider = EvmProvider::new(compile_config.provider.clone());
        let results = provider
            .fetch_proofs_from_keys(keys.into_iter().collect())
            .await?;

        Ok(CompilationResults::new(
            true,
            commit_results_maps,
            results.headers.into_iter().collect(),
            results.accounts.into_iter().collect(),
            results.storages.into_iter().collect(),
            results.transactions.into_iter().collect(),
            results.transaction_receipts.into_iter().collect(),
            results.mmr_meta,
        ))
    }
}

/// Categorize fetch keys by chain id
/// This is require to initiate multiple provider for different chain id
fn categrize_fetch_keys_by_chain_id(
    fetch_keys: Vec<FetchKeyEnvelope>,
) -> Vec<(ChainId, Vec<FetchKeyEnvelope>)> {
    let mut chain_id_map = std::collections::HashMap::new();
    for key in fetch_keys {
        let chain_id = key.get_chain_id();
        let keys = chain_id_map.entry(chain_id).or_insert_with(Vec::new);
        keys.push(key);
    }
    chain_id_map.into_iter().collect()
}

/// Generate input structure for preprocessor that need to pass to runner
async fn generate_input(
    extended_modules: Vec<ExtendedModule>,
    identified_keys_file: PathBuf,
) -> Result<DryRunnerProgramInput, CompileError> {
    // Collect results, filter out any errors
    let mut collected_results = Vec::new();
    for module in extended_modules {
        let input_module =
            cairo_format::ProcessedModule::new(module.task.inputs, module.module_class);
        collected_results.push(input_module);
    }

    Ok(DryRunnerProgramInput::new(
        identified_keys_file,
        collected_results,
    ))
}

#[cfg(test)]
mod tests {
    // use hdp_primitives::task::module::{Module, ModuleTag};
    // use hdp_provider::evm::provider::EvmProviderConfig;
    // use starknet::macros::felt;

    // use crate::module_registry::ModuleRegistry;

    // use super::*;
    const SEPOLIA_RPC_URL: &str =
        "https://eth-sepolia.g.alchemy.com/v2/xar76cftwEtqTBWdF4ZFy9n8FLHAETDv";
    const SN_SEPOLIA_RPC_URL: &str =
        "https://starknet-sepolia.g.alchemy.com/v2/lINonYKIlp4NH9ZI6wvqJ4HeZj7T4Wm6";

    #[ignore = "ignore for now"]
    #[tokio::test]
    async fn test_compile_module() {
        // let program_path = "../../build/compiled_cairo/contract_dry_run.json";

        // let module = Module::from_tag(
        //     ModuleTag::AccountBalanceExample,
        //     vec![felt!("1"), felt!("0")],
        // );

        // let module_config = ModuleCompilerConfig {
        //     module_registry_rpc_url: Url::parse(SN_SEPOLIA_RPC_URL).unwrap(),
        //     program_path: PathBuf::from(program_path),
        // };

        // let provider_config = EvmProviderConfig {
        //     rpc_url: Url::parse(SEPOLIA_RPC_URL).unwrap(),
        //     chain_id: 11155111,
        //     max_requests: 100,
        // };

        // let module_regisry = ModuleRegistry::new(Url::parse(SN_SEPOLIA_RPC_URL).unwrap());
        // let module_class = module_regisry
        //     .get_module_class(module.tag().to_class_hash())
        //     .await
        //     .unwrap();
        // let compiled_result: CompilationResults = vec![module.clone()]
        //     .compile(&CompileConfig {
        //         provider: provider_config,
        //         module: module_config,
        //     })
        //     .await
        //     .unwrap();
        // assert_eq!(compiled_result.headers.len(), 10);
        // assert_eq!(compiled_result.accounts.len(), 1);
        // let account = compiled_result.accounts.iter().next().unwrap();
        // assert_eq!(account.proofs.len(), 10);
    }
}
