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
    #[arg(short, long)]
    pub cairo_pie_file: PathBuf,
}
