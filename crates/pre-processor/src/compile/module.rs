//!  Preprocessor is reponsible for identifying the required values.
//!  This will be most abstract layer of the preprocessor.

use alloy::primitives::ChainId;
use core::panic;
use hdp_cairo_runner::dry_run::DryRunResult;
use hdp_cairo_runner::{cairo_dry_run, input::dry_run::DryRunnerProgramInput};
use hdp_primitives::constant::DRY_CAIRO_RUN_OUTPUT_FILE;
use hdp_primitives::processed_types::cairo_format;
use hdp_primitives::task::ExtendedModule;
use hdp_provider::{evm::provider::EvmProvider, key::FetchKeyEnvelope};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

use super::config::CompilerConfig;
use super::{Compilable, CompilationResults, CompileError};

pub type ModuleVec = Vec<ExtendedModule>;

impl Compilable for ModuleVec {
    async fn compile(
        &self,
        compile_config: &CompilerConfig,
    ) -> Result<CompilationResults, CompileError> {
        info!("target task: {:#?}", self[0].task);
        let dry_run_program_path = compile_config.dry_run_program_path.clone();
        let tasks_commitments = self
            .iter()
            .map(|module| module.task.commit())
            .collect::<Vec<_>>();

        let input = generate_input(self.to_vec(), PathBuf::from(DRY_CAIRO_RUN_OUTPUT_FILE)).await?;
        let input_string =
            serde_json::to_string_pretty(&input).expect("Failed to serialize module class");

        // 2. run the dry run and get the fetch points
        info!("2. Running dry-run... ");
        let keys: DryRunResult = cairo_dry_run(dry_run_program_path, input_string)?;

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
            tasks_commitments[0],
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
        let provider = EvmProvider::new(compile_config.provider_config.clone());
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
            results.mmr_metas.into_iter().collect(),
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
