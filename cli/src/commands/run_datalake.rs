use std::path::PathBuf;

use alloy::primitives::ChainId;
use clap::Parser;
use hdp::primitives::aggregate_fn::{AggregationFunction, FunctionContext};
use starknet::providers::Url;

use super::DataLakeCommands;

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

    /// The RPC URL to fetch the datalake
    pub rpc_url: Option<Url>,

    /// The chain id to fetch the datalake
    pub chain_id: Option<ChainId>,

    /// Path to save output file after pre-processing.
    ///
    /// This will trigger pre-processing step
    #[arg(short, long)]
    pub preprocessor_output_file: Option<PathBuf>,

    /// hdp cairo compiled program. main entry point
    #[arg(long)]
    pub sound_run_cairo_file: Option<PathBuf>,

    /// Path to save output file after process
    ///
    /// This will trigger processing(=pie generation) step
    #[arg(short, long, requires("preprocessor_output_file"))]
    pub output_file: Option<PathBuf>,

    /// Path to save pie file
    ///
    /// This will trigger processing(=pie generation) step
    #[arg(short, long, requires("preprocessor_output_file"))]
    pub cairo_pie_file: Option<PathBuf>,
}
