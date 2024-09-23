use crate::{
    constant::{DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE, DEFAULT_SOUND_CAIRO_RUN_CAIRO_FILE},
    preprocessor::{compile::config::CompilerConfig, PreProcessor},
    primitives::{processed_types::cairo_format::AsCairoFormat, task::TaskEnvelope, ChainId},
    processor::Processor,
    provider::config::ProviderConfig,
};

use anyhow::Result;
use reqwest::Url;
use std::{collections::HashMap, env, fs, path::PathBuf};
use tracing::{debug, info};

/// HdpRunConfig for the CLI
#[derive(Debug)]
pub struct HdpRunConfig {
    // chain_id => provider config
    pub provider_config: HashMap<ChainId, ProviderConfig>,
    pub dry_run_program_path: PathBuf,
    pub sound_run_program_path: PathBuf,
    pub program_input_file: PathBuf,
    pub is_cairo_format: bool,
    pub batch_proof_file: Option<PathBuf>,
    pub cairo_pie_file: Option<PathBuf>,
    pub save_fetch_keys_file: Option<PathBuf>,
}

#[cfg(feature = "test_utils")]
impl Default for HdpRunConfig {
    fn default() -> Self {
        Self {
            provider_config: HashMap::new(),
            dry_run_program_path: DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE.into(),
            sound_run_program_path: DEFAULT_SOUND_CAIRO_RUN_CAIRO_FILE.into(),
            program_input_file: "program_input.json".into(),
            is_cairo_format: false,
            cairo_pie_file: None,
            batch_proof_file: None,
            save_fetch_keys_file: None,
        }
    }
}

impl HdpRunConfig {
    pub fn init(
        cli_dry_run_cairo_file: Option<PathBuf>,
        cli_sound_run_cairo_file: Option<PathBuf>,
        program_input_file: PathBuf,
        cli_is_cairo_format: bool,
        cli_save_fetch_keys_file: Option<PathBuf>,
        batch_proof_file: Option<PathBuf>,
        cli_cairo_pie_file: Option<PathBuf>,
    ) -> Self {
        let mut provider_config = HashMap::new();

        // Iterate through environment variables to find PROVIDER_URL and PROVIDER_CHUNK_SIZE configurations
        for (key, value) in env::vars() {
            if let Some(stripped_chain_id) = key.strip_prefix("PROVIDER_URL_") {
                let chain_id: ChainId = stripped_chain_id
                    .parse()
                    .expect("Invalid chain ID in PROVIDER_URL env var");
                let provider_url: Url = value.parse().expect("Invalid URL in PROVIDER_URL env var");

                let chunk_size_key = format!("PROVIDER_CHUNK_SIZE_{}", chain_id);
                let provider_chunk_size: u64 = env::var(&chunk_size_key)
                    .unwrap_or_else(|_| "40".to_string())
                    .parse()
                    .unwrap_or_else(|_| panic!("{} must be a number", chunk_size_key));

                provider_config.insert(
                    chain_id,
                    ProviderConfig {
                        provider_url,
                        chain_id,
                        max_requests: provider_chunk_size,
                    },
                );
            }
        }

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

        let config = HdpRunConfig {
            provider_config,
            dry_run_program_path: dry_run_cairo_path,
            sound_run_program_path: sound_run_cairo_path,
            program_input_file,
            is_cairo_format: cli_is_cairo_format,
            save_fetch_keys_file,
            batch_proof_file,
            cairo_pie_file: cli_cairo_pie_file,
        };

        debug!("Running with configuration: {:#?}", config);
        config
    }
}

/// Main entry point for the hdp_run command.
/// # Arguments
/// - `hdp_run_config`: The configuration for the hdp_run command.
/// - `tasks`: The tasks to be processed.
/// - `pre_processor_output_file`: The path to the file where the preprocessor output will be saved. (Optional)
/// - `output_file`: The path to the file where the output will be saved. (Optional)
/// - `cairo_pie_file`: The path to the file where the cairo pie will be saved. (Optional)
pub async fn run(hdp_run_config: &HdpRunConfig, tasks: Vec<TaskEnvelope>) -> Result<()> {
    let compiler_config = CompilerConfig {
        dry_run_program_path: hdp_run_config.dry_run_program_path.clone(),
        provider_config: hdp_run_config.provider_config.clone(),
        save_fetch_keys_file: hdp_run_config.save_fetch_keys_file.clone(),
    };
    let preprocessor = PreProcessor::new_with_config(compiler_config);
    let preprocessor_result = preprocessor.process(tasks).await?;

    let input_string = match hdp_run_config.is_cairo_format {
        true => serde_json::to_string_pretty(&preprocessor_result.as_cairo_format())
            .map_err(|e| anyhow::anyhow!("Failed to serialize preprocessor result: {}", e))?,
        false => serde_json::to_string_pretty(&preprocessor_result)
            .map_err(|e| anyhow::anyhow!("Failed to serialize preprocessor result: {}", e))?,
    };

    fs::write(&hdp_run_config.program_input_file, input_string)
        .map_err(|e| anyhow::anyhow!("Unable to write input file: {}", e))?;

    match &hdp_run_config.batch_proof_file {
        Some(batch_proof_file) => {
            let batch_proof_data = preprocessor_result.into_processor_output();
            fs::write(
                batch_proof_file,
                serde_json::to_string_pretty(&batch_proof_data)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize processor result: {}", e))?,
            )
            .map_err(|e| anyhow::anyhow!("Unable to write output file: {}", e))?;
            info!(
                "saved the batch proof file in {}",
                &batch_proof_file.display()
            );
        }
        None => {}
    }

    info!(
        "finished pre processing the data, saved the program input file in {}",
        &hdp_run_config.program_input_file.display()
    );

    if hdp_run_config.cairo_pie_file.is_none() {
        Ok(())
    } else {
        info!("starting processing the data... ");
        let pie_file_path = &hdp_run_config
            .cairo_pie_file
            .clone()
            .ok_or_else(|| anyhow::anyhow!("PIE path should be specified"))?;
        let processor = Processor::new(hdp_run_config.sound_run_program_path.clone());
        processor
            .process(preprocessor_result.as_cairo_format(), pie_file_path)
            .await?;

        info!(
            "finished processing the data, saved pie file in {}",
            pie_file_path.display()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_hdp_run_config_init_with_env() {
        // Set up environment variables
        env::set_var("PROVIDER_URL_ETHEREUM_MAINNET", "https://example.com/rpc1");
        env::set_var("PROVIDER_CHUNK_SIZE_ETHEREUM_MAINNET", "50");
        env::set_var("PROVIDER_URL_STARKNET_MAINNET", "https://example.com/rpc2");
        env::set_var("PROVIDER_CHUNK_SIZE_STARKNET_MAINNET", "60");
        env::set_var("DRY_RUN_CAIRO_PATH", "/path/to/dry_run.cairo");
        env::set_var("SOUND_RUN_CAIRO_PATH", "/path/to/sound_run.cairo");
        env::set_var("SAVE_FETCH_KEYS_FILE", "/path/to/save_fetch_keys.json");

        // Initialize HdpRunConfig
        let config = HdpRunConfig::init(
            None,
            None,
            PathBuf::from("input.json"),
            false,
            None,
            None,
            None,
        );

        // Assert provider configurations
        assert_eq!(config.provider_config.len(), 2);
        assert!(config
            .provider_config
            .contains_key(&ChainId::EthereumMainnet));
        assert!(config
            .provider_config
            .contains_key(&ChainId::StarknetMainnet));

        let provider_config_1 = config
            .provider_config
            .get(&ChainId::EthereumMainnet)
            .unwrap();
        assert_eq!(
            provider_config_1.provider_url.to_string(),
            "https://example.com/rpc1"
        );
        assert_eq!(provider_config_1.max_requests, 50);

        let provider_config_2 = config
            .provider_config
            .get(&ChainId::StarknetMainnet)
            .unwrap();
        assert_eq!(
            provider_config_2.provider_url.to_string(),
            "https://example.com/rpc2"
        );
        assert_eq!(provider_config_2.max_requests, 60);

        // Assert other configurations
        assert_eq!(
            config.dry_run_program_path,
            PathBuf::from("/path/to/dry_run.cairo")
        );
        assert_eq!(
            config.sound_run_program_path,
            PathBuf::from("/path/to/sound_run.cairo")
        );
        assert_eq!(config.program_input_file, PathBuf::from("input.json"));
        assert!(!config.is_cairo_format);
        assert_eq!(
            config.save_fetch_keys_file,
            Some(PathBuf::from("/path/to/save_fetch_keys.json"))
        );
        assert_eq!(config.batch_proof_file, None);
        assert_eq!(config.cairo_pie_file, None);

        // Clean up environment variables
        env::remove_var("PROVIDER_URL_1");
        env::remove_var("PROVIDER_CHUNK_SIZE_1");
        env::remove_var("PROVIDER_URL_2");
        env::remove_var("PROVIDER_CHUNK_SIZE_2");
        env::remove_var("DRY_RUN_CAIRO_PATH");
        env::remove_var("SOUND_RUN_CAIRO_PATH");
        env::remove_var("SAVE_FETCH_KEYS_FILE");
    }
}
