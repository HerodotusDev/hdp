//! The input for the pre-runner.
//! This serialized struct will be passed to the pre-runner(cairo-run) as input.json file.

use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use serde::Serialize;
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;
use std::path::PathBuf;

#[serde_as]
#[derive(Serialize)]
pub struct PreRunnerInput {
    pub identified_keys_file: PathBuf,
    pub modules: Vec<InputModule>,
}

#[serde_as]
#[derive(Serialize)]
pub struct InputModule {
    #[serde_as(as = "Vec<UfeHex>")]
    pub inputs: Vec<FieldElement>,
    /// Detail class code of the module.
    /// This will be loaded to bootloader.
    pub module_class: CasmContractClass,
}

impl PreRunnerInput {
    pub fn new(identified_keys_file: PathBuf) -> Self {
        Self {
            identified_keys_file,
            modules: vec![],
        }
    }

    pub fn add_module(&mut self, inputs: Vec<FieldElement>, module_class: CasmContractClass) {
        self.modules.push(InputModule {
            inputs,
            module_class,
        });
    }
}
