//! Processor is reponsible for running the module.
//! This run is sound execution of the module.
//! This will be most abstract layer of the processor.

use crate::constant::DEFAULT_SOUND_CAIRO_RUN_CAIRO_FILE;
use crate::{cairo_runner::cairo_run, primitives::processed_types::cairo_format::ProcessorInput};
use anyhow::Result;
use std::env;
use std::path::PathBuf;
use tracing::{debug, info};

/// HdpProcessorConfig for the CLI
#[derive(Debug)]
pub struct HdpProcessorConfig {
    pub input_file: PathBuf,
    pub sound_run_program_path: PathBuf,
    pub cairo_pie_file: PathBuf,
}

impl HdpProcessorConfig {
    pub fn init(
        cli_sound_run_cairo_file: Option<PathBuf>,
        cli_input_file: PathBuf,
        cli_cairo_pie_file: PathBuf,
    ) -> Self {
        let sound_run_cairo_path: PathBuf = cli_sound_run_cairo_file.unwrap_or_else(|| {
            env::var("SOUND_RUN_CAIRO_PATH")
                .unwrap_or_else(|_| DEFAULT_SOUND_CAIRO_RUN_CAIRO_FILE.to_string())
                .parse()
                .expect("SOUND_RUN_CAIRO_PATH must be a path to a cairo file")
        });

        let config = HdpProcessorConfig {
            input_file: cli_input_file,
            sound_run_program_path: sound_run_cairo_path,
            cairo_pie_file: cli_cairo_pie_file,
        };

        debug!("Running with configuration: {:#?}", config);
        config
    }
}

pub struct Processor {
    program_path: PathBuf,
}

impl Processor {
    pub fn new(program_path: PathBuf) -> Self {
        Self { program_path }
    }

    /// Execute process that involves sound-cairo-run.
    pub async fn process(
        &self,
        processor_input: ProcessorInput,
        pie_file_path: &PathBuf,
    ) -> Result<()> {
        let cairo_run_input = serde_json::to_string_pretty(&processor_input)
            .expect("Failed to serialize module class");
        let _ = cairo_run(&self.program_path, cairo_run_input, pie_file_path)?;
        info!("2️⃣  Processor completed successfully");
        Ok(())
    }
}
