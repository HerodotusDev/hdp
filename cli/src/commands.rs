use alloy::{
    primitives::{BlockNumber, ChainId, TxIndex},
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
    /// Run single datalake compute
    #[command(arg_required_else_help = true)]
    RunDatalake {
        /// The aggregate function id e.g. "sum", "min", "avg"
        aggregate_fn_id: AggregationFunction,
        /// Optional context for applying conditions on the aggregate function "count".
        /// Format: "{operator}.{value}" (e.g., "eq.100" for equality, "gt.100" for greater-than).
        /// Supported operators are in the [`Operator`] enum.
        aggregate_fn_ctx: Option<FunctionContext>,

        #[command(subcommand)]
        datalake: DataLakeCommands,

        /// The RPC URL to fetch the datalake
        rpc_url: Option<Url>,

        /// The chain id to fetch the datalake
        chain_id: Option<ChainId>,

        /// Path to save output file after pre-processing.
        ///
        /// This will trigger pre-processing step
        #[arg(short, long)]
        preprocessor_output_file: Option<PathBuf>,

        /// hdp cairo compiled program. main entry point
        #[arg(long)]
        sound_run_cairo_file: Option<PathBuf>,

        /// Path to save output file after process
        ///
        /// This will trigger processing(=pie generation) step
        #[arg(short, long, requires("preprocessor_output_file"))]
        output_file: Option<PathBuf>,

        /// Path to save pie file
        ///
        /// This will trigger processing(=pie generation) step
        #[arg(short, long, requires("preprocessor_output_file"))]
        cairo_pie_file: Option<PathBuf>,
    },

    /// Run single module with either class hash deployed on starknet or local class path
    #[command(arg_required_else_help = true)]
    RunModule {
        /// Input field elements for the module contract.
        /// The input field elements should be separated by comma.
        ///
        /// e.g. "0x1234,0xabcd"
        #[arg(required = true, use_value_delimiter = true)]
        module_inputs: Vec<String>,

        #[arg(long, group = "class_source")]
        program_hash: Option<String>,

        /// Local path of the contract class file.
        /// Make sure to have structure match with [CasmContractClass](https://github.com/starkware-libs/cairo/blob/53f7a0d26d5c8a99a8ad6ba07207a762678f2931/crates/cairo-lang-starknet-classes/src/casm_contract_class.rs)
        ///
        /// (Note: either class_hash or local_class_path should be provided)
        #[arg(long, group = "class_source")]
        local_class_path: Option<PathBuf>,

        /// optionally can save keys for module task to file
        save_fetch_keys_file: Option<PathBuf>,

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

        /// dry run contract bootloader program.
        /// only used for module task
        #[arg(long)]
        dry_run_cairo_file: Option<PathBuf>,

        /// Path to save output file after pre-processing.
        ///
        /// This will trigger pre-processing step
        #[arg(short, long)]
        preprocessor_output_file: Option<PathBuf>,

        /// hdp cairo compiled program. main entry point
        #[arg(long)]
        sound_run_cairo_file: Option<PathBuf>,

        /// Path to save output file after process
        ///
        /// This will trigger processing(=pie generation) step
        #[arg(short, long, requires("preprocessor_output_file"))]
        output_file: Option<PathBuf>,

        /// Path to save pie file
        ///
        /// This will trigger processing(=pie generation) step
        #[arg(short, long, requires("preprocessor_output_file"))]
        cairo_pie_file: Option<PathBuf>,
    },
    /// Run batch of tasks base on request json file
    #[command(arg_required_else_help = true)]
    Run {
        /// Pass request as json file
        #[arg(short, long)]
        request_file: PathBuf,

        /// The RPC URL to fetch the data.
        ///
        /// Can be overwritten by `RPC_URL` environment variable.
        #[arg(long)]
        rpc_url: Option<Url>,

        /// dry run contract bootloader program.
        /// only used for module task
        #[arg(long)]
        dry_run_cairo_file: Option<PathBuf>,

        /// Path to save output file after pre-processing.
        #[arg(short, long)]
        preprocessor_output_file: PathBuf,

        /// hdp cairo compiled program. main entry point
        #[arg(long)]
        sound_run_cairo_file: Option<PathBuf>,

        /// Path to save output file after process
        ///
        /// This will trigger processing(=pie generation) step
        #[arg(short, long, requires("preprocessor_output_file"))]
        output_file: Option<PathBuf>,

        /// Path to save pie file
        ///
        /// This will trigger processing(=pie generation) step
        #[arg(short, long, requires("preprocessor_output_file"))]
        cairo_pie_file: Option<PathBuf>,
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
