use alloy_primitives::FixedBytes;
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use hdp_primitives::datalake::output::TaskFormatted;
use hdp_provider::evm::AbstractProviderResult;
use serde::Serialize;
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

/*
    input.json file that will be passed to the processor, is generated by this struct.
*/

#[derive(Serialize)]
pub struct RunnerInput {
    /// Batched tasks root of all tasks.
    pub task_root: String,
    /// if every tasks are pre computable, this can be Some
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_root: Option<String>,
    /// Fetched proofs per each fetch point.
    pub proofs: AbstractProviderResult,
    /// tasks compatible with v2
    pub tasks: Vec<InputTask>,
}

#[derive(Serialize)]
pub enum InputTask {
    #[serde(rename = "datalake_compute")]
    DatalakeCompute(TaskFormatted),
    #[serde(rename = "module")]
    Module(InputProcessModule),
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

impl RunnerInput {
    pub fn new(proofs: AbstractProviderResult, task_root: String, tasks: Vec<InputTask>) -> Self {
        Self {
            task_root,
            result_root: None,
            tasks,
            proofs,
        }
    }
}
