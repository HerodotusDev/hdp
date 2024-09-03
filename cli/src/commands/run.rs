use crate::commands::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct RunArgs {
    /// Pass request as json file
    #[arg(short, long)]
    pub request_file: PathBuf,

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
