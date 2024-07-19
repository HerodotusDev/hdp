use alloy::primitives::{Bytes, B256};
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessedModule {
    /// encoded computational task
    pub encoded_task: Bytes,
    pub task_commitment: B256,
    pub result_commitment: B256,
    pub task_proof: Vec<B256>,
    pub result_proof: Vec<B256>,

    #[serde_as(as = "Vec<UfeHex>")]
    pub inputs: Vec<FieldElement>,
    /// Detail class code of the module.
    /// This will be loaded to bootloader.
    pub module_class: CasmContractClass,
}

impl ProcessedModule {
    pub fn new(
        encoded_task: Bytes,
        task_commitment: B256,
        result_commitment: B256,
        task_proof: Vec<B256>,
        result_proof: Vec<B256>,
        inputs: Vec<FieldElement>,
        module_class: CasmContractClass,
    ) -> Self {
        ProcessedModule {
            encoded_task,
            task_commitment,
            result_commitment,
            task_proof,
            result_proof,
            inputs,
            module_class,
        }
    }
}
