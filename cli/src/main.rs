use alloy_primitives::U256;
use anyhow::{bail, Result};
use hdp_primitives::datalake::{
    block_sampled::{AccountField, BlockSampledCollectionType, BlockSampledDatalake, HeaderField},
    datalake_type::DatalakeType,
    envelope::DatalakeEnvelope,
    transactions::{
        TransactionField, TransactionReceiptField, TransactionsCollectionType,
        TransactionsInBlockDatalake,
    },
};
use inquire::{error::InquireError, Select};
use std::{str::FromStr, sync::Arc, vec};
use tracing_subscriber::FmtSubscriber;

use clap::{Parser, Subcommand};
use hdp_core::{
    aggregate_fn::{integer::Operator, FunctionContext},
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
    /// New to the HDP CLI? Start here!
    Start,
    ///  Encode the task and datalake in batched format test purposes
    #[command(arg_required_else_help = true)]
    Encode {
        /// Decide if want to run evaluator as follow step or not (default: false)
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        allow_run: bool,

        /// The aggregate function id e.g. "sum", "min", "avg"
        aggregate_fn_id: String,
        /// Optional context for applying conditions on the aggregate function "count".
        /// Format: "{operator}.{value}" (e.g., "eq.100" for equality, "gt.100" for greater-than).
        /// Supported operators are in the [`Operator`] enum.
        aggregate_fn_ctx: Option<FunctionContext>,

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
    /// Return the compiled cairo file that works with integration test
    CompiledCairo {
        /// Path to save the compiled cairo json file
        #[arg(short, long)]
        output_path: String,
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
    TransactionsInBlock {
        /// target block number
        target_block: u64,
        /// Sampled property
        /// Fields from transaction: "chain_id", "gas_price"... etc
        /// Fields from transaction receipt: "cumulative_gas_used".. etc
        sampled_property: String,
        /// Increment number of transactions in the block
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
        decoded_result.datalakes,
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
        Commands::Start => {
            println!("Welcome to Herodotus Data Processor interactive CLI! 🛰️");
            println!(
                r"
                _   _   ____    ____  
                | | | | |  _ \  |  _ \ 
                | |_| | | | | | | |_) |
                |  _  | | |_| | |  __/ 
                |_| |_| |____/  |_|    
    "
            );

            let variants = DatalakeType::variants();
            let datalake_opts: Vec<&str> = variants.iter().map(AsRef::as_ref).collect();

            let ans: Result<&str, InquireError> =
                Select::new("Step 1. What's your datalake type?", datalake_opts).prompt();

            let datalake_envelope: DatalakeEnvelope = match ans {
                Ok(choice) => {
                    let datalake_type = DatalakeType::from_str(choice)?;
                    match datalake_type {
                        DatalakeType::BlockSampled => {
                            // ================== Block Sampled Datalake Fields ==================
                            // 1. Block range start
                            let block_range_start: u64 = inquire::Text::new("Block range start")
                                .with_help_message(
                                    "What is the block range start? (Enter to set default)",
                                )
                                .with_default("4952200")
                                .prompt()?
                                .parse()?;
                            // 2. Block range end
                            let block_range_end: u64 = inquire::Text::new("Block range end")
                                .with_help_message(
                                    "What is the block range end? (Enter to set default)",
                                )
                                .with_default("4952229")
                                .prompt()?
                                .parse()?;
                            // 3. Increment
                            let increment: u64 = inquire::Text::new("Increment")
                                .with_help_message(
                                    "How many blocks to skip in the range? (Enter to set default)",
                                )
                                .with_default("1")
                                .prompt()?
                                .parse()?;
                            // 4. Sampled Property
                            // 4.1. Block Sampled Collection Type
                            let variants: Vec<String> = BlockSampledCollectionType::variants();
                            let collection_opts: Vec<&str> =
                                variants.iter().map(AsRef::as_ref).collect();
                            let collection_ans: &str = Select::new(
                                "Sample Property: Select block sample type",
                                collection_opts,
                            )
                            .with_help_message("What type of block sample do you want to process?")
                            .prompt()?;
                            let collection_type =
                                BlockSampledCollectionType::from_str(collection_ans)?;
                            // 4.2. Detail Sampled Property
                            let sampled_property = match collection_type {
                                BlockSampledCollectionType::Header => {
                                    let variants: Vec<String> = HeaderField::variants();
                                    let header_opts: Vec<&str> =
                                        variants.iter().map(AsRef::as_ref).collect();
                                    let header_ans: &str =
                                        Select::new("Select detail header property", header_opts).with_help_message("What header property do you want to sample? (all properties are decodable from rlp encoded data)")
                                            .prompt()?;
                                    format!("header.{}", header_ans)
                                }
                                BlockSampledCollectionType::Account => {
                                    let address = inquire::Text::new("Enter target address")
                                        .with_help_message("Enter target address")
                                        .prompt()?;
                                    let variants: Vec<String> = AccountField::variants();
                                    let account_opts: Vec<&str> =
                                        variants.iter().map(AsRef::as_ref).collect();
                                    let account_ans: &str =
                                        Select::new("Select detail account property", account_opts)
                                        .with_help_message("What account property do you want to sample? (all properties are decodable from rlp encoded data)")
                                            .prompt()?;
                                    format!("account.{}.{}", address, account_ans)
                                }
                                BlockSampledCollectionType::Storage => {
                                    let address = inquire::Text::new("Enter target address")
                                        .with_help_message("Enter target address")
                                        .prompt()?;
                                    let storage_key =
                                        inquire::Text::new("Enter target storage key")
                                            .with_help_message("Enter the storage key")
                                            .prompt()?;
                                    format!("storage.{}.{}", address, storage_key)
                                }
                            };
                            let block_sampled_datalake = BlockSampledDatalake::new(
                                block_range_start,
                                block_range_end,
                                sampled_property,
                                increment,
                            )?;
                            DatalakeEnvelope::BlockSampled(block_sampled_datalake)
                        }
                        DatalakeType::TransactionsInBlock => {
                            let target_block: u64 = inquire::Text::new("Enter target block number")
                            .with_help_message(
                                "What block you target to get transactions? (Enter to set default)",
                            )
                            .with_default("4952200")
                            .prompt()?
                            .parse()?;
                            let increment: u64 = inquire::Text::new("Increment")
                                .with_help_message(
                                    "How many transactions to skip in the range? (Enter to set default)",
                                )
                                .with_default("1")
                                .prompt()?
                                .parse()?;
                            let variants = TransactionsCollectionType::variants();
                            let collection_opts: Vec<&str> =
                                variants.iter().map(AsRef::as_ref).collect();
                            let collection_ans: &str = Select::new(
                                "Sample Property: Select block sample type",
                                collection_opts,
                            )
                            .with_help_message("What type of block sample do you want to process?")
                            .prompt()?;
                            let collection_type =
                                TransactionsCollectionType::from_str(collection_ans)?;
                            let sampled_property = match collection_type {
                                TransactionsCollectionType::Transactions => {
                                    let variants: Vec<String> = TransactionField::variants();
                                    let transaction_opts: Vec<&str> =
                                        variants.iter().map(AsRef::as_ref).collect();
                                    let transaction_ans: &str =
                                        Select::new("Select detail transaction property", transaction_opts)
                                        .with_help_message("What transaction property do you want to sample? (all properties are decodable from rlp encoded data)")
                                            .prompt()?;
                                    format!("tx.{}", transaction_ans)
                                }
                                TransactionsCollectionType::TransactionReceipts => {
                                    let variants = TransactionReceiptField::variants();
                                    let transaction_receipt_opts: Vec<&str> =
                                        variants.iter().map(AsRef::as_ref).collect();
                                    let transaction_receipt_ans: &str =
                                        Select::new("Select detail transaction receipt property", transaction_receipt_opts)
                                        .with_help_message("What transaction receipt property do you want to sample? (all properties are decodable from rlp encoded data)")
                                            .prompt()?;
                                    format!("tx_receipt.{}", transaction_receipt_ans)
                                }
                            };
                            let transactions_datalake = TransactionsInBlockDatalake::new(
                                target_block,
                                sampled_property,
                                increment,
                            )?;
                            DatalakeEnvelope::Transactions(transactions_datalake)
                        }
                    }
                }
                Err(e) => {
                    error!("Error: {:?}", e);
                    bail!(e);
                }
            };

            let task_opts: Vec<&str> = vec!["AVG", "SUM", "MIN", "MAX", "COUNT"];

            let aggregate_fn_id = Select::new("Select the aggregation function", task_opts)
                .with_help_message(
                    "Step 2. What type of aggregation do you want to perform on the datalake?",
                )
                .prompt()?;

            let aggregate_fn_ctx = match aggregate_fn_id {
                "COUNT" => {
                    let operator_ans: String = Select::new(
                        "Select the COURNIF operator",
                        vec!["=", "!=", ">", ">=", "<", "<="],
                    )
                    .with_help_message("How would like to set opersation case?")
                    .prompt()?
                    .into();
                    let value_to_compare: String = inquire::Text::new("Enter the value to compare")
                        .with_help_message("Make sure to input Uint256 value")
                        .prompt()?;
                    Some(FunctionContext::new(
                        Operator::from_symbol(&operator_ans)?,
                        U256::from_str(&value_to_compare)?,
                    ))
                }
                _ => None,
            };

            let encoded_result = handle_encode_multiple(
                vec![ComputationalTask::new(aggregate_fn_id, aggregate_fn_ctx)],
                vec![datalake_envelope],
            )
            .await?;

            let allow_run: bool = inquire::Confirm::new("Do you want to run the evaluator?")
                .with_default(true)
                .prompt()?;
            if allow_run {
                let rpc_url: Option<String> = match inquire::Text::new("Enter RPC URL: ")
                    .with_help_message("Skip if you have it in your .env file")
                    .prompt()
                {
                    Ok(url) => match url.as_str() {
                        "" => None,
                        _ => Some(url),
                    },
                    Err(_) => None,
                };
                let chain_id: Option<u64> = match inquire::Text::new("Enter Chain ID: ")
                    .with_help_message("Skip if you have it in your .env file")
                    .prompt()
                {
                    Ok(chain_id) => match chain_id.as_str() {
                        "" => None,
                        _ => Some(chain_id.parse()?),
                    },
                    Err(_) => None,
                };
                let output_file: String = inquire::Text::new("Enter Output file path: ")
                    .with_default("output.json")
                    .prompt()?;
                let cairo_input: String = inquire::Text::new("Enter Cairo input file path:")
                    .with_default("input.json")
                    .prompt()?;

                handle_run(
                    Some(encoded_result.tasks),
                    Some(encoded_result.datalakes),
                    rpc_url,
                    chain_id,
                    Some(output_file),
                    Some(cairo_input),
                )
                .await?
            }
        }
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
                DataLakeCommands::TransactionsInBlock {
                    target_block,
                    sampled_property,
                    increment,
                } => {
                    let transactions_datalake = TransactionsInBlockDatalake::new(
                        target_block,
                        sampled_property,
                        increment,
                    )?;
                    DatalakeEnvelope::Transactions(transactions_datalake)
                }
            };

            let encoded_result = handle_encode_multiple(
                vec![ComputationalTask::new(&aggregate_fn_id, aggregate_fn_ctx)],
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
        Commands::CompiledCairo { output_path } => {
            // save the compiled cairo file to the output path
            let compiled_cairo = include_str!("../../compiled_cairo/hdp.json");
            std::fs::write(output_path, compiled_cairo)?;
        }
    }
    let duration_run = start_run.elapsed();
    info!("HDP Cli Finished in: {:?}", duration_run);
    Ok(())
}
