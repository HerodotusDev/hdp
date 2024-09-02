use std::{env, fs};

use crate::commands::process::ProcessArgs;
use crate::commands::run::RunArgs;
use crate::commands::run_datalake::DataLakeCommands;
use crate::{
    commands::{run_datalake::RunDatalakeArgs, run_module::RunModuleArgs, HDPCli, HDPCliCommands},
    interactive,
};
use anyhow::Result;
use clap::Parser;
use hdp::primitives::chain_id::ChainId;
use hdp::primitives::processed_types::cairo_format::query::ProcessorInput;
use hdp::primitives::request::{SubmitBatchQuery, Task};
use hdp::processor::{self, Processor};
use hdp::{
    hdp_run,
    preprocessor::module_registry::ModuleRegistry,
    primitives::task::{
        datalake::{
            block_sampled::BlockSampledDatalake, compute::Computation, envelope::DatalakeEnvelope,
            transactions::TransactionsInBlockDatalake, DatalakeCompute,
        },
        TaskEnvelope,
    },
};
use tracing::{debug, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub async fn hdp_cli_run() -> anyhow::Result<()> {
    let start_run = std::time::Instant::now();
    let cli = init_cli()?;
    match cli.command {
        HDPCliCommands::Start => {
            interactive::run_interactive().await?;
        }
        HDPCliCommands::RunDatalake(args) => {
            datalake_entry_run(args).await?;
        }
        HDPCliCommands::RunModule(args) => {
            module_entry_run(args).await?;
        }
        HDPCliCommands::Run(args) => {
            entry_run(args).await?;
        }
        HDPCliCommands::Process(args) => {
            process_entry_run(args).await?;
        }
    }
    let duration_run = start_run.elapsed();
    info!("HDP Cli Finished in: {:?}", duration_run);
    Ok(())
}

/// Initialize the CLI
fn init_cli() -> Result<HDPCli> {
    dotenv::dotenv().ok();
    let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::new(&rust_log))
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
    debug!("running on log level: {}", rust_log);
    let cli = HDPCli::parse();
    Ok(cli)
}

pub async fn process_entry_run(args: ProcessArgs) -> Result<()> {
    let config = processor::HdpProcessorConfig::init(
        args.sound_run_cairo_file,
        args.input_file,
        args.cairo_pie_file,
    );
    let input_string = fs::read_to_string(&config.input_file)?;
    let preprocessor_result: ProcessorInput = serde_json::from_str(&input_string)
        .expect("Input file is not valid. Check if format is correct");

    info!(
        "processing the data from {}... ",
        &config.input_file.display()
    );
    let processor = Processor::new(config.sound_run_program_path.clone());
    processor
        .process(preprocessor_result, &config.cairo_pie_file)
        .await?;

    info!(
        "finished processing the data, saved pie file in {}",
        &config.cairo_pie_file.display()
    );

    Ok(())
}

pub async fn module_entry_run(args: RunModuleArgs) -> Result<()> {
    let config = hdp_run::HdpRunConfig::init(
        args.rpc_url,
        args.chain_id,
        args.dry_run_cairo_file,
        args.sound_run_cairo_file,
        args.program_input_file,
        args.cairo_format,
        args.save_fetch_keys_file,
        args.batch_proof_file,
        args.cairo_pie_file,
    );
    let module_registry = ModuleRegistry::new();
    let module = module_registry
        .get_extended_module_from_class_source_string(
            args.program_hash,
            args.local_class_path,
            args.module_inputs,
        )
        .await?;
    // TODO: for now, we only support one task if its a module
    let tasks = vec![TaskEnvelope::Module(module)];

    hdp::run(&config, tasks).await?;
    Ok(())
}

pub async fn datalake_entry_run(args: RunDatalakeArgs) -> Result<()> {
    let config = hdp_run::HdpRunConfig::init(
        args.rpc_url,
        args.chain_id,
        None,
        args.sound_run_cairo_file,
        args.program_input_file,
        args.cairo_format,
        None,
        args.batch_proof_file,
        args.cairo_pie_file,
    );
    let parsed_datalake = match args.datalake {
        DataLakeCommands::BlockSampled {
            block_range_start,
            block_range_end,
            sampled_property,
            increment,
        } => DatalakeEnvelope::BlockSampled(BlockSampledDatalake::new(
            ChainId::EthereumSepolia,
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
            ChainId::EthereumSepolia,
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
        Computation::new(args.aggregate_fn_id, args.aggregate_fn_ctx),
    ))];

    hdp::run(&config, tasks).await?;
    Ok(())
}

pub async fn entry_run(args: RunArgs) -> Result<()> {
    let request_context =
        fs::read_to_string(args.request_file).expect("No request file exist in the path");
    let parsed: SubmitBatchQuery = serde_json::from_str(&request_context)
        .expect("Invalid format of request. Cannot parse it.");
    let config = hdp_run::HdpRunConfig::init(
        args.rpc_url,
        Some(parsed.destination_chain_id),
        args.dry_run_cairo_file,
        args.sound_run_cairo_file,
        args.program_input_file,
        args.cairo_format,
        None,
        args.batch_proof_file,
        args.cairo_pie_file,
    );
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
    hdp::run(&config, task_envelopes).await?;
    Ok(())
}
