use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use serde::Serialize;
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

#[serde_as]
#[derive(Serialize, Clone)]
pub struct ProcessedModule {
    #[serde_as(as = "Vec<UfeHex>")]
    pub inputs: Vec<FieldElement>,
    /// Detail class code of the module.
    /// This will be loaded to bootloader.
    pub module_class: CasmContractClass,
}

impl ProcessedModule {
    pub fn new(inputs: Vec<FieldElement>, module_class: CasmContractClass) -> Self {
        ProcessedModule {
            inputs,
            module_class,
        }
    }
}
