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

    pub async fn get_module(&self, contract_address: FieldElement) -> Result<CasmContractClass> {
        let contract_class = self
            ._starknet_get_class(BlockId::Tag(BlockTag::Latest), contract_address)
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
        contract_address: FieldElement,
    ) -> Result<ContractClass> {
        let contract_class = self
            .provider
            .get_class_at(block_id, contract_address)
            .await?;
        Ok(contract_class)
    }
}
