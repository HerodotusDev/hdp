use alloy::{primitives::ChainId, transports::http::reqwest::Url};
use anyhow::Result;
use hdp_preprocessor::{
    compile::config::CompilerConfig, module_registry::ModuleRegistry, PreProcessor,
};
use hdp_primitives::{
    aggregate_fn::{AggregationFunction, FunctionContext},
    processed_types::cairo_format::AsCairoFormat,
    task::{
        datalake::{
            block_sampled::BlockSampledDatalake, compute::Computation, envelope::DatalakeEnvelope,
            transactions::TransactionsInBlockDatalake, DatalakeCompute,
        },
        TaskEnvelope,
    },
};
use std::{fs, path::PathBuf};
use tracing_subscriber::FmtSubscriber;

use clap::Parser;
use hdp_processor::Processor;

use tracing::{info, Level};

use crate::{
    commands::{DataLakeCommands, HDPCli, HDPCliCommands},
    config::Config,
    interactive,
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
            rpc_url,
            chain_id,
            preprocessor_output_file,
            sound_run_cairo_file,
            output_file,
            cairo_pie_file,
            aggregate_fn_id,
            aggregate_fn_ctx,
            datalake,
        } => {
            datalake_entry_run(
                aggregate_fn_id,
                aggregate_fn_ctx,
                datalake,
                rpc_url,
                chain_id,
                preprocessor_output_file,
                sound_run_cairo_file,
                output_file,
                cairo_pie_file,
            )
            .await?
        }

        HDPCliCommands::RunModule {
            program_hash,
            local_class_path,
            save_fetch_keys_file,
            module_inputs,
            rpc_url,
            chain_id,
            dry_run_cairo_file,
            preprocessor_output_file,
            sound_run_cairo_file,
            output_file,
            cairo_pie_file,
        } => {
            module_entry_run(
                program_hash,
                local_class_path,
                save_fetch_keys_file,
                module_inputs,
                rpc_url,
                chain_id,
                dry_run_cairo_file,
                preprocessor_output_file,
                sound_run_cairo_file,
                output_file,
                cairo_pie_file,
            )
            .await?;
        }
        HDPCliCommands::Run {
            request_file,
            rpc_url,
            dry_run_cairo_file,
            preprocessor_output_file,
            sound_run_cairo_file,
            output_file,
            cairo_pie_file,
        } => {
            entry_run(
                request_file,
                rpc_url,
                dry_run_cairo_file,
                preprocessor_output_file,
                sound_run_cairo_file,
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
    save_fetch_keys_file: Option<PathBuf>,
    module_inputs: Vec<String>,
    rpc_url: Option<Url>,
    chain_id: Option<ChainId>,
    dry_run_cairo_file: Option<PathBuf>,
    preprocessor_output_file: Option<PathBuf>,
    sound_run_cairo_file: Option<PathBuf>,
    output_file: Option<PathBuf>,
    cairo_pie_file: Option<PathBuf>,
) -> Result<()> {
    let config = Config::init(
        rpc_url,
        chain_id,
        dry_run_cairo_file,
        sound_run_cairo_file,
        save_fetch_keys_file,
    )
    .await;
    let module_registry = ModuleRegistry::new();
    let module = module_registry
        .get_extended_module_from_class_source_string(class_hash, local_class_path, module_inputs)
        .await?;
    // TODO: for now, we only support one task if its a module
    let tasks = vec![TaskEnvelope::Module(module)];

    handle_running_tasks(
        config,
        tasks,
        preprocessor_output_file,
        output_file,
        cairo_pie_file,
    )
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn datalake_entry_run(
    aggregate_fn_id: AggregationFunction,
    aggregate_fn_ctx: Option<FunctionContext>,
    datalake: DataLakeCommands,
    rpc_url: Option<Url>,
    chain_id: Option<ChainId>,
    pre_processor_output: Option<PathBuf>,
    sound_run_cairo_file: Option<PathBuf>,
    output_file: Option<PathBuf>,
    cairo_pie_file: Option<PathBuf>,
) -> Result<()> {
    let config = Config::init(rpc_url, chain_id, None, sound_run_cairo_file, None).await;
    let parsed_datalake = match datalake {
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
    let tasks = vec![TaskEnvelope::DatalakeCompute(DatalakeCompute::new(
        parsed_datalake,
        Computation::new(aggregate_fn_id, aggregate_fn_ctx),
    ))];

    handle_running_tasks(
        config,
        tasks,
        pre_processor_output,
        output_file,
        cairo_pie_file,
    )
    .await?;
    Ok(())
}

pub async fn handle_running_tasks(
    config: &Config,
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

pub async fn entry_run(
    request_file: PathBuf,
    rpc_url: Option<Url>,
    dry_run_cairo_file: Option<PathBuf>,
    pre_processor_output_file: PathBuf,
    sound_run_cairo_file: Option<PathBuf>,
    output_file: Option<PathBuf>,
    cairo_pie_file: Option<PathBuf>,
) -> Result<()> {
    let request_context = fs::read_to_string(request_file).unwrap();
    let parsed: SubmitBatchQuery = serde_json::from_str(&request_context).unwrap();
    let config = Config::init(
        rpc_url,
        Some(parsed.destination_chain_id),
        dry_run_cairo_file,
        sound_run_cairo_file,
        None,
    )
    .await;
    let module_registry = ModuleRegistry::new();
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
        dry_run_program_path: PathBuf::from(&config.dry_run_program_path),
        provider_config: config.evm_provider.clone(),
        save_fetch_keys_file: config.save_fetch_keys_file.clone(),
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
        let processor = Processor::new(config.sound_run_program_path.clone());
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
