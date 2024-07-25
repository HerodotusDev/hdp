use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use lazy_static::lazy_static;

pub const DRY_CAIRO_RUN_OUTPUT_FILE: &str = "dry_run_output.json";
pub const SOUND_CAIRO_RUN_OUTPUT_FILE: &str = "cairo_run_output.json";
pub const DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE: &str = "build/contract_dry_run.json";
pub const DEFAULT_SOUND_CAIRO_RUN_CAIRO_FILE: &str = "build/hdp.json";

lazy_static! {
    pub static ref NEW_EXAMPLE_CONTRACT: CasmContractClass = read_compiled_class_artifact(
        include_str!("../../../module-registery/309330763873113246779612148550241378819218676920940711586132083200424787914.json")
    );
}

pub fn read_compiled_class_artifact(artifact: &str) -> CasmContractClass {
    serde_json::from_str(artifact).unwrap()
}
