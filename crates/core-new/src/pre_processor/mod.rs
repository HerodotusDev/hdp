use config::PreProcessorConfig;

use crate::cairo_runner::pre_run::PreRunner;
use anyhow::Result;
pub mod config;

pub struct PreProcessResult {
    pub fetch_points: Vec<String>,
    pub module_hash: String,
}

/*
  Preprocessor is reponsible for identifying the required values
*/
pub struct PreProcessor {
    config: PreProcessorConfig,
}

impl PreProcessor {
    pub fn new(config: PreProcessorConfig) -> Self {
        Self { config }
    }

    pub fn process(&self) -> Result<PreProcessResult> {
        let pre_runner = PreRunner::new();
        let input_bytes = self.config.module_bytes();
        let points = pre_runner.run(input_bytes.to_vec())?;
        Ok(PreProcessResult {
            fetch_points: points,
            module_hash: self.config.module_hash().to_string(),
        })
    }
}
