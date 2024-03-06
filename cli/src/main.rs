use std::{sync::Arc, vec};

use clap::{Parser, Subcommand};
use common::{
    codec::{
        datalake_decoder, datalakes_decoder, datalakes_encoder, task_decoder, tasks_decoder,
        tasks_encoder,
    },
    config::Config,
    datalake::Datalake,
    fetcher::AbstractFetcher,
};

use evaluator::evaluator;
use tokio::sync::RwLock;

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
        /// The aggregate function id e.g. "sum", "min", "avg"
        aggregate_fn_id: String,
        /// The aggregate function context. It depends on the aggregate function
        aggregate_fn_ctx: Option<String>,
        #[command(subcommand)]
        command: DataLakeCommands,
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
        tasks: Option<String>,
        datalakes: Option<String>,
        rpc_url: Option<String>,
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
}

#[tokio::main]
async fn main() {
    let start = std::time::Instant::now();
    let cli = Cli::parse();
    dotenv::dotenv().ok();
    match cli.command {
        Commands::Encode {
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
                    let block_sampled_datalake =
                        common::datalake::block_sampled::BlockSampledDatalake::new(
                            block_range_start,
                            block_range_end,
                            sampled_property,
                            increment,
                        );
                    Datalake::BlockSampled(block_sampled_datalake)
                }
            };
            println!("Original datalake: \n{:?}\n", datalake);
            let encoded_datalake = datalakes_encoder(vec![datalake]).unwrap();
            println!("Encoded datalake: \n{}\n", encoded_datalake);
            let tasks =
                common::task::ComputationalTask::new(None, aggregate_fn_id, aggregate_fn_ctx);
            println!("Original task: \n{:?}\n", tasks);
            let encoded_task = tasks_encoder(vec![tasks]).unwrap();
            println!("Encoded task: \n{}\n", encoded_task);
        }
        Commands::Decode { tasks, datalakes } => {
            let datalakes = datalakes_decoder(datalakes.clone()).unwrap();
            println!("datalakes: \n{:?}\n", datalakes);

            let tasks = tasks_decoder(tasks).unwrap();
            println!("tasks: \n{:?}\n", tasks);

            if tasks.len() != datalakes.len() {
                panic!("Tasks and datalakes must have the same length");
            }
        }
        Commands::DecodeOne { task, datalake } => {
            let task = task_decoder(task).unwrap();
            let datalake = datalake_decoder(datalake).unwrap();

            println!("task: \n{:?}\n", task);
            println!("datalake: \n{:?}\n", datalake);
        }
        Commands::Run {
            tasks,
            datalakes,
            rpc_url,
            output_file,
            cairo_input,
        } => {
            let config = Config::init(rpc_url, datalakes, tasks).await;
            let abstract_fetcher = AbstractFetcher::new(config.rpc_url.clone());
            let tasks = tasks_decoder(config.tasks.clone()).unwrap();
            let datalakes = datalakes_decoder(config.datalakes.clone()).unwrap();

            println!("tasks: \n{:?}\n", tasks);
            println!("datalakes: \n{:?}\n", datalakes);

            if tasks.len() != datalakes.len() {
                panic!("Tasks and datalakes must have the same length");
            }

            let res = evaluator(
                tasks,
                Some(datalakes),
                Arc::new(RwLock::new(abstract_fetcher)),
            )
            .await
            .unwrap();

            let duration = start.elapsed();
            println!("Time elapsed in main() is: {:?}", duration);

            if let Some(output_file) = output_file {
                res.save_to_file(&output_file, false).unwrap();
            }
            if let Some(cairo_input) = cairo_input {
                res.save_to_file(&cairo_input, true).unwrap();
            }
        }
    }
}
