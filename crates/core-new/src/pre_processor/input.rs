use crate::module::Module;
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use serde::Serialize;
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

#[serde_as]
#[derive(Serialize)]
pub struct PreProcessorInput {
    #[serde_as(as = "Vec<UfeHex>")]
    pub inputs: Vec<FieldElement>,
    /// Detail casm code of the module.
    /// This will be loaded to bootloader.
    module_class: CasmContractClass,
}

impl PreProcessorInput {
    pub fn new(module: Module, module_class: CasmContractClass) -> Self {
        Self {
            inputs: module.inputs,
            module_class,
        }
    }

    pub fn get_module_class(&self) -> CasmContractClass {
        self.module_class.clone()
    }

    pub fn get_inputs(&self) -> Vec<FieldElement> {
        self.inputs.clone()
    }
}
