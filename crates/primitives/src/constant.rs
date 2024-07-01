use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use lazy_static::lazy_static;

pub const DRY_RUN_OUTPUT_FILE: &str = "dry_run_output.json";

lazy_static! {
    pub static ref ACCOUNT_BALANCE_EXAMPLE_CONTRACT: CasmContractClass =
        read_compiled_class_artifact(include_str!(
            "../contracts/compiled/account_balance_example.json"
        ));
}

pub fn read_compiled_class_artifact(artifact: &str) -> CasmContractClass {
    serde_json::from_str(artifact).unwrap()
}
