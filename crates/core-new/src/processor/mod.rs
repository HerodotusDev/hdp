use anyhow::Result;
use input::ProcessorInput;

use crate::{
    cairo_runner::run::{RunResult, Runner},
    module::Module,
};

pub mod input;

/*
    Processor is reponsible for running the module.
    This run is sound execution of the module.
    This will be most abstract layer of the processor.
*/
pub struct Processor {
    runner: Runner,
}

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor {
    pub fn new() -> Self {
        let runner = Runner::new();
        Self { runner }
    }

    pub fn process(&self, module: Module, fetch_points: Vec<String>) -> Result<RunResult> {
        // generate input file from fetch points
        // 1. fetch proofs from provider by using fetch points
        let proofs = vec![];
        // 2. generate input struct with proofs and module bytes
        let input = self.generate_input(proofs, module);
        // 3. pass the input file to the runner
        let input_bytes = input.to_bytes();
        self.runner.run(input_bytes)
    }

    pub fn generate_input(&self, proofs: Vec<String>, module: Module) -> ProcessorInput {
        // TODO: get module bytes from registry by module_hash
        let module_bytes = vec![];
        ProcessorInput::new(module_bytes, module, proofs)
    }
}
