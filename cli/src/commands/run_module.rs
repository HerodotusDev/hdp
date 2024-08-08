use std::path::PathBuf;

use alloy::primitives::ChainId;
use clap::{arg, Parser};
use starknet::providers::Url;

#[derive(Parser, Debug)]
pub struct RunModuleArgs {
    /// Input field elements for the module contract.
    /// The input field elements should be separated by comma.
    ///
    /// e.g. "0x1234,0xabcd"
    #[arg(long, required = true, use_value_delimiter = true)]
    pub module_inputs: Vec<String>,

    #[arg(long, group = "class_source")]
    pub program_hash: Option<String>,

    /// Local path of the contract class file.
    /// Make sure to have structure match with [CasmContractClass](https://github.com/starkware-libs/cairo/blob/53f7a0d26d5c8a99a8ad6ba07207a762678f2931/crates/cairo-lang-starknet-classes/src/casm_contract_class.rs)
    ///
    /// (Note: either class_hash or local_class_path should be provided)
    #[arg(long, group = "class_source")]
    pub local_class_path: Option<PathBuf>,

    /// optionally can save keys for module task to file
    #[arg(long)]
    pub save_fetch_keys_file: Option<PathBuf>,

    /// The RPC URL to fetch the data.
    ///
    /// Can be overwritten by `RPC_URL` environment variable.
    #[arg(long)]
    pub rpc_url: Option<Url>,

    /// The chain id to fetch the data.
    ///
    /// Can be overwritten by `CHAIN_ID` environment variable
    #[arg(long)]
    pub chain_id: Option<ChainId>,

    /// dry run contract bootloader program.
    /// only used for module task
    #[arg(long)]
    pub dry_run_cairo_file: PathBuf,

    /// Path to save output file after pre-processing.
    ///
    /// This will trigger pre-processing step
    #[arg(short, long)]
    pub preprocessor_output_file: PathBuf,

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
