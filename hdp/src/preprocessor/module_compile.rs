// ! This file is sketching out compilation logic that only considering module task.
// ! We already confirmed on direction to deprecating datalake, which curren ./compile file
//! is causing too much overhead interms of abstraction around legacy types that we supported.
//! Ideally later this file will deprecate ./compile

use alloy::primitives::U256;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

use crate::{
    cairo_runner::{cairo_dry_run, dry_run::DryRunResult, input::dry_run::DryRunnerProgramInput},
    constant::DRY_CAIRO_RUN_OUTPUT_FILE,
    primitives::{
        processed_types::{block_proofs::ProcessedBlockProofs, cairo_format},
        task::ExtendedModule,
        ChainId,
    },
    provider::{key::categorize_fetch_keys, traits::new_provider_from_config},
};

use super::compile::{config::CompilerConfig, CompileError};

#[derive(Debug, Default, PartialEq)]
pub struct ModuleCompilationResult {
    /// results of tasks
    pub task_results: Vec<U256>,
    /// proofs
    pub proofs: HashMap<ChainId, ProcessedBlockProofs>,
}

pub async fn module_compile(
    task: ExtendedModule,
    compile_config: &CompilerConfig,
) -> Result<ModuleCompilationResult, CompileError> {
    // Log the target task for debugging purposes
    info!("target task: {:#?}", task.task);
    let dry_run_program_path = compile_config.dry_run_program_path.clone();

    // Generate input for the dry run based on the extended modules
    let dry_run_input = DryRunnerProgramInput::new(
        PathBuf::from(DRY_CAIRO_RUN_OUTPUT_FILE),
        vec![cairo_format::DryRunProcessedModule::new(
            task.task.inputs,
            task.module_class,
        )],
    );
    let input_string =
        serde_json::to_string_pretty(&dry_run_input).expect("Failed to serialize module class");

    // 2. Run the dry run and retrieve the fetch points
    info!("2. Running dry-run... ");
    let dry_run_results: DryRunResult = cairo_dry_run(
        dry_run_program_path,
        input_string,
        compile_config.save_fetch_keys_file.clone(),
    )?;

    // TODO: prob as soon as we deprecate data lake this check no need
    // Check if the program hash matches the expected hash
    if dry_run_results[0].program_hash != task.task.program_hash {
        return Err(CompileError::ClassHashMismatch);
    }
    // Ensure only one module is supported
    if dry_run_results.len() != 1 {
        panic!("Multiple Modules are not supported");
    }

    // Extract the dry run module result
    let dry_run_module = dry_run_results.into_iter().next().unwrap();
    let task_results = vec![dry_run_module.result.into()];

    // 3. Categorize fetch keys by chain ID
    let categorized_keys = categorize_fetch_keys(dry_run_module.fetch_keys);

    let mut proofs_map: std::collections::HashMap<ChainId, ProcessedBlockProofs> = HashMap::new();
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
        proofs_map.insert(chain_id, results);
    }

    Ok(ModuleCompilationResult {
        task_results,
        proofs: proofs_map,
    })
}
