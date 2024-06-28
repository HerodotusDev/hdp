use alloy::{
    primitives::{Bytes, ChainId},
    transports::http::reqwest::Url,
};
use anyhow::Result;
use hdp_primitives::processed_types::cairo_format::AsCairoFormat;
use hdp_provider::evm::provider::EvmProviderConfig;
use std::{fs, path::PathBuf};
use tracing_subscriber::FmtSubscriber;

use clap::Parser;
use hdp_core::{
    compiler::module::ModuleCompilerConfig,
    config::Config,
    pre_processor::{PreProcessor, PreProcessorConfig},
    processor::Processor,
};

use tracing::{info, Level};

use crate::command::HDPCli;

/// Initialize the CLI
pub fn init_cli() -> Result<HDPCli> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    let cli = HDPCli::parse();
    dotenv::dotenv().ok();
    Ok(cli)
}

pub async fn handle_run(
    tasks: Option<Bytes>,
    datalakes: Option<Bytes>,
    rpc_url: Option<Url>,
    chain_id: Option<ChainId>,
    output_file: Option<PathBuf>,
    cairo_input: Option<PathBuf>,
    pie_file: Option<PathBuf>,
) -> Result<()> {
    // TODO: module config is not used rn, hard coded url
    let url: Url = "http://localhost:3030".parse()?;
    let program_path = "./build/compiled_cairo/hdp.json";
    let config = Config::init(rpc_url, datalakes, tasks, chain_id).await;
    let datalake_config = EvmProviderConfig {
        rpc_url: config.rpc_url.clone(),
        chain_id: config.chain_id,
        max_requests: config.rpc_chunk_size,
    };
    let module_config = ModuleCompilerConfig {
        module_registry_rpc_url: url,
        program_path: PathBuf::from(&program_path),
    };
    let preprocessor_config = PreProcessorConfig::new(datalake_config, module_config);
    let preprocessor = PreProcessor::new_with_config(preprocessor_config);
    let result = preprocessor
        .process_from_serialized(config.datalakes.clone(), config.tasks.clone())
        .await?;

    if cairo_input.is_none() {
        info!("Finished pre processing the data");
        Ok(())
    } else {
        let input_string = serde_json::to_string_pretty(&result.as_cairo_format())
            .map_err(|e| anyhow::anyhow!("Failed to serialize preprocessor result: {}", e))?;
        if let Some(input_file_path) = cairo_input {
            fs::write(&input_file_path, input_string.clone())
                .map_err(|e| anyhow::anyhow!("Unable to write file: {}", e))?;
            info!(
                "Finished pre processing the data, saved the input file in {}",
                input_file_path.display()
            );
            if output_file.is_none() && pie_file.is_none() {
                Ok(())
            } else {
                info!("Starting processing the data... ");
                let output_file_path = output_file
                    .ok_or_else(|| anyhow::anyhow!("Output file path should be specified"))?;
                let pie_file_path =
                    pie_file.ok_or_else(|| anyhow::anyhow!("PIE path should be specified"))?;
                let processor = Processor::new(PathBuf::from(program_path));
                let processor_result = processor.process(result, pie_file_path.clone()).await?;
                let output_string = serde_json::to_string_pretty(&processor_result)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize processor result: {}", e))?;
                fs::write(&output_file_path, output_string)
                    .map_err(|e| anyhow::anyhow!("Unable to write file: {}", e))?;
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
