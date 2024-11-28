use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct ProcessArgs {
    /// Path to save output file after pre-processing.
    #[arg(short, long)]
    pub input_file: PathBuf,

    /// hdp cairo compiled program. main entry point
    #[arg(long)]
    pub sound_run_cairo_file: Option<PathBuf>,

    /// Path to save pie file
    ///
    /// This will trigger processing(=pie generation) step
    #[arg(short, long, requires("input_file"), conflicts_with = "proof_mode")]
    pub cairo_pie_file: Option<PathBuf>,

    /// Flag to run `cairo-run` in proof mode
    ///
    /// This will trigger processing(=pie generation) step
    /// By default, it will run in non-proof mode to generate pie
    /// Note that if this flag is set
    #[arg(long, default_value_t = false, conflicts_with = "cairo_pie_file")]
    pub proof_mode: bool,
}
