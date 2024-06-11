use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref TEST_CONTRACT_CASM: CasmContractClass =
        read_compiled_class_artifact(include_str!("../../contracts/compiled/test.json"));
}

pub fn read_compiled_class_artifact(artifact: &str) -> CasmContractClass {
    serde_json::from_str(artifact).unwrap()
}
