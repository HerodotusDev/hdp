use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use serde::Serialize;

use crate::module::Module;

#[derive(Serialize)]
pub struct PreProcessorInput {
    module: Module,
    /// Detail casm code of the module.
    /// This will be loaded to bootloader.
    module_casm: CasmContractClass,
}

impl PreProcessorInput {
    pub fn new(module: Module, module_casm: CasmContractClass) -> Self {
        Self {
            module,
            module_casm,
        }
    }

    pub fn get_module_casm(&self) -> CasmContractClass {
        self.module_casm.clone()
    }

    pub fn get_module(&self) -> Module {
        self.module.clone()
    }
}
