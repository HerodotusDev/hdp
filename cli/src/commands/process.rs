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

    /// Path to save output file after process
    ///
    /// This will trigger processing(=pie generation) step
    #[arg(short, long)]
    pub output_file: PathBuf,

    /// Path to save pie file
    ///
    /// This will trigger processing(=pie generation) step
    #[arg(short, long)]
    pub cairo_pie_file: PathBuf,
}
