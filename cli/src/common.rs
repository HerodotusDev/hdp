use alloy::{
    primitives::{Bytes, ChainId},
    transports::http::reqwest::Url,
};
use anyhow::Result;
use hdp_preprocessor::{
    compile::config::CompilerConfig, module_registry::ModuleRegistry, PreProcessor,
    PreProcessorError,
};
use hdp_primitives::{
    constant::{DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE, DEFAULT_SOUND_CAIRO_RUN_CAIRO_FILE},
    processed_types::cairo_format::AsCairoFormat,
    solidity_types::{
        datalake_compute::BatchedDatalakeCompute, traits::BatchedDatalakeComputeCodecs,
    },
    task::{
        datalake::{
            block_sampled::BlockSampledDatalake, compute::Computation, envelope::DatalakeEnvelope,
            transactions::TransactionsInBlockDatalake, DatalakeCompute,
        },
        TaskEnvelope,
    },
};
use hdp_provider::evm::config::EvmProviderConfig;
use std::{fs, path::PathBuf};
use tracing_subscriber::FmtSubscriber;

use clap::Parser;
use hdp_processor::Processor;

use tracing::{info, Level};

use crate::{
    commands::{DataLakeCommands, HDPCli, HDPCliCommands},
    config::Config,
    interactive,
    module_config::ModuleConfig,
    query::{SubmitBatchQuery, Task},
};

pub async fn run() -> anyhow::Result<()> {
    let start_run = std::time::Instant::now();
    let cli = init_cli()?;
    match cli.command {
        HDPCliCommands::Start => {
            interactive::run_interactive().await?;
        }
        HDPCliCommands::RunDatalake {
            allow_process,
            rpc_url,
            chain_id,
            preprocessor_output_file,
            output_file,
            cairo_pie_file,
            aggregate_fn_id,
            aggregate_fn_ctx,
            command,
        } => {
            let datalake = match command {
                DataLakeCommands::BlockSampled {
                    block_range_start,
                    block_range_end,
                    sampled_property,
                    increment,
                } => DatalakeEnvelope::BlockSampled(BlockSampledDatalake::new(
                    11155111,
                    block_range_start,
                    block_range_end,
                    increment,
                    sampled_property,
                )),
                DataLakeCommands::TransactionsInBlock {
                    target_block,
                    sampled_property,
                    start_index,
                    end_index,
                    increment,
                    included_types,
                } => DatalakeEnvelope::TransactionsInBlock(TransactionsInBlockDatalake::new(
                    11155111,
                    target_block,
                    sampled_property,
                    start_index,
                    end_index,
                    increment,
                    included_types,
                )),
            };
            let (encoded_datalakes, encoded_computes) = vec![DatalakeCompute::new(
                datalake,
                Computation::new(aggregate_fn_id, aggregate_fn_ctx),
            )]
            .encode()?;

            let encoded_datalakes_string = Bytes::from(encoded_datalakes);
            let encoded_computes_string = Bytes::from(encoded_computes);

            info!("Encoded datalakes: {:#?}", encoded_datalakes_string);
            info!("Encoded computes: {:#?}", encoded_computes_string);

            // if allow_process is true, then run the processor
            if allow_process {
                datalake_entry_run(
                    Some(encoded_computes_string),
                    Some(encoded_datalakes_string),
                    rpc_url,
                    chain_id,
                    preprocessor_output_file,
                    output_file,
                    cairo_pie_file,
                )
                .await?
            }
        }
        HDPCliCommands::RunEncodedDatalake {
            tasks,
            datalakes,
            rpc_url,
            chain_id,
            preprocessor_output_file,
            output_file,
            cairo_pie_file,
        } => {
            datalake_entry_run(
                tasks,
                datalakes,
                rpc_url,
                chain_id,
                preprocessor_output_file,
                output_file,
                cairo_pie_file,
            )
            .await?
        }
        HDPCliCommands::RunModule {
            class_hash,
            local_class_path,
            module_inputs,
            module_registry_rpc_url,
            rpc_url,
            chain_id,
            preprocessor_output_file,
            output_file,
            cairo_pie_file,
        } => {
            module_entry_run(
                class_hash,
                local_class_path,
                module_inputs,
                module_registry_rpc_url,
                rpc_url,
                chain_id,
                preprocessor_output_file,
                output_file,
                cairo_pie_file,
            )
            .await?;
        }
        HDPCliCommands::Run {
            request_file,
            module_registry_rpc_url,
            rpc_url,
            preprocessor_output_file,
            output_file,
            cairo_pie_file,
        } => {
            entry_run(
                request_file,
                module_registry_rpc_url,
                rpc_url,
                preprocessor_output_file,
                output_file,
                cairo_pie_file,
            )
            .await?;
        }
    }
    let duration_run = start_run.elapsed();
    info!("HDP Cli Finished in: {:?}", duration_run);
    Ok(())
}

/// Initialize the CLI
fn init_cli() -> Result<HDPCli> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    let cli = HDPCli::parse();
    dotenv::dotenv().ok();
    Ok(cli)
}

#[allow(clippy::too_many_arguments)]
pub async fn module_entry_run(
    class_hash: Option<String>,
    local_class_path: Option<PathBuf>,
    module_inputs: Vec<String>,
    module_registry_rpc_url: Option<Url>,
    rpc_url: Option<Url>,
    chain_id: Option<ChainId>,
    preprocessor_output_file: Option<PathBuf>,
    output_file: Option<PathBuf>,
    cairo_pie_file: Option<PathBuf>,
) -> Result<()> {
    let config = ModuleConfig::init(rpc_url, chain_id, module_registry_rpc_url).await;
    let module_registry = ModuleRegistry::new(config.module_registry_rpc_url.clone());
    let module = module_registry
        .get_extended_module_from_class_source_string(class_hash, local_class_path, module_inputs)
        .await?;
    // TODO: for now, we only support one task if its a module
    let tasks = vec![TaskEnvelope::Module(module)];
    let provider_config = config.evm_provider.clone();

    handle_running_tasks(
        provider_config,
        tasks,
        preprocessor_output_file,
        output_file,
        cairo_pie_file,
    )
    .await?;
    Ok(())
}

pub async fn datalake_entry_run(
    tasks: Option<Bytes>,
    datalakes: Option<Bytes>,
    rpc_url: Option<Url>,
    chain_id: Option<ChainId>,
    pre_processor_output: Option<PathBuf>,
    output_file: Option<PathBuf>,
    cairo_pie_file: Option<PathBuf>,
) -> Result<()> {
    let config = Config::init(rpc_url, datalakes, tasks, chain_id).await;
    let tasks = BatchedDatalakeCompute::decode(&config.datalakes, &config.tasks)
        .map_err(PreProcessorError::DecodeError)?;
    let tasks = tasks
        .into_iter()
        .map(TaskEnvelope::DatalakeCompute)
        .collect::<Vec<_>>();

    handle_running_tasks(
        config.evm_provider.clone(),
        tasks,
        pre_processor_output,
        output_file,
        cairo_pie_file,
    )
    .await?;
    Ok(())
}

async fn handle_running_tasks(
    evm_config: EvmProviderConfig,
    tasks: Vec<TaskEnvelope>,
    pre_processor_output_file: Option<PathBuf>,
    output_file: Option<PathBuf>,
    cairo_pie_file: Option<PathBuf>,
) -> Result<()> {
    let compiler_config = CompilerConfig {
        dry_run_program_path: PathBuf::from(&DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE),
        provider_config: evm_config,
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
                let processor = Processor::new(PathBuf::from(DEFAULT_SOUND_CAIRO_RUN_CAIRO_FILE));
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

pub async fn entry_run(
    request_file: PathBuf,
    module_registry_rpc_url: Option<Url>,
    rpc_url: Option<Url>,
    pre_processor_output_file: PathBuf,
    output_file: Option<PathBuf>,
    cairo_pie_file: Option<PathBuf>,
) -> Result<()> {
    let request_context = fs::read_to_string(request_file).unwrap();
    let parsed: SubmitBatchQuery = serde_json::from_str(&request_context).unwrap();
    let config = ModuleConfig::init(
        rpc_url,
        Some(parsed.destination_chain_id),
        module_registry_rpc_url,
    )
    .await;
    let module_registry = ModuleRegistry::new(config.module_registry_rpc_url.clone());
    let mut task_envelopes = Vec::new();
    for task in parsed.tasks {
        match task {
            Task::DatalakeCompute(task) => {
                task_envelopes.push(TaskEnvelope::DatalakeCompute(task));
            }
            Task::Module(task) => {
                let module = module_registry
                    .get_extended_module_from_class_source(
                        Some(task.program_hash),
                        None,
                        task.inputs,
                    )
                    .await?;
                task_envelopes.push(TaskEnvelope::Module(module));
            }
        }
    }
    let compiler_config = CompilerConfig {
        dry_run_program_path: PathBuf::from(&DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE),
        provider_config: config.evm_provider.clone(),
    };
    let preprocessor = PreProcessor::new_with_config(compiler_config);
    let preprocessor_result = preprocessor.process(task_envelopes).await?;
    let input_string = serde_json::to_string_pretty(&preprocessor_result.as_cairo_format())
        .map_err(|e| anyhow::anyhow!("Failed to serialize preprocessor result: {}", e))?;
    fs::write(&pre_processor_output_file, input_string)
        .map_err(|e| anyhow::anyhow!("Unable to write input file: {}", e))?;
    info!(
        "Finished pre processing the data, saved the input file of cairo program in {}",
        pre_processor_output_file.display()
    );
    if output_file.is_none() && cairo_pie_file.is_none() {
        Ok(())
    } else {
        info!("Starting processing the data... ");
        let output_file_path =
            output_file.ok_or_else(|| anyhow::anyhow!("Output file path should be specified"))?;
        let pie_file_path =
            cairo_pie_file.ok_or_else(|| anyhow::anyhow!("PIE path should be specified"))?;
        let processor = Processor::new(PathBuf::from(DEFAULT_SOUND_CAIRO_RUN_CAIRO_FILE));
        let processor_result = processor
            .process(preprocessor_result, &pie_file_path)
            .await?;
        fs::write(
            &output_file_path,
            serde_json::to_string_pretty(&processor_result)
                .map_err(|e| anyhow::anyhow!("Failed to serialize processor result: {}", e))?,
        )
        .map_err(|e| anyhow::anyhow!("Unable to write output file: {}", e))?;

        info!(
            "Finished processing the data, saved the output file in {} and pie file in {}",
            output_file_path.display(),
            pie_file_path.display()
        );
        Ok(())
    }
}
