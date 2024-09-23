use crate::primitives::{
    processed_types::module::ProcessedModule as BaseProcessedModule, task::module::ModuleInput,
};
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::Felt;

use super::{AsCairoFormat, FieldElementVectorUnit};

impl AsCairoFormat for BaseProcessedModule {
    type Output = ProcessedModule;

    fn as_cairo_format(&self) -> Self::Output {
        let module_task_felts = FieldElementVectorUnit::from_bytes(&self.encoded_task).unwrap();
        ProcessedModule {
            module_class: self.module_class.clone(),
            encoded_task: module_task_felts.felts,
            inputs: self.inputs.clone(),
            task_bytes_len: module_task_felts.bytes_len,
        }
    }
}

impl BaseProcessedModule {
    pub fn as_dry_run_cairo_format(&self) -> DryRunProcessedModule {
        DryRunProcessedModule {
            inputs: self.inputs.clone(),
            module_class: self.module_class.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DryRunProcessedModule {
    pub inputs: Vec<ModuleInput>,
    /// Detail class code of the module.
    /// This will be loaded to bootloader.
    pub module_class: CasmContractClass,
}

impl DryRunProcessedModule {
    pub fn new(inputs: Vec<ModuleInput>, module_class: CasmContractClass) -> Self {
        Self {
            inputs,
            module_class,
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessedModule {
    #[serde_as(as = "Vec<UfeHex>")]
    pub encoded_task: Vec<Felt>,
    pub task_bytes_len: u64,
    pub inputs: Vec<ModuleInput>,
    /// Detail class code of the module.
    /// This will be loaded to bootloader.
    pub module_class: CasmContractClass,
}

impl ProcessedModule {
    pub fn new(
        encoded_task: Vec<Felt>,
        task_bytes_len: u64,
        inputs: Vec<ModuleInput>,
        module_class: CasmContractClass,
    ) -> Self {
        Self {
            encoded_task,
            task_bytes_len,
            inputs,
            module_class,
        }
    }
}
