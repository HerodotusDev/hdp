use alloy_primitives::FixedBytes;
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use serde::Serialize;
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

#[serde_as]
#[derive(Serialize)]
pub struct InputModule {
    #[serde_as(as = "Vec<UfeHex>")]
    pub inputs: Vec<FieldElement>,
    /// Detail class code of the module.
    /// This will be loaded to bootloader.
    pub module_class: CasmContractClass,
}

#[serde_as]
#[derive(Serialize)]
pub struct InputProcessModule {
    #[serde_as(as = "Vec<UfeHex>")]
    pub inputs: Vec<FieldElement>,
    /// Detail class code of the module.
    /// This will be loaded to bootloader.
    pub module_class: CasmContractClass,

    /// The commitment of the task.
    pub task_commitment: String,

    pub task_proof: Vec<FixedBytes<32>>,
}
