//! Module registry is a service that provides the ability to fetch modules from the StarkNet network.
//! It fetch contract class from the StarkNet network and compile it to the casm.

use anyhow::{bail, Result};
use cairo_lang_starknet_classes::{
    casm_contract_class::CasmContractClass, contract_class::ContractClass as CairoContractClass,
};
use starknet::{
    core::types::{BlockId, BlockTag, ContractClass, FlattenedSierraClass},
    providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider, Url},
};
use starknet_crypto::FieldElement;
use tracing::info;

pub struct ModuleRegistry {
    provider: JsonRpcClient<HttpTransport>,
}

impl ModuleRegistry {
    pub fn new(url: Url) -> Self {
        let provider = JsonRpcClient::new(HttpTransport::new(url));
        Self { provider }
    }

    pub async fn get_module_class(&self, class_hash: FieldElement) -> Result<CasmContractClass> {
        info!(
            "Fetching contract class from module registry... Class hash: {}",
            class_hash
        );
        let contract_class = self
            ._starknet_get_class(BlockId::Tag(BlockTag::Latest), class_hash)
            .await?;
        info!("Contract class fetched successfully");
        let sierra = match contract_class {
            ContractClass::Sierra(sierra) => sierra,
            _ => bail!("cairo1 module should have sierra as class"),
        };
        flattened_sierra_to_compiled_class(&sierra)
    }

    async fn _starknet_get_class(
        &self,
        block_id: BlockId,
        class_hash: FieldElement,
    ) -> Result<ContractClass> {
        let contract_class = self.provider.get_class(block_id, class_hash).await?;
        Ok(contract_class)
    }
}

/// Convert the given [FlattenedSierraClass] into [CasmContractClass].
/// Taken from https://github.com/dojoengine/dojo/blob/920500986855fdaf203471ac11900b15dcf6035f/crates/katana/primitives/src/conversion/rpc.rs#L140
fn flattened_sierra_to_compiled_class(sierra: &FlattenedSierraClass) -> Result<CasmContractClass> {
    let class = rpc_to_cairo_contract_class(sierra)?;
    let casm = CasmContractClass::from_contract_class(class, true, usize::MAX)?;
    Ok(casm)
}

/// Converts RPC [FlattenedSierraClass] type to Cairo's [CairoContractClass] type.
/// Taken from https://github.com/dojoengine/dojo/blob/920500986855fdaf203471ac11900b15dcf6035f/crates/katana/primitives/src/conversion/rpc.rs#L187
fn rpc_to_cairo_contract_class(sierra: &FlattenedSierraClass) -> Result<CairoContractClass> {
    let value = serde_json::to_value(sierra)?;

    Ok(CairoContractClass {
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

#[cfg(test)]
mod tests {
    use super::*;
    use hdp_primitives::constant::TEST_CONTRACT_CASM;

    fn init() -> (ModuleRegistry, FieldElement) {
        let url = Url::parse(
            "https://starknet-sepolia.g.alchemy.com/v2/lINonYKIlp4NH9ZI6wvqJ4HeZj7T4Wm6",
        )
        .unwrap();
        let module_registry = ModuleRegistry::new(url);
        // This is test contract class hash
        let class_hash = FieldElement::from_hex_be(
            "0x054af96825d987ca89cf320f7c5a8031017815d884cff1592e8ff6da309f3ca6",
        )
        .unwrap();

        (module_registry, class_hash)
    }

    #[tokio::test]
    async fn test_get_module() {
        let (module_registry, class_hash) = init();
        let casm_from_rpc = module_registry.get_module_class(class_hash).await.unwrap();

        assert_eq!(casm_from_rpc, TEST_CONTRACT_CASM.clone());
    }

    #[tokio::test]
    async fn test_flattened_sierra_to_compiled_class() {
        let (module_registry, class_hash) = init();
        let contract_class = module_registry
            ._starknet_get_class(BlockId::Tag(BlockTag::Latest), class_hash)
            .await
            .unwrap();
        let sierra = match contract_class {
            ContractClass::Sierra(sierra) => sierra,
            _ => panic!("cairo1 module should have sierra as class"),
        };
        let casm_from_rpc = flattened_sierra_to_compiled_class(&sierra).unwrap();
        assert_eq!(casm_from_rpc, TEST_CONTRACT_CASM.clone());
    }
}
