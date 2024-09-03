use std::path::PathBuf;

use alloy::primitives::{BlockNumber, TxIndex};
use clap::{arg, command, Parser, Subcommand};
use hdp::primitives::{
    aggregate_fn::{AggregationFunction, FunctionContext},
    task::datalake::{
        block_sampled::BlockSampledCollection,
        transactions::{IncludedTypes, TransactionsCollection},
    },
    ChainId,
};

#[derive(Parser, Debug)]
pub struct RunDatalakeArgs {
    /// The aggregate function id e.g. "sum", "min", "avg"
    pub aggregate_fn_id: AggregationFunction,
    /// Optional context for applying conditions on the aggregate function "count".
    /// Format: "{operator}.{value}" (e.g., "eq.100" for equality, "gt.100" for greater-than).
    /// Supported operators are in the [`Operator`] enum.
    pub aggregate_fn_ctx: Option<FunctionContext>,

    #[command(subcommand)]
    pub datalake: DataLakeCommands,

    /// Path to save program input file after pre-processing.
    ///
    /// This will be input data for cairo program
    #[arg(short, long)]
    pub program_input_file: PathBuf,

    /// Set this boolean to true to generate cairo format program_input_file
    ///
    /// By default, program_input_file is generated in cairo format. If you dont want, set this to false.
    #[arg(long, default_value_t = true)]
    pub cairo_format: bool,

    /// Path to save batch proof file after pre-processing.
    ///
    /// This will be used to verify the batch proof on-chain
    #[arg(short, long, requires("program_input_file"))]
    pub batch_proof_file: Option<PathBuf>,

    /// hdp cairo compiled program. main entry point
    #[arg(long)]
    pub sound_run_cairo_file: Option<PathBuf>,

    /// Path to save pie file
    ///
    /// This will trigger processing(=pie generation) step
    #[arg(short, long, requires("program_input_file"))]
    pub cairo_pie_file: Option<PathBuf>,
}

#[derive(Subcommand, Clone, Debug, PartialEq, Eq)]
pub enum DataLakeCommands {
    #[command(arg_required_else_help = true)]
    #[command(short_flag = 's')]
    BlockSampled {
        /// Chain id
        chain_id: ChainId,
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
        /// Chain id
        chain_id: ChainId,
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
