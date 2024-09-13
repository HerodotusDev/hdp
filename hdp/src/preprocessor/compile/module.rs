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

use std::collections::HashSet;
use std::path::PathBuf;
use tracing::info;

use super::config::CompilerConfig;
use super::{CompilationResult, CompileError};

pub type ModuleVec = Vec<ExtendedModule>;

pub async fn compile_modules(
    modules: ModuleVec,
    compile_config: &CompilerConfig,
) -> Result<CompilationResult, CompileError> {
    info!("target task: {:#?}", modules[0].task);
    let dry_run_program_path = compile_config.dry_run_program_path.clone();

    let input = generate_input(modules.to_vec(), PathBuf::from(DRY_CAIRO_RUN_OUTPUT_FILE)).await?;
    let input_string =
        serde_json::to_string_pretty(&input).expect("Failed to serialize module class");

    // 2. run the dry run and get the fetch points
    info!("2. Running dry-run... ");
    let keys: DryRunResult = cairo_dry_run(
        dry_run_program_path,
        input_string,
        compile_config.save_fetch_keys_file.clone(),
    )?;

    if keys[0].program_hash != modules[0].task.program_hash {
        return Err(CompileError::ClassHashMismatch);
    }

    if keys.len() != 1 {
        panic!("Multiple Modules are not supported");
    }

    let dry_runned_module = keys.into_iter().next().unwrap();
    let commit_results_maps = vec![dry_runned_module.result.to_combined_string().into()];

    // 3. call provider using keys
    let keys_maps_chain = categorize_fetch_keys(dry_runned_module.fetch_keys);
    if keys_maps_chain.len() > 1 {
        // TODO: This is temporary solution. Need to handle multiple chain id in future
        panic!("Multiple chain id is not supported yet");
    }

    let mut accounts = HashSet::new();
    let mut storages = HashSet::new();
    let mut transactions = HashSet::new();
    let mut transaction_receipts = HashSet::new();
    let mut mmr_header_map = HashSet::new();

    info!("3. Fetching proofs from provider...");
    for (chain_id, keys) in keys_maps_chain {
        info!("target provider chain id: {}", chain_id);
        let target_provider_config = compile_config
            .provider_config
            .get(&chain_id)
            .expect("target task's chain had not been configured.");
        let provider = new_provider_from_config(target_provider_config);
        let results = provider.fetch_proofs_from_keys(keys).await?;

        // TODO: can we do better?
        mmr_header_map.extend(results.mmr_with_headers.into_iter());
        accounts.extend(results.accounts.into_iter());
        storages.extend(results.storages.into_iter());
        transactions.extend(results.transactions.into_iter());
        transaction_receipts.extend(results.transaction_receipts.into_iter());
    }

    // TODO : need fix
    let compiled_result = CompilationResult::new(
        11155111,
        commit_results_maps,
        mmr_header_map,
        accounts,
        storages,
        transactions,
        transaction_receipts,
    );
    Ok(compiled_result)
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
