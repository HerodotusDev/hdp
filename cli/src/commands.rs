use alloy::{
    hex,
    primitives::{BlockNumber, Bytes, ChainId, TxIndex},
    transports::http::reqwest::Url,
};
use clap::{command, Parser, Subcommand};
use hdp_primitives::{
    aggregate_fn::{AggregationFunction, FunctionContext},
    task::datalake::{
        block_sampled::BlockSampledCollection,
        transactions::{IncludedTypes, TransactionsCollection},
    },
};
use std::path::PathBuf;

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
    /// Encode the compute and datalake in batch and allow to proceed
    #[command(arg_required_else_help = true)]
    Encode {
        /// Decide to run processor. (default: false)
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        allow_process: bool,

        /// The aggregate function id e.g. "sum", "min", "avg"
        aggregate_fn_id: AggregationFunction,
        /// Optional context for applying conditions on the aggregate function "count".
        /// Format: "{operator}.{value}" (e.g., "eq.100" for equality, "gt.100" for greater-than).
        /// Supported operators are in the [`Operator`] enum.
        aggregate_fn_ctx: Option<FunctionContext>,

        #[command(subcommand)]
        command: DataLakeCommands,

        /// The RPC URL to fetch the datalake
        rpc_url: Option<Url>,

        /// The chain id to fetch the datalake
        chain_id: Option<ChainId>,

        /// Path to save output file after pre-process, this is the input file for processor
        ///
        /// This will trigger pre-processing step
        #[arg(short, long)]
        cairo_input: Option<PathBuf>,

        /// Path to save output file after process
        ///
        /// This will trigger processing(=pie generation) step
        #[arg(short, long, requires("cairo_input"))]
        output_file: Option<PathBuf>,

        /// Path to save pie file
        ///
        /// This will trigger processing(=pie generation) step
        #[arg(short, long, requires("cairo_input"))]
        pie_file: Option<PathBuf>,
    },
    /// Decode batch computes and datalakes
    ///
    /// Note: Batch computes and datalakes should be encoded in bytes[] format
    #[command(arg_required_else_help = true)]
    Decode {
        /// Batched computes bytes
        #[arg(value_parser = parse_bytes)]
        tasks: Bytes,
        /// Batched datalakes bytes
        #[arg(value_parser = parse_bytes)]
        datalakes: Bytes,
    },

    /// Decode one compute and one datalake (not batched format)
    #[command(arg_required_else_help = true)]
    DecodeOne {
        #[arg(value_parser = parse_bytes)]
        task: Bytes,
        #[arg(value_parser = parse_bytes)]
        datalake: Bytes,
    },
    /// Run from encoded compute and datalake. Usefull for request batch tasks.
    #[command(arg_required_else_help = true)]
    Run {
        /// Batched computes bytes
        #[arg(value_parser = parse_bytes)]
        tasks: Option<Bytes>,
        /// Batched datalakes bytes
        #[arg(value_parser = parse_bytes)]
        datalakes: Option<Bytes>,
        /// The RPC URL to fetch the data
        rpc_url: Option<Url>,
        /// The chain id to fetch the data
        chain_id: Option<ChainId>,

        /// Path to save output file after pre-process, this is the input file for processor
        ///
        /// This will trigger pre-processing step
        #[arg(short, long)]
        cairo_input: Option<PathBuf>,

        /// Path to save output file after process
        ///
        /// This will trigger processing(=pie generation) step
        #[arg(short, long, requires("cairo_input"))]
        output_file: Option<PathBuf>,

        /// Path to save pie file
        ///
        /// This will trigger processing(=pie generation) step
        #[arg(short, long, requires("cairo_input"))]
        pie_file: Option<PathBuf>,
    },

    /// Local run module with either class hash deployed on starknet or local class path
    #[command(arg_required_else_help = true)]
    LocalRunModule {
        /// Input field elements for the module contract.
        /// The input field elements should be separated by comma.
        ///
        /// e.g. "0x1234,0xabcd"
        #[arg(required = true, use_value_delimiter = true)]
        module_inputs: Vec<String>,

        /// Class hash of the module that deployed on starknet.
        /// This will trigger fetching the class from the starknet.
        ///
        /// (Note: either class_hash or local_class_path should be provided)
        #[arg(long, group = "class_source")]
        class_hash: Option<String>,

        /// Local path of the contract class file.
        /// Make sure to have structure match with [CasmContractClass](https://github.com/starkware-libs/cairo/blob/53f7a0d26d5c8a99a8ad6ba07207a762678f2931/crates/cairo-lang-starknet-classes/src/casm_contract_class.rs)
        ///
        /// (Note: either class_hash or local_class_path should be provided)
        #[arg(long, group = "class_source")]
        local_class_path: Option<PathBuf>,

        /// The RPC URL to fetch the data.
        ///
        /// Can be overwritten by `RPC_URL` environment variable.
        #[arg(long)]
        rpc_url: Option<Url>,

        /// The chain id to fetch the data.
        ///
        /// Can be overwritten by `CHAIN_ID` environment variable
        #[arg(long)]
        chain_id: Option<ChainId>,

        /// Module registry starknet rpc url, This is used to fetch the class from the module registry
        ///
        /// (Note: This is only used when the class is provided by `class_hash`)
        ///
        /// Can be overwritten by `MODULE_REGISTRY_RPC_URL` environment variable
        #[arg(long, requires("class_hash"))]
        module_registry_rpc_url: Option<Url>,

        /// Path to save output file after pre-process, this is the input file for processor
        ///
        /// This will trigger pre-processing step
        #[arg(short, long)]
        cairo_input: Option<PathBuf>,

        /// Path to save output file after process
        ///
        /// This will trigger processing(=pie generation) step
        #[arg(short, long, requires("cairo_input"))]
        output_file: Option<PathBuf>,

        /// Path to save pie file
        ///
        /// This will trigger processing(=pie generation) step
        #[arg(short, long, requires("cairo_input"))]
        pie_file: Option<PathBuf>,
    },
}

#[derive(Subcommand, Clone, Debug, PartialEq, Eq)]
pub enum DataLakeCommands {
    #[command(arg_required_else_help = true)]
    #[command(short_flag = 'b')]
    BlockSampled {
        /// Block number range start (inclusive)
        block_range_start: BlockNumber,
        /// Block number range end (inclusive)
        block_range_end: BlockNumber,
        /// Sampled property e.g. "header.number", "account.0xaccount.balance", "storage.0xcontract.0xstoragekey"
        sampled_property: BlockSampledCollection,
        /// Increment number of given range blocks
        #[arg(default_value_t = 1)]
        increment: u64,
    },

    #[command(arg_required_else_help = true)]
    #[command(short_flag = 't')]
    TransactionsInBlock {
        /// Target block number
        target_block: BlockNumber,
        /// Sampled property
        /// Fields from transaction: "chain_id", "gas_price"... etc
        /// Fields from transaction receipt: "cumulative_gas_used".. etc
        sampled_property: TransactionsCollection,
        /// Start index of transactions range (inclusive)
        start_index: TxIndex,
        /// End index of transactions range (exclusive)
        end_index: TxIndex,
        /// Increment number of transaction indexes in the block
        increment: u64,
        /// Filter out the specific type of Txs
        /// Each byte represents a type of transaction to be included in the datalake
        /// e.g 1,0,1,0 -> include legacy, exclude eip2930, include eip1559, exclude eip4844
        included_types: IncludedTypes,
    },
}

/// Parse bytes from hex string
fn parse_bytes(arg: &str) -> Result<Bytes, String> {
    hex::decode(arg)
        .map(Bytes::from)
        .map_err(|e| format!("Failed to parse bytes: {}", e))
}
