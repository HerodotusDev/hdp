use std::path::PathBuf;
use thiserror::Error;

pub mod dry_run;
pub mod input;
pub mod run;

#[derive(Error, Debug)]
pub enum CairoRunnerError {
    #[error("Error while running the cairo program")]
    CairoRunError,

    #[error("Error while parsing json: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Input file is empty")]
    EmptyInput,

    #[error("Error while temp file creation: {0}")]
    TempFileError(#[from] std::io::Error),

    #[error("Error while convert to alloy: {0}")]
    ConvertToAlloyError(#[from] alloy::primitives::ruint::ParseError),

    #[error("Error while parse int: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Result root not found")]
    ResultRootNotFound,

    #[error("Geneal error: {0}")]
    GeneralError(#[from] anyhow::Error),
}

/// Compatible with cairo-run command
pub fn cairo_run(
    program_path: PathBuf,
    input_string: String,
    pie_file_path: PathBuf,
) -> Result<run::RunResult, CairoRunnerError> {
    let cairo_runner = run::Runner::new(program_path);
    cairo_runner.run(input_string, pie_file_path)
}

/// Compatible with cairo-run command, performs dry run
pub fn cairo_dry_run(
    program_path: PathBuf,
    input_string: String,
) -> Result<dry_run::DryRunResult, CairoRunnerError> {
    let dry_runner = dry_run::DryRunner::new(program_path);
    dry_runner.run(input_string)
}
