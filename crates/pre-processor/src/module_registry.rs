//! Module registry is a service that provides the ability to fetch modules from the StarkNet network.
//! It fetch contract class from the StarkNet network and compile it to the casm.

use cairo_lang_starknet_classes::{
    casm_contract_class::{CasmContractClass, StarknetSierraCompilationError},
    contract_class::ContractClass as CairoContractClass,
};
use futures::future::join_all;
use hdp_primitives::task::module::Module;
use starknet::{
    core::types::{BlockId, BlockTag, ContractClass, FlattenedSierraClass},
    providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider, Url},
};
use starknet_crypto::FieldElement;
use std::sync::Arc;
use thiserror::Error;
use tokio::task;
use tracing::info;

use crate::ExtendedModule;

#[derive(Error, Debug)]
pub enum ModuleRegistryError {
    #[error("Serialize error: {0}")]
    SerializeError(#[from] serde_json::Error),

    #[error("StarkNet error: {0}")]
    StarkNetSierraCompileError(#[from] StarknetSierraCompilationError),

    #[error("StarkNet Provider error: {0}")]
    StarkNetProviderError(#[from] starknet::providers::ProviderError),

    #[error("Cairo1 module should have sierra as class")]
    SierraNotFound,

    #[error("Tokio join error: {0}")]
    TokioJoinError(#[from] tokio::task::JoinError),
}

pub struct ModuleRegistry {
    provider: JsonRpcClient<HttpTransport>,
}

impl ModuleRegistry {
    pub fn new(url: Url) -> Self {
        let provider = JsonRpcClient::new(HttpTransport::new(url));
        Self { provider }
    }

    pub async fn get_multiple_module_classes(
        self: Arc<Self>,
        modules: Vec<Module>,
    ) -> Result<Vec<ExtendedModule>, ModuleRegistryError> {
        // Create an Arc to the ModuleRegistry to be used inside the async blocks
        let module_registry: Arc<ModuleRegistry> = self;

        // Map each module to an asynchronous task
        let module_futures: Vec<_> = modules
            .into_iter()
            .map(|module| {
                let module_registry = Arc::clone(&module_registry);
                task::spawn(async move {
                    let module_hash = module.class_hash;
                    let module_class = module_registry.get_module_class(module_hash).await?;
                    Ok(ExtendedModule {
                        task: module,
                        module_class,
                    }) as Result<ExtendedModule, ModuleRegistryError>
                })
            })
            .collect();

        // Join all tasks and collect their results
        let results = join_all(module_futures).await;

        // Collect results, filter out any errors
        let mut collected_results = Vec::new();
        for result in results {
            let module_with_class = result??;
            collected_results.push(module_with_class);
        }

        Ok(collected_results)
    }

    pub async fn get_module_class(
        &self,
        class_hash: FieldElement,
    ) -> Result<CasmContractClass, ModuleRegistryError> {
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
            _ => return Err(ModuleRegistryError::SierraNotFound),
        };
        flattened_sierra_to_compiled_class(&sierra)
    }

    async fn _starknet_get_class(
        &self,
        block_id: BlockId,
        class_hash: FieldElement,
    ) -> Result<ContractClass, ModuleRegistryError> {
        let contract_class = self.provider.get_class(block_id, class_hash).await?;
        Ok(contract_class)
    }
}

/// Convert the given [FlattenedSierraClass] into [CasmContractClass].
/// Taken from https://github.com/dojoengine/dojo/blob/920500986855fdaf203471ac11900b15dcf6035f/crates/katana/primitives/src/conversion/rpc.rs#L140
fn flattened_sierra_to_compiled_class(
    sierra: &FlattenedSierraClass,
) -> Result<CasmContractClass, ModuleRegistryError> {
    let class = rpc_to_cairo_contract_class(sierra)?;
    let casm = CasmContractClass::from_contract_class(class, true, usize::MAX)?;
    Ok(casm)
}

/// Converts RPC [FlattenedSierraClass] type to Cairo's [CairoContractClass] type.
/// Taken from https://github.com/dojoengine/dojo/blob/920500986855fdaf203471ac11900b15dcf6035f/crates/katana/primitives/src/conversion/rpc.rs#L187
fn rpc_to_cairo_contract_class(
    sierra: &FlattenedSierraClass,
) -> Result<CairoContractClass, ModuleRegistryError> {
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
    use hdp_primitives::constant::ACCOUNT_BALANCE_EXAMPLE_CONTRACT;

    fn init() -> (ModuleRegistry, FieldElement) {
        let url = Url::parse(
            "https://starknet-sepolia.g.alchemy.com/v2/lINonYKIlp4NH9ZI6wvqJ4HeZj7T4Wm6",
        )
        .unwrap();
        let module_registry = ModuleRegistry::new(url);
        // This is test contract class hash
        let class_hash = FieldElement::from_hex_be(
            "0x034d4ff54bc5c6cfee6719bfaa94ffa374071e8d656b74823681a955e9033dd9",
        )
        .unwrap();

        (module_registry, class_hash)
    }

    #[tokio::test]
    async fn test_get_module() {
        let (module_registry, class_hash) = init();
        let casm_from_rpc = module_registry.get_module_class(class_hash).await.unwrap();
        std::fs::write(
            "contract.casm",
            serde_json::to_string_pretty(&casm_from_rpc).unwrap(),
        )
        .unwrap();

        assert_eq!(casm_from_rpc, ACCOUNT_BALANCE_EXAMPLE_CONTRACT.clone());
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
        assert_eq!(casm_from_rpc, ACCOUNT_BALANCE_EXAMPLE_CONTRACT.clone());
    }

    #[tokio::test]
    async fn test_get_multiple_module_classes() {
        let (module_registry, class_hash) = init();
        let module = Module {
            class_hash,
            inputs: vec![],
        };
        let arc_registry = Arc::new(module_registry);
        let extended_modules = arc_registry
            .get_multiple_module_classes(vec![module.clone(), module.clone()])
            .await
            .unwrap();
        assert_eq!(extended_modules.len(), 2);
        assert_eq!(extended_modules[0].task, module);
        assert_eq!(
            extended_modules[0].module_class,
            ACCOUNT_BALANCE_EXAMPLE_CONTRACT.clone()
        );
        assert_eq!(extended_modules[1].task, module);
        assert_eq!(
            extended_modules[1].module_class,
            ACCOUNT_BALANCE_EXAMPLE_CONTRACT.clone()
        );
    }
}
