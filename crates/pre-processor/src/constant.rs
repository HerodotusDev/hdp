use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref NEW_EXAMPLE_CONTRACT: CasmContractClass = read_compiled_class_artifact(
        include_str!("../module-registery/2827408317705259417874723934254019043433650844527016195134625210424507225106.json")
    );
}

pub fn read_compiled_class_artifact(artifact: &str) -> CasmContractClass {
    serde_json::from_str(artifact).unwrap()
}
