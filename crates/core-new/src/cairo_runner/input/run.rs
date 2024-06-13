use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use hdp_primitives::datalake::output::Task;
use hdp_provider::evm::AbstractProviderResult;
use serde::Serialize;
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

/*
    input.json file that will be passed to the processor, is generated by this struct.
*/

pub struct RunnerInput {
    /// Batched tasks root of all tasks.
    pub task_root: String,
    /// if every tasks are pre computable, this can be Some
    pub result_root: Option<String>,
    /// Detail sierra code of the module.
    /// This will be loaded to bootloader.
    /// tasks compatible with v2
    pub modules: Vec<InputModule>,
    /// Fetched proofs per each fetch point.
    pub proofs: AbstractProviderResult,
    /// tasks compatible with v1
    pub datalakes: Vec<Task>,
}

#[serde_as]
#[derive(Serialize)]
pub struct InputModule {
    #[serde_as(as = "Vec<UfeHex>")]
    pub inputs: Vec<FieldElement>,
    /// Detail class code of the module.
    /// This will be loaded to bootloader.
    pub module_class: CasmContractClass,
    /// inclusion proof of the batch
    pub task_proof: Vec<String>,
}

impl RunnerInput {
    pub fn new(proofs: AbstractProviderResult, datalakes: Vec<Task>) -> Self {
        Self {
            task_root: "".to_string(),
            result_root: None,
            modules: vec![],
            proofs,
            datalakes,
        }
    }

    pub fn add_module(&mut self, inputs: Vec<FieldElement>, module_class: CasmContractClass) {
        todo!("Add module to RunnerInput")
    }

    // TODO: Somehow need to make `Vec<CairoProgram>`, `Vec<Module>`, Vec<Proof> to input data format
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!("Convert ProcessorInput to json")
    }
}