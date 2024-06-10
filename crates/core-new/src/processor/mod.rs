//! Processor is reponsible for running the module.
//! This run is sound execution of the module.
//! This will be most abstract layer of the processor.

use std::sync::Arc;

use anyhow::Result;
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use futures::future::join_all;
use hdp_provider::key::FetchKeyEnvelope;
use input::ProcessorInput;
use starknet::{core::types::FieldElement, providers::Url};
use tokio::task;

use crate::{
    cairo_runner::run::{RunResult, Runner},
    module::Module,
    module_registry::ModuleRegistry,
};

pub mod input;

pub struct Processor {
    runner: Runner,
    /// Registery provider
    module_registry: Arc<ModuleRegistry>,
}

impl Processor {
    pub fn new(url: &str) -> Self {
        let url = Url::parse(url).expect("Invalid url");
        let module_registry = ModuleRegistry::new(url);
        let runner = Runner::new();
        Self {
            runner,
            module_registry: Arc::new(module_registry),
        }
    }

    pub async fn process(
        &self,
        modules: Vec<Module>,
        fetch_keys: Vec<FetchKeyEnvelope>,
    ) -> Result<RunResult> {
        // generate input file from fetch points
        // 1. fetch proofs from provider by using fetch points
        let proofs = vec![];
        // 2. generate input struct with proofs and module bytes
        let input = self.generate_input(proofs, modules).await?;
        // 3. pass the input file to the runner
        let input_bytes = input.to_bytes();
        self.runner.run(input_bytes)
    }

    pub async fn generate_input(
        &self,
        proofs: Vec<String>,
        modules: Vec<Module>,
    ) -> Result<ProcessorInput> {
        let class_hashes: Vec<FieldElement> = modules
            .iter()
            .map(|module| module.get_class_hash())
            .collect();
        let modules_casm = self._process_modules_in_parallel(class_hashes).await?;

        Ok(ProcessorInput::new(modules_casm, modules, proofs))
    }

    async fn _process_modules_in_parallel(
        &self,
        class_hashes: Vec<FieldElement>,
    ) -> Result<Vec<CasmContractClass>> {
        let registry = Arc::clone(&self.module_registry);
        // Map each module to an asynchronous task
        let module_futures: Vec<_> = class_hashes
            .into_iter()
            .map(|hash| {
                let module_registry = Arc::clone(&registry);
                task::spawn(async move { module_registry.get_module(hash).await })
            })
            .collect();

        // Join all tasks and collect their results
        let results: Vec<_> = join_all(module_futures).await;

        // Collect results, filter out any errors
        let mut collected_results = Vec::new();
        for result in results {
            match result {
                Ok(Ok(data)) => collected_results.push(data),
                Ok(Err(e)) => eprintln!("Error processing module: {}", e), // Handle each module's error
                Err(e) => eprintln!("Task failed: {:?}", e), // Handle the task failure
            }
        }

        Ok(collected_results)
    }
}
