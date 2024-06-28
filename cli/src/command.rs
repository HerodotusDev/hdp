use std::path::PathBuf;

use alloy::{
    hex,
    primitives::{BlockNumber, Bytes, ChainId, TxIndex},
    transports::http::reqwest::Url,
};
use clap::{command, Parser, Subcommand};
use hdp_primitives::{
    aggregate_fn::{AggregationFunction, FunctionContext},
    datalake::{
        block_sampled::BlockSampledCollection,
        transactions::{IncludedTypes, TransactionsCollection},
    },
};

/// Parse bytes from hex string
fn parse_bytes(arg: &str) -> Result<Bytes, String> {
    hex::decode(arg)
        .map(Bytes::from)
        .map_err(|e| format!("Failed to parse bytes: {}", e))
}

/// Simple Herodotus Data Processor CLI to handle tasks and datalakes
#[derive(Debug, Parser)]
#[command(name = "hdp")]
#[command(version, about, long_about = None)]
pub struct HDPCli {
    #[command(subcommand)]
    pub command: HDPCliCommands,
}

#[derive(Debug, Subcommand)]
pub enum HDPCliCommands {
    /// New to the HDP CLI? Start here!
    Start,
    ///  Encode the task and datalake in batched format test purposes
    #[command(arg_required_else_help = true)]
    Encode {
        /// Decide if want to run evaluator as follow step or not (default: false)
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        allow_run: bool,

        /// The aggregate function id e.g. "sum", "min", "avg"
        aggregate_fn_id: AggregationFunction,
        /// Optional context for applying conditions on the aggregate function "count".
        /// Format: "{operator}.{value}" (e.g., "eq.100" for equality, "gt.100" for greater-than).
        /// Supported operators are in the [`Operator`] enum.
        aggregate_fn_ctx: Option<FunctionContext>,

        #[command(subcommand)]
        command: DataLakeCommands,

        /// The RPC URL to fetch the data
        rpc_url: Option<Url>,

        /// The chain id to fetch the data
        chain_id: Option<ChainId>,

        /// Path to the file to save the input.json in cairo format
        #[arg(short, long)]
        cairo_input: Option<PathBuf>,

        /// Path to the file to save the output result
        #[arg(short, long, requires("cairo_input"))]
        output_file: Option<PathBuf>,

        /// Path to pie file
        #[arg(short, long, requires("cairo_input"))]
        pie_file: Option<PathBuf>,
    },
    /// Decode batch tasks and datalakes
    ///
    /// Note: Batch tasks and datalakes should be encoded in bytes[] format
    #[command(arg_required_else_help = true)]
    Decode {
        /// Batched tasks bytes
        #[arg(value_parser = parse_bytes)]
        tasks: Bytes,
        /// Batched datalakes bytes
        #[arg(value_parser = parse_bytes)]
        datalakes: Bytes,
    },

    /// Decode one task and one datalake (not batched format)
    #[command(arg_required_else_help = true)]
    DecodeOne {
        #[arg(value_parser = parse_bytes)]
        task: Bytes,
        #[arg(value_parser = parse_bytes)]
        datalake: Bytes,
    },
    /// Run the evaluator
    Run {
        /// Batched tasks bytes
        #[arg(value_parser = parse_bytes)]
        tasks: Option<Bytes>,
        /// Batched datalakes bytes
        #[arg(value_parser = parse_bytes)]
        datalakes: Option<Bytes>,
        /// The RPC URL to fetch the data
        rpc_url: Option<Url>,
        /// The chain id to fetch the data
        chain_id: Option<ChainId>,

        /// Path to the file to save the input.json in cairo format
        #[arg(short, long)]
        cairo_input: Option<PathBuf>,

        /// Path to the file to save the output result
        #[arg(short, long, requires("cairo_input"))]
        output_file: Option<PathBuf>,

        /// Path to pie file
        #[arg(short, long, requires("cairo_input"))]
        pie_file: Option<PathBuf>,
    },
}

#[derive(Subcommand, Clone, Debug, PartialEq, Eq)]
pub enum DataLakeCommands {
    ///  Encode the block sampled data lake for test purposes
    #[command(arg_required_else_help = true)]
    #[command(short_flag = 'b')]
    BlockSampled {
        /// Block number range start
        block_range_start: BlockNumber,
        /// Block number range end
        block_range_end: BlockNumber,
        /// Sampled property e.g. "header.number", "account.0xaccount.balance", "storage.0xcontract.0xstoragekey"
        sampled_property: BlockSampledCollection,
        /// Increment number of given range blocks
        #[arg(default_value_t = 1)]
        increment: u64,
    },

    /// Encode the transactions data lake for test purposes
    #[command(arg_required_else_help = true)]
    #[command(short_flag = 't')]
    TransactionsInBlock {
        /// Target block number
        target_block: BlockNumber,
        /// Sampled property
        /// Fields from transaction: "chain_id", "gas_price"... etc
        /// Fields from transaction receipt: "cumulative_gas_used".. etc
        sampled_property: TransactionsCollection,
        /// Start index of transactions range
        start_index: TxIndex,
        /// End index of transactions range
        end_index: TxIndex,
        /// Increment number of transactions in the block
        increment: u64,
        /// Filter out the specific type of Txs
        #[arg(value_delimiter = ',')]
        included_types: IncludedTypes,
    },
}
