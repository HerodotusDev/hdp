use alloy::{
    primitives::{Bytes, ChainId},
    transports::http::reqwest::Url,
};
use anyhow::Result;
use hdp_preprocessor::{
    compile::{module::ModuleCompilerConfig, CompileConfig},
    PreProcessor, PreProcessorError,
};
use hdp_primitives::{
    processed_types::cairo_format::AsCairoFormat,
    solidity_types::{
        datalake_compute::BatchedDatalakeCompute,
        traits::{BatchedDatalakeComputeCodecs, DatalakeComputeCodecs},
    },
    task::{
        datalake::{
            block_sampled::BlockSampledDatalake, compute::Computation, envelope::DatalakeEnvelope,
            transactions::TransactionsInBlockDatalake, DatalakeCompute,
        },
        module::Module,
        TaskEnvelope,
    },
};
use hdp_provider::evm::provider::EvmProviderConfig;
use std::{fs, path::PathBuf};
use tracing_subscriber::FmtSubscriber;

use clap::Parser;
use hdp_processor::Processor;

use tracing::{info, Level};

use crate::{
    commands::{DataLakeCommands, HDPCli, HDPCliCommands},
    config::Config,
    interactive,
};

pub async fn run() -> anyhow::Result<()> {
    let start_run = std::time::Instant::now();
    let cli = init_cli()?;
    match cli.command {
        HDPCliCommands::Start => {
            interactive::run_interactive().await?;
        }
        HDPCliCommands::Encode {
            allow_process,
            rpc_url,
            chain_id,
            output_file,
            cairo_input,
            pie_file,
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
                    block_range_start,
                    block_range_end,
                    sampled_property,
                    increment,
                )),
                DataLakeCommands::TransactionsInBlock {
                    target_block,
                    sampled_property,
                    start_index,
                    end_index,
                    increment,
                    included_types,
                } => DatalakeEnvelope::Transactions(TransactionsInBlockDatalake::new(
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
                    output_file,
                    cairo_input,
                    pie_file,
                )
                .await?
            }
        }
        HDPCliCommands::Decode { tasks, datalakes } => {
            let decoded_tasks = BatchedDatalakeCompute::decode(&datalakes, &tasks)?;
            info!("Decoded tasks: {:#?}", decoded_tasks);
        }
        HDPCliCommands::DecodeOne { task, datalake } => {
            let decoded_task = DatalakeCompute::decode(&datalake, &task)?;
            info!("Decoded task: {:#?}", decoded_task);
        }
        HDPCliCommands::Run {
            tasks,
            datalakes,
            rpc_url,
            chain_id,
            output_file,
            cairo_input,
            pie_file,
        } => {
            datalake_entry_run(
                tasks,
                datalakes,
                rpc_url,
                chain_id,
                output_file,
                cairo_input,
                pie_file,
            )
            .await?
        }
        HDPCliCommands::LocalRunModule {
            class_hash,
            local_class_path,
            module_inputs,
            rpc_url,
            chain_id,
            output_file,
            cairo_input,
            pie_file,
        } => {
            module_entry_run(
                class_hash,
                local_class_path,
                module_inputs,
                rpc_url,
                chain_id,
                output_file,
                cairo_input,
                pie_file,
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
    class_hash: String,
    local_class_path: Option<PathBuf>,
    module_inputs: Vec<String>,
    rpc_url: Option<Url>,
    chain_id: Option<ChainId>,
    output_file: Option<PathBuf>,
    cairo_input: Option<PathBuf>,
    pie_file: Option<PathBuf>,
) -> Result<()> {
    let url: Url =
        "https://starknet-sepolia.g.alchemy.com/v2/lINonYKIlp4NH9ZI6wvqJ4HeZj7T4Wm6".parse()?;

    let program_path = "./build/compiled_cairo/contract_dry_run.json";
    let module = Module::new_from_string(class_hash, module_inputs, local_class_path)?;

    let tasks = vec![TaskEnvelope::Module(module)];
    let module_config = ModuleCompilerConfig {
        module_registry_rpc_url: url,
        program_path: PathBuf::from(&program_path),
    };

    let provider_config = EvmProviderConfig {
        rpc_url: rpc_url.unwrap(),
        chain_id: chain_id.unwrap(),
        max_requests: 100,
    };
    let compile_config = CompileConfig {
        provider: provider_config,
        module: module_config,
    };
    handle_running_tasks(compile_config, tasks, output_file, cairo_input, pie_file).await?;
    Ok(())
}

pub async fn datalake_entry_run(
    tasks: Option<Bytes>,
    datalakes: Option<Bytes>,
    rpc_url: Option<Url>,
    chain_id: Option<ChainId>,
    output_file: Option<PathBuf>,
    cairo_input: Option<PathBuf>,
    pie_file: Option<PathBuf>,
) -> Result<()> {
    let config = Config::init(rpc_url, datalakes, tasks, chain_id).await;
    // 1. decode the tasks
    let tasks = BatchedDatalakeCompute::decode(&config.datalakes, &config.tasks)
        .map_err(PreProcessorError::DecodeError)?;

    // wrap into TaskEnvelope
    // TODO: this is temporary, need to remove this
    let tasks = tasks
        .iter()
        .map(|task| TaskEnvelope::DatalakeCompute(task.clone()))
        .collect::<Vec<_>>();

    // TODO: module config is not used rn, hard coded url
    let url: Url = "http://localhost:3030".parse()?;
    let program_path = "./build/compiled_cairo/hdp.json";
    let module_config = ModuleCompilerConfig {
        module_registry_rpc_url: url,
        program_path: PathBuf::from(&program_path),
    };
    let compile_config = CompileConfig {
        provider: config.evm_provider.clone(),
        module: module_config,
    };
    handle_running_tasks(compile_config, tasks, output_file, cairo_input, pie_file).await?;

    Ok(())
}

async fn handle_running_tasks(
    compile_config: CompileConfig,
    tasks: Vec<TaskEnvelope>,
    output_file: Option<PathBuf>,
    cairo_input: Option<PathBuf>,
    pie_file: Option<PathBuf>,
) -> Result<()> {
    info!("Target Tasks: {:#?}", tasks);
    let program_path = "./build/compiled_cairo/hdp.json";
    let preprocessor = PreProcessor::new_with_config(compile_config);
    let result = preprocessor.process(tasks).await?;

    if cairo_input.is_none() {
        info!("Finished pre processing the data");
        Ok(())
    } else {
        let input_string = serde_json::to_string_pretty(&result.as_cairo_format())
            .map_err(|e| anyhow::anyhow!("Failed to serialize preprocessor result: {}", e))?;
        if let Some(input_file_path) = cairo_input {
            fs::write(&input_file_path, input_string.clone())
                .map_err(|e| anyhow::anyhow!("Unable to write input file: {}", e))?;
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
