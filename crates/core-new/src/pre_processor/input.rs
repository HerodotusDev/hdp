use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use serde::Serialize;

use crate::module::Module;

#[derive(Serialize)]
pub struct PreProcessorInput {
    module: Module,
    /// Detail casm code of the module.
    /// This will be loaded to bootloader.
    module_class: CasmContractClass,
}

impl PreProcessorInput {
    pub fn new(module: Module, module_class: CasmContractClass) -> Self {
        Self {
            module,
            module_class,
        }
    }

    pub fn get_module_class(&self) -> CasmContractClass {
        self.module_class.clone()
    }

    pub fn get_module(&self) -> Module {
        self.module.clone()
    }
}
