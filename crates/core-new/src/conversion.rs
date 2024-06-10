use anyhow::Result;
use cairo_lang_starknet_classes::{
    casm_contract_class::CasmContractClass, contract_class::ContractClass,
};
use starknet::core::types::FlattenedSierraClass;

/// Convert the given [FlattenedSierraClass] into [CasmContractClass].
pub fn flattened_sierra_to_compiled_class(
    sierra: &FlattenedSierraClass,
) -> Result<CasmContractClass> {
    let class = rpc_to_cairo_contract_class(sierra)?;
    let casm = CasmContractClass::from_contract_class(class, true, usize::MAX)?;
    Ok(casm)
}

/// Converts RPC [FlattenedSierraClass] type to Cairo's [ContractClass] type.
fn rpc_to_cairo_contract_class(sierra: &FlattenedSierraClass) -> Result<ContractClass> {
    let value = serde_json::to_value(sierra)?;

    Ok(ContractClass {
        abi: serde_json::from_value(value["abi"].clone()).ok(),
        sierra_program: serde_json::from_value(value["sierra_program"].clone())?,
        entry_points_by_type: serde_json::from_value(value["entry_points_by_type"].clone())?,
        contract_class_version: serde_json::from_value(value["contract_class_version"].clone())?,
        sierra_program_debug_info: serde_json::from_value(
            value["sierra_program_debug_info"].clone(),
        )
        .ok(),
    })
}
