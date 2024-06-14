//! Processor is reponsible for running the module.
//! This run is sound execution of the module.
//! This will be most abstract layer of the processor.

use std::{collections::HashSet, sync::Arc};

use anyhow::Result;

use hdp_primitives::task::ExtendedTask;
use hdp_provider::{
    evm::{AbstractProvider, AbstractProviderConfig, AbstractProviderResult},
    key::FetchKeyEnvelope,
};
use starknet::providers::Url;

use crate::{
    cairo_runner::{
        input::run::RunnerInput,
        run::{RunResult, Runner},
    },
    module_registry::ModuleRegistry,
};

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
        tasks: Vec<ExtendedTask>,
        fetch_keys: HashSet<FetchKeyEnvelope>,
    ) -> Result<RunResult> {
        // generate input file from fetch points
        // 1. fetch proofs from provider by using fetch points
        let config = AbstractProviderConfig {
            rpc_url: "http://localhost:8080",
            chain_id: 1,
            rpc_chunk_size: 1,
        };
        let provider = AbstractProvider::new(config);
        let proofs = provider.fetch_proofs_from_keys(fetch_keys).await?;
        // 2. generate input struct with proofs and module bytes
        let input = self.generate_input(proofs, tasks).await?;
        // 3. pass the input file to the runner
        let input_bytes = input.to_bytes();
        self.runner.run(input_bytes)
    }

    pub async fn generate_input(
        &self,
        proofs: AbstractProviderResult,
        tasks: Vec<ExtendedTask>,
    ) -> Result<RunnerInput> {
        // let registry: Arc<ModuleRegistry> = Arc::clone(&self.module_registry);
        // // Map each module to an asynchronous task
        // let module_futures: Vec<_> = tasks
        //     .into_iter()
        //     .map(|module_with_class| {
        //         let module_registry = Arc::clone(&registry);
        //         task::spawn(async move {
        //             // create input_module
        //             let module = module_with_class.get_module();
        //             let inputs = module.inputs;
        //             let module_class = module_registry
        //                 .get_module_class(module.class_hash)
        //                 .await
        //                 .unwrap();
        //             Ok(InputModule {
        //                 inputs,
        //                 module_class,
        //                 task_proof: vec![],
        //             })
        //         })
        //     })
        //     .collect();

        // // Join all tasks and collect their results
        // let results: Vec<_> = join_all(module_futures).await;

        // // Collect results, filter out any errors
        // let mut collected_results = Vec::new();
        // for result in results {
        //     let input_module = result??;
        //     collected_results.push(input_module);
        // }

        // Ok(RunnerInput {
        //     task_root: "".to_string(),
        //     result_root: None,
        //     modules: collected_results,
        //     proofs,
        //     datalakes: vec![],
        // });
        todo!("Implement generate_input")
    }
}
