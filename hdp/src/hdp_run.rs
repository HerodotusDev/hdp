use alloy::primitives::ChainId;
use anyhow::Result;
use reqwest::Url;
use std::{env, fs, path::PathBuf};
use tracing::info;

use crate::{
    constant::{DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE, DEFAULT_SOUND_CAIRO_RUN_CAIRO_FILE},
    preprocessor::{compile::config::CompilerConfig, PreProcessor},
    primitives::{processed_types::cairo_format::AsCairoFormat, task::TaskEnvelope},
    processor::Processor,
    provider::evm::config::EvmProviderConfig,
};

/// HdpRunConfig for the CLI
#[derive(Debug)]
pub struct HdpRunConfig {
    pub evm_provider: EvmProviderConfig,
    pub dry_run_program_path: PathBuf,
    pub sound_run_program_path: PathBuf,
    pub save_fetch_keys_file: Option<PathBuf>,
}

#[cfg(feature = "test_utils")]
impl Default for HdpRunConfig {
    fn default() -> Self {
        Self {
            evm_provider: EvmProviderConfig::default(),
            dry_run_program_path: DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE.into(),
            sound_run_program_path: DEFAULT_SOUND_CAIRO_RUN_CAIRO_FILE.into(),
            save_fetch_keys_file: None,
        }
    }
}

impl HdpRunConfig {
    pub async fn init(
        cli_rpc_url: Option<Url>,
        cli_chain_id: Option<ChainId>,
        cli_dry_run_cairo_file: Option<PathBuf>,
        cli_sound_run_cairo_file: Option<PathBuf>,
        cli_save_fetch_keys_file: Option<PathBuf>,
    ) -> Self {
        let chain_id = cli_chain_id.unwrap_or_else(|| {
            env::var("CHAIN_ID")
                .expect("CHAIN_ID must be set")
                .parse()
                .expect("CHAIN_ID must be a number")
        });
        let rpc_url = cli_rpc_url.unwrap_or_else(|| {
            env::var("RPC_URL")
                .expect("RPC_URL must be set")
                .parse()
                .expect("RPC_URL must be a valid URL")
        });
        let rpc_chunk_size = env::var("RPC_CHUNK_SIZE")
            .unwrap_or_else(|_| "40".to_string())
            .parse()
            .expect("RPC_CHUNK_SIZE must be a number");
        let save_fetch_keys_file: Option<PathBuf> = cli_save_fetch_keys_file
            .or_else(|| env::var("SAVE_FETCH_KEYS_FILE").ok().map(PathBuf::from));
        let dry_run_cairo_path: PathBuf = cli_dry_run_cairo_file.unwrap_or_else(|| {
            env::var("DRY_RUN_CAIRO_PATH")
                .unwrap_or_else(|_| DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE.to_string())
                .parse()
                .expect("DRY_RUN_CAIRO_PATH must be a path to a cairo file")
        });
        let sound_run_cairo_path: PathBuf = cli_sound_run_cairo_file.unwrap_or_else(|| {
            env::var("SOUND_RUN_CAIRO_PATH")
                .unwrap_or_else(|_| DEFAULT_SOUND_CAIRO_RUN_CAIRO_FILE.to_string())
                .parse()
                .expect("SOUND_RUN_CAIRO_PATH must be a path to a cairo file")
        });

        HdpRunConfig {
            evm_provider: EvmProviderConfig {
                rpc_url,
                chain_id,
                max_requests: rpc_chunk_size,
            },
            dry_run_program_path: dry_run_cairo_path,
            sound_run_program_path: sound_run_cairo_path,
            save_fetch_keys_file,
        }
    }
}

/// Main entry point for the hdp_run command.
/// # Arguments
/// - `hdp_run_config`: The configuration for the hdp_run command.
/// - `tasks`: The tasks to be processed.
/// - `pre_processor_output_file`: The path to the file where the preprocessor output will be saved. (Optional)
/// - `output_file`: The path to the file where the output will be saved. (Optional)
/// - `cairo_pie_file`: The path to the file where the cairo pie will be saved. (Optional)
pub async fn hdp_run(
    hdp_run_config: &HdpRunConfig,
    tasks: Vec<TaskEnvelope>,
    pre_processor_output_file: Option<PathBuf>,
    output_file: Option<PathBuf>,
    cairo_pie_file: Option<PathBuf>,
) -> Result<()> {
    let compiler_config = CompilerConfig {
        dry_run_program_path: hdp_run_config.dry_run_program_path.clone(),
        provider_config: hdp_run_config.evm_provider.clone(),
        save_fetch_keys_file: hdp_run_config.save_fetch_keys_file.clone(),
    };
    let preprocessor = PreProcessor::new_with_config(compiler_config);
    let preprocessor_result = preprocessor.process(tasks).await?;

    if pre_processor_output_file.is_none() {
        info!("Finished pre processing the data");
        Ok(())
    } else {
        let input_string = serde_json::to_string_pretty(&preprocessor_result.as_cairo_format())
            .map_err(|e| anyhow::anyhow!("Failed to serialize preprocessor result: {}", e))?;
        if let Some(input_file_path) = pre_processor_output_file {
            fs::write(&input_file_path, input_string)
                .map_err(|e| anyhow::anyhow!("Unable to write input file: {}", e))?;
            info!(
                "Finished pre processing the data, saved the input file in {}",
                input_file_path.display()
            );
            if output_file.is_none() && cairo_pie_file.is_none() {
                Ok(())
            } else {
                info!("Starting processing the data... ");
                let output_file_path = output_file
                    .ok_or_else(|| anyhow::anyhow!("Output file path should be specified"))?;
                let pie_file_path = cairo_pie_file
                    .ok_or_else(|| anyhow::anyhow!("PIE path should be specified"))?;
                let processor = Processor::new(hdp_run_config.sound_run_program_path.clone());
                let processor_result = processor
                    .process(preprocessor_result, &pie_file_path)
                    .await?;
                fs::write(
                    &output_file_path,
                    serde_json::to_string_pretty(&processor_result).map_err(|e| {
                        anyhow::anyhow!("Failed to serialize processor result: {}", e)
                    })?,
                )
                .map_err(|e| anyhow::anyhow!("Unable to write output file: {}", e))?;

                info!(
                    "Finished processing the data, saved the output file in {} and pie file in {}",
                    output_file_path.display(),
                    pie_file_path.display()
                );
                Ok(())
            }
        } else {
            Err(anyhow::anyhow!("Cairo input path should be specified"))
        }
    }
}
