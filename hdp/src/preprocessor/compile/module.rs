//!  Preprocessor is reponsible for identifying the required values.
//!  This will be most abstract layer of the preprocessor.

use crate::cairo_runner::dry_run::DryRunResult;
use crate::cairo_runner::{cairo_dry_run, input::dry_run::DryRunnerProgramInput};
use crate::constant::DRY_CAIRO_RUN_OUTPUT_FILE;
use crate::primitives::processed_types::cairo_format;
use crate::primitives::task::ExtendedModule;
use crate::provider::key::categorize_fetch_keys;
use crate::provider::traits::new_provider_from_config;
use core::panic;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tracing::info;

use super::config::CompilerConfig;
use super::{Compilable, CompilationResult, CompileError};

pub type ModuleVec = Vec<ExtendedModule>;

impl Compilable for ModuleVec {
    async fn compile(
        &self,
        compile_config: &CompilerConfig,
    ) -> Result<CompilationResult, CompileError> {
        // Log the target task for debugging purposes
        info!("target task: {:#?}", self[0].task);
        let dry_run_program_path = compile_config.dry_run_program_path.clone();

        // Generate input for the dry run based on the extended modules
        let dry_run_input =
            generate_input(self.to_vec(), PathBuf::from(DRY_CAIRO_RUN_OUTPUT_FILE)).await?;
        let input_string =
            serde_json::to_string_pretty(&dry_run_input).expect("Failed to serialize module class");

        // 2. Run the dry run and retrieve the fetch points
        info!("2. Running dry-run... ");
        let dry_run_results: DryRunResult = cairo_dry_run(
            dry_run_program_path,
            input_string,
            compile_config.save_fetch_keys_file.clone(),
        )?;

        // Check if the program hash matches the expected hash
        if dry_run_results[0].program_hash != self[0].task.program_hash {
            return Err(CompileError::ClassHashMismatch);
        }

        // Ensure only one module is supported
        if dry_run_results.len() != 1 {
            panic!("Multiple Modules are not supported");
        }

        // Extract the dry run module result
        let dry_run_module = dry_run_results.into_iter().next().unwrap();
        let commit_results = vec![dry_run_module.result.to_combined_string().into()];

        // 3. Categorize fetch keys by chain ID
        let categorized_keys = categorize_fetch_keys(dry_run_module.fetch_keys);
        if categorized_keys.len() > 1 {
            // TODO: This is a temporary solution. Need to handle multiple chain IDs in the future
            panic!("Multiple chain IDs are not supported yet");
        }

        // Initialize maps to store fetched proofs grouped by chain ID
        let mut accounts_map = HashMap::new();
        let mut storages_map = HashMap::new();
        let mut transactions_map = HashMap::new();
        let mut transaction_receipts_map = HashMap::new();
        let mut mmr_header_map = HashMap::new();

        info!("3. Fetching proofs from provider...");
        // Loop through each chain ID and fetch proofs
        for (chain_id, keys) in categorized_keys {
            info!("target provider chain id: {}", chain_id);
            let target_provider_config = compile_config
                .provider_config
                .get(&chain_id)
                .expect("target task's chain had not been configured.");
            let provider = new_provider_from_config(target_provider_config);
            let results = provider.fetch_proofs_from_keys(keys).await?;

            // Update the maps with fetched results
            mmr_header_map.insert(
                chain_id.to_numeric_id(),
                HashSet::from_iter(results.mmr_with_headers.into_iter()),
            );
            accounts_map.insert(
                chain_id.to_numeric_id(),
                HashSet::from_iter(results.accounts.into_iter()),
            );
            storages_map.insert(
                chain_id.to_numeric_id(),
                HashSet::from_iter(results.storages.into_iter()),
            );
            transactions_map.insert(
                chain_id.to_numeric_id(),
                HashSet::from_iter(results.transactions.into_iter()),
            );
            transaction_receipts_map.insert(
                chain_id.to_numeric_id(),
                HashSet::from_iter(results.transaction_receipts.into_iter()),
            );
        }

        // Create and return the compilation result containing all relevant proofs
        let compiled_result = CompilationResult::new(
            commit_results,
            mmr_header_map,
            accounts_map,
            storages_map,
            transactions_map,
            transaction_receipts_map,
        );
        Ok(compiled_result)
    }
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
            cairo_format::DryRunProcessedModule::new(module.task.inputs, module.module_class);
        collected_results.push(input_module);
    }

    Ok(DryRunnerProgramInput::new(
        identified_keys_file,
        collected_results,
    ))
}
