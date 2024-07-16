use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use lazy_static::lazy_static;

pub const DRY_CAIRO_RUN_OUTPUT_FILE: &str = "dry_run_output.json";
pub const SOUND_CAIRO_RUN_OUTPUT_FILE: &str = "cairo_run_output.json";
pub const DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE: &str = "build/compiled_cairo/contract_dry_run.json";
pub const DEFAULT_SOUND_CAIRO_RUN_CAIRO_FILE: &str = "build/compiled_cairo/hdp.json";

lazy_static! {
    pub static ref ACCOUNT_BALANCE_EXAMPLE_CONTRACT: CasmContractClass =
        read_compiled_class_artifact(include_str!(
            "../../contracts/account_balance_example.compiled_contract_class.json"
        ));
}

pub fn read_compiled_class_artifact(artifact: &str) -> CasmContractClass {
    serde_json::from_str(artifact).unwrap()
}
