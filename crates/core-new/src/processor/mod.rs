use anyhow::Result;

use crate::{
    cairo_runner::run::{RunResult, Runner},
    pre_processor::PreProcessResult,
};

pub struct Processor {
    fetch_points: Vec<String>,
    module_hash: String,
    module_bytes: Vec<u8>,
}

impl Processor {
    pub fn new(pre_processed: PreProcessResult) -> Self {
        // get module bytes from module hash
        // rearrange fetch points considering duplicated points
        let module_bytes = vec![];
        Self {
            fetch_points: pre_processed.fetch_points,
            module_hash: pre_processed.module_hash,
            module_bytes,
        }
    }

    pub fn process(&self) -> Result<RunResult> {
        // generate input file from fetch points
        // 1. fetch proofs from provider by using fetch points

        // 2. generate input struct with proofs and module bytes

        // 3. pass the input file to the runner
        let runner = Runner::new();
        let input_bytes = vec![];
        runner.run(input_bytes)
    }
}
