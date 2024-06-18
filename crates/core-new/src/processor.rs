//! Processor is reponsible for running the module.
//! This run is sound execution of the module.
//! This will be most abstract layer of the processor.

use anyhow::Result;

use hdp_provider::evm::{AbstractProvider, AbstractProviderConfig, AbstractProviderResult};

use crate::{
    cairo_runner::{
        input::run::RunnerInput,
        run::{RunResult, Runner},
    },
    pre_processor::{ExtendedTask, PreProcessResult},
};

pub struct Processor {
    runner: Runner,
    provider: AbstractProvider,
}

impl Processor {
    pub fn new(provider_config: AbstractProviderConfig) -> Self {
        let runner = Runner::new();
        let provider = AbstractProvider::new(provider_config);
        Self { runner, provider }
    }

    pub async fn process(&self, requset: PreProcessResult) -> Result<RunResult> {
        // generate input file from fetch points
        // 1. fetch proofs from provider by using fetch points
        let proofs = self
            .provider
            .fetch_proofs_from_keys(requset.fetch_keys)
            .await?;

        println!("Proofs: {:?}", proofs);

        // 2. pre-compute tasks

        // 2. generate input struct with proofs and module bytes
        let input = self.generate_input(proofs, requset.tasks).await?;
        // 3. pass the input file to the runner
        let input_bytes = input.to_bytes();
        self.runner.run(input_bytes)
    }

    async fn generate_input(
        &self,
        proofs: AbstractProviderResult,
        tasks: Vec<ExtendedTask>,
    ) -> Result<RunnerInput> {
        todo!("Generate input file for runner")
    }
}
