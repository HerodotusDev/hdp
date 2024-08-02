//! Processor is reponsible for running the module.
//! This run is sound execution of the module.
//! This will be most abstract layer of the processor.

use crate::cairo_runner::cairo_run;
use crate::primitives::processed_types::cairo_format::AsCairoFormat;
use crate::primitives::processed_types::processor_output::ProcessorOutput;
use crate::primitives::processed_types::query::ProcessorInput;
use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

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
    ) -> Result<ProcessorOutput> {
        let cairo_run_input = serde_json::to_string_pretty(&processor_input.as_cairo_format())
            .expect("Failed to serialize module class");
        let cairo_run_result = cairo_run(&self.program_path, cairo_run_input, pie_file_path)?;
        let processor_result =
            processor_input.into_processor_output(cairo_run_result.cairo_run_output.results);
        info!("2️⃣  Processor completed successfully");
        Ok(processor_result)
    }
}
