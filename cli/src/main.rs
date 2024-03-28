use anyhow::{bail, Result};
use hdp_primitives::datalake::{
    block_sampled::BlockSampledDatalake, envelope::DatalakeEnvelope,
    transactions::TransactionsDatalake,
};
use std::{sync::Arc, vec};
use tracing_subscriber::FmtSubscriber;

use clap::{Parser, Subcommand};
use hdp_core::{
    codec::{
        datalake_decoder, datalakes_decoder, datalakes_encoder, task_decoder, tasks_decoder,
        tasks_encoder,
    },
    config::Config,
    evaluator::evaluator,
    task::ComputationalTask,
};

use hdp_provider::evm::AbstractProvider;

use tokio::sync::RwLock;
use tracing::{debug, error, info, Level};

/// Simple Herodotus Data Processor CLI to handle tasks and datalakes
#[derive(Debug, Parser)]
#[command(name = "hdp")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    ///  Encode the task and datalake in batched format test purposes
    #[command(arg_required_else_help = true)]
    Encode {
        /// Decide if want to run evaluator as follow step or not (default: false)
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        allow_run: bool,

        /// The aggregate function id e.g. "sum", "min", "avg"
        aggregate_fn_id: String,
        /// The aggregate function context. It depends on the aggregate function
        aggregate_fn_ctx: Option<String>,
        #[command(subcommand)]
        command: DataLakeCommands,

        /// The RPC URL to fetch the data
        rpc_url: Option<String>,

        /// The chain id to fetch the data
        chain_id: Option<u64>,

        /// Path to the file to save the output result
        #[arg(short, long)]
        output_file: Option<String>,
        /// Path to the file to save the input.json in cairo format
        #[arg(short, long)]
        cairo_input: Option<String>,
    },
    /// Decode batch tasks and datalakes
    ///
    /// Note: Batch tasks and datalakes should be encoded in bytes[] format
    #[command(arg_required_else_help = true)]
    Decode {
        /// Batched tasks bytes
        tasks: String,
        /// Batched datalakes bytes
        datalakes: String,
    },

    /// Decode one task and one datalake (not batched format)
    #[command(arg_required_else_help = true)]
    DecodeOne { task: String, datalake: String },
    /// Run the evaluator
    Run {
        /// Batched tasks bytes
        tasks: Option<String>,
        /// Batched datalakes bytes
        datalakes: Option<String>,
        /// The RPC URL to fetch the data
        rpc_url: Option<String>,
        /// The chain id to fetch the data
        chain_id: Option<u64>,
        /// Path to the file to save the output result
        #[arg(short, long)]
        output_file: Option<String>,

        /// Path to the file to save the input.json in cairo format
        #[arg(short, long)]
        cairo_input: Option<String>,
    },
}

#[derive(Subcommand, Clone, Debug, PartialEq, Eq)]
enum DataLakeCommands {
    ///  Encode the block sampled data lake for test purposes
    #[command(arg_required_else_help = true)]
    #[command(short_flag = 'b')]
    BlockSampled {
        /// Block number range start
        block_range_start: u64,
        /// Block number range end
        block_range_end: u64,
        /// Sampled property e.g. "header.number", "account.0xaccount.balance", "storage.0xcontract.0xstoragekey"
        sampled_property: String,
        /// Increment number of given range blocks
        #[arg(default_value_t = 1)]
        increment: u64,
    },

    ///  Encode the transactions data lake for test purposes
    #[command(arg_required_else_help = true)]
    #[command(short_flag = 't')]
    Transactions {
        /// Sender address of the transactions
        address: String,
        /// From nonce
        from_nonce: u64,
        /// To nonce
        to_nonce: u64,
        /// Sampled property
        /// Fields from transaction: "chain_id", "gas_price"... etc
        /// Fields from transaction receipt: "cumulative_gas_used".. etc
        sampled_property: String,
        /// Increment number of given range nonce
        #[arg(default_value_t = 1)]
        increment: u64,
    },
}

struct DecodeMultipleResult {
    tasks: Vec<ComputationalTask>,
    datalakes: Vec<DatalakeEnvelope>,
}

struct EncodeMultipleResult {
    tasks: String,
    datalakes: String,
}

async fn handle_decode_multiple(datalakes: String, tasks: String) -> Result<DecodeMultipleResult> {
    let datalakes = datalakes_decoder(datalakes.clone())?;
    info!("datalakes: {:#?}", datalakes);

    let tasks = tasks_decoder(tasks)?;
    info!("tasks: {:#?}", tasks);

    if tasks.len() != datalakes.len() {
        error!("Tasks and datalakes must have the same length");
        bail!("Tasks and datalakes must have the same length");
    } else {
        Ok(DecodeMultipleResult { tasks, datalakes })
    }
}

async fn handle_encode_multiple(
    tasks: Vec<ComputationalTask>,
    datalakes: Vec<DatalakeEnvelope>,
) -> Result<EncodeMultipleResult> {
    let encoded_datalakes = datalakes_encoder(datalakes)?;
    info!("Encoded datalakes: {}", encoded_datalakes);

    let encoded_tasks = tasks_encoder(tasks)?;
    info!("Encoded tasks: {}", encoded_tasks);

    Ok(EncodeMultipleResult {
        tasks: encoded_tasks,
        datalakes: encoded_datalakes,
    })
}

async fn handle_run(
    tasks: Option<String>,
    datalakes: Option<String>,
    rpc_url: Option<String>,
    chain_id: Option<u64>,
    output_file: Option<String>,
    cairo_input: Option<String>,
) -> Result<()> {
    let config = Config::init(rpc_url, datalakes, tasks, chain_id).await;
    let provider = AbstractProvider::new(&config.rpc_url, config.chain_id);

    let decoded_result =
        handle_decode_multiple(config.datalakes.clone(), config.tasks.clone()).await?;

    match evaluator(
        decoded_result.tasks,
        Some(decoded_result.datalakes),
        Arc::new(RwLock::new(provider)),
    )
    .await
    {
        Ok(res) => {
            debug!("Result: {:#?}", res);

            if let Some(output_file) = output_file {
                res.save_to_file(&output_file, false)?;
                info!("Output file saved to: {}", output_file);
            }
            if let Some(cairo_input) = cairo_input {
                res.save_to_file(&cairo_input, true)?;
                info!("Cairo input file saved to: {}", cairo_input);
            }

            Ok(())
        }
        Err(e) => {
            error!("Error: {:?}", e);
            bail!(e);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let start_run = std::time::Instant::now();
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    let cli = Cli::parse();
    dotenv::dotenv().ok();
    match cli.command {
        Commands::Encode {
            allow_run,
            rpc_url,
            chain_id,
            output_file,
            cairo_input,
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
                } => {
                    let block_sampled_datalake = BlockSampledDatalake::new(
                        block_range_start,
                        block_range_end,
                        sampled_property,
                        increment,
                    )?;
                    DatalakeEnvelope::BlockSampled(block_sampled_datalake)
                }
                DataLakeCommands::Transactions {
                    address,
                    from_nonce,
                    to_nonce,
                    sampled_property,
                    increment,
                } => {
                    let transactions_datalake = TransactionsDatalake::new(
                        address,
                        from_nonce,
                        to_nonce,
                        sampled_property,
                        increment,
                    )?;
                    DatalakeEnvelope::Transactions(transactions_datalake)
                }
            };

            let encoded_result = handle_encode_multiple(
                vec![ComputationalTask::new(
                    None,
                    aggregate_fn_id,
                    aggregate_fn_ctx,
                )],
                vec![datalake],
            )
            .await?;
            // if allow_run is true, then run the evaluator
            if allow_run {
                handle_run(
                    Some(encoded_result.tasks),
                    Some(encoded_result.datalakes),
                    rpc_url,
                    chain_id,
                    output_file,
                    cairo_input,
                )
                .await?
            }
        }
        Commands::Decode { tasks, datalakes } => {
            handle_decode_multiple(datalakes, tasks).await?;
        }
        Commands::DecodeOne { task, datalake } => {
            let task = task_decoder(task)?;
            let datalake = datalake_decoder(datalake)?;

            info!("task: \n{:?}\n", task);
            info!("datalake: \n{:?}\n", datalake);
        }
        Commands::Run {
            tasks,
            datalakes,
            rpc_url,
            chain_id,
            output_file,
            cairo_input,
        } => {
            handle_run(
                tasks,
                datalakes,
                rpc_url,
                chain_id,
                output_file,
                cairo_input,
            )
            .await?
        }
    }
    let duration_run = start_run.elapsed();
    info!("HDP Cli Finished in: {:?}", duration_run);
    Ok(())
}
