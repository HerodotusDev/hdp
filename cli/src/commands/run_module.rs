use clap::{arg, Parser};
use hdp::primitives::chain_id::ChainId;
use starknet::providers::Url;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct RunModuleArgs {
    /// Input field elements for the module contract.
    /// The input field elements should be separated by comma.
    /// The first element is visibility, and the second element is the value.
    ///
    /// e.g. "private.0x1234,public.0xabcd"
    #[arg(long, required = true, use_value_delimiter = true)]
    pub module_inputs: Vec<String>,

    /// Program hash of the contract class.
    /// (Note: either class_hash or local_class_path should be provided)
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
    pub dry_run_cairo_file: Option<PathBuf>,

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
