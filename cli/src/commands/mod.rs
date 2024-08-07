use crate::commands::run::RunArgs;
use alloy::primitives::{BlockNumber, TxIndex};
use clap::{command, Parser, Subcommand};
use hdp::primitives::task::datalake::{
    block_sampled::BlockSampledCollection,
    transactions::{IncludedTypes, TransactionsCollection},
};
use run_datalake::RunDatalakeArgs;
use run_module::RunModuleArgs;

pub mod run;
pub mod run_datalake;
pub mod run_module;

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
    RunDatalake(RunDatalakeArgs),

    /// Run single module with either class hash deployed on starknet or local class path
    #[command(arg_required_else_help = true)]
    RunModule(RunModuleArgs),
    /// Run batch of tasks base on request json file
    #[command(arg_required_else_help = true)]
    Run(RunArgs),
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
