//! Processor is reponsible for running the module.
//! This run is sound execution of the module.
//! This will be most abstract layer of the processor.

use anyhow::Result;
use hdp_provider::key::FetchKey;
use input::ProcessorInput;

use crate::{
    cairo_runner::run::{RunResult, Runner},
    module::Module,
};

pub mod input;

pub struct Processor<T> {
    _phantom: std::marker::PhantomData<T>,
    runner: Runner,
}

impl<T: FetchKey> Default for Processor<T>
where
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T: FetchKey> Processor<T>
where
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    pub fn new() -> Self {
        let runner = Runner::new();
        Self {
            _phantom: std::marker::PhantomData,
            runner,
        }
    }

    pub fn process(&self, modules: Vec<Module>, fetch_keys: Vec<T>) -> Result<RunResult> {
        // generate input file from fetch points
        // 1. fetch proofs from provider by using fetch points
        let proofs = vec![];
        // 2. generate input struct with proofs and module bytes
        let input = self.generate_input(proofs, modules);
        // 3. pass the input file to the runner
        let input_bytes = input.to_bytes();
        self.runner.run(input_bytes)
    }

    pub fn generate_input(&self, proofs: Vec<String>, modules: Vec<Module>) -> ProcessorInput {
        // TODO: get module bytes from registry by module_hash
        let modules_bytes = vec![];
        ProcessorInput::new(modules_bytes, modules, proofs)
    }
}
