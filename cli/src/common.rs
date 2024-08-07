use alloy::{primitives::ChainId, transports::http::reqwest::Url};
use anyhow::Result;
use hdp::hdp_run;
use hdp::preprocessor::module_registry::ModuleRegistry;
use hdp::primitives::{
    aggregate_fn::{AggregationFunction, FunctionContext},
    task::{
        datalake::{
            block_sampled::BlockSampledDatalake, compute::Computation, envelope::DatalakeEnvelope,
            transactions::TransactionsInBlockDatalake, DatalakeCompute,
        },
        TaskEnvelope,
    },
};
use std::{env, fs, path::PathBuf};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use clap::Parser;

use tracing::{debug, info};

use crate::{
    commands::{DataLakeCommands, HDPCli, HDPCliCommands},
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
    let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::new(&rust_log))
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
    debug!("running on log level: {}", rust_log);
    let cli = HDPCli::parse();
    dotenv::dotenv().ok();
    Ok(cli)
}

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
    let config = hdp_run::HdpRunConfig::init(
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

    hdp_run(
        &config,
        tasks,
        preprocessor_output_file,
        output_file,
        cairo_pie_file,
    )
    .await?;
    Ok(())
}

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
    let config =
        hdp_run::HdpRunConfig::init(rpc_url, chain_id, None, sound_run_cairo_file, None).await;
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

    hdp_run(
        &config,
        tasks,
        pre_processor_output,
        output_file,
        cairo_pie_file,
    )
    .await?;
    Ok(())
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
    let request_context =
        fs::read_to_string(request_file).expect("No request file exist in the path");
    let parsed: SubmitBatchQuery = serde_json::from_str(&request_context)
        .expect("Invalid format of request. Cannot parse it.");
    let config = hdp_run::HdpRunConfig::init(
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
    hdp_run(
        &config,
        task_envelopes,
        Some(pre_processor_output_file),
        output_file,
        cairo_pie_file,
    )
    .await?;
    Ok(())
}
