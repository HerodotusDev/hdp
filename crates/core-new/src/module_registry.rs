//! Module registry is a service that provides the ability to fetch modules from the StarkNet network.
//! It fetch contract class from the StarkNet network and compile it to the casm.

use anyhow::{bail, Result};
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use starknet::{
    core::types::{BlockId, BlockTag, ContractClass, FieldElement},
    providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider, Url},
};

use crate::conversion::flattened_sierra_to_compiled_class;

pub struct ModuleRegistry {
    provider: JsonRpcClient<HttpTransport>,
}

impl ModuleRegistry {
    pub fn new(url: Url) -> Self {
        let provider = JsonRpcClient::new(HttpTransport::new(url));
        Self { provider }
    }

    pub async fn get_module_class(&self, class_hash: FieldElement) -> Result<CasmContractClass> {
        let contract_class = self
            ._starknet_get_class(BlockId::Tag(BlockTag::Latest), class_hash)
            .await?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constant::TEST_CONTRACT_CASM;

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
