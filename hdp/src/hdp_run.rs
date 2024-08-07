use anyhow::Result;
use std::{fs, path::PathBuf};
use tracing::info;

use crate::{
    config::HdpRunConfig,
    preprocessor::{compile::config::CompilerConfig, PreProcessor},
    primitives::{processed_types::cairo_format::AsCairoFormat, task::TaskEnvelope},
    processor::Processor,
};

pub async fn hdp_run(
    config: &HdpRunConfig,
    tasks: Vec<TaskEnvelope>,
    pre_processor_output_file: Option<PathBuf>,
    output_file: Option<PathBuf>,
    cairo_pie_file: Option<PathBuf>,
) -> Result<()> {
    let compiler_config = CompilerConfig {
        dry_run_program_path: config.dry_run_program_path.clone(),
        provider_config: config.evm_provider.clone(),
        save_fetch_keys_file: config.save_fetch_keys_file.clone(),
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
                let processor = Processor::new(config.sound_run_program_path.clone());
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
