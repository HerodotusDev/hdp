use alloy::primitives::{Bytes, B256, U256};
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use serde::{Deserialize, Serialize};

use crate::primitives::task::module::ModuleInput;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessedModule {
    /// encoded computational task
    pub encoded_task: Bytes,
    pub task_commitment: B256,
    pub result_commitment: B256,
    /// raw evaluation result of target compiled task
    pub compiled_result: U256,
    pub task_proof: Vec<B256>,
    pub result_proof: Vec<B256>,

    pub inputs: Vec<ModuleInput>,
    /// Detail class code of the module.
    /// This will be loaded to bootloader.
    pub module_class: CasmContractClass,
}

impl ProcessedModule {
    pub fn new(
        encoded_task: Bytes,
        task_commitment: B256,
        result_commitment: B256,
        compiled_result: U256,
        task_proof: Vec<B256>,
        result_proof: Vec<B256>,
        inputs: Vec<ModuleInput>,
        module_class: CasmContractClass,
    ) -> Self {
        ProcessedModule {
            encoded_task,
            task_commitment,
            result_commitment,
            compiled_result,
            task_proof,
            result_proof,
            inputs,
            module_class,
        }
    }
}
