use crate::commands::Parser;
use starknet::providers::Url;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct RunArgs {
    /// Pass request as json file
    #[arg(short, long)]
    pub request_file: PathBuf,

    /// The RPC URL to fetch the data.
    ///
    /// Can be overwritten by `RPC_URL` environment variable.
    #[arg(long)]
    pub rpc_url: Option<Url>,

    /// dry run contract bootloader program.
    /// only used for module task
    #[arg(long)]
    pub dry_run_cairo_file: Option<PathBuf>,

    /// Path to save output file after pre-processing.
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
