//! The input for the pre-runner.
//! This serialized struct will be passed to the pre-runner(cairo-run) as input.json file.

use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use hdp_primitives::processed_types::module::ProcessedModule;
use serde::Serialize;
use serde_with::serde_as;
use starknet_crypto::FieldElement;
use std::path::PathBuf;

#[serde_as]
#[derive(Serialize)]
pub struct PreRunnerInput {
    pub identified_keys_file: PathBuf,
    pub modules: Vec<ProcessedModule>,
}

impl PreRunnerInput {
    pub fn new(identified_keys_file: PathBuf) -> Self {
        Self {
            identified_keys_file,
            modules: vec![],
        }
    }

    pub fn add_module(&mut self, inputs: Vec<FieldElement>, module_class: CasmContractClass) {
        self.modules
            .push(ProcessedModule::new(inputs, module_class));
    }
}
