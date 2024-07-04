//! Module registry is a service that provides the ability to fetch modules from the StarkNet network.
//! It fetch contract class from the StarkNet network and compile it to the casm.

use cairo_lang_starknet_classes::{
    casm_contract_class::{CasmContractClass, StarknetSierraCompilationError},
    contract_class::ContractClass as CairoContractClass,
};

use hdp_primitives::task::{module::Module, ExtendedModule};
use starknet::{
    core::types::{BlockId, BlockTag, ContractClass, FlattenedSierraClass},
    providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider, Url},
};
use starknet_crypto::FieldElement;
use std::path::PathBuf;
use thiserror::Error;
use tracing::info;

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

    #[error("Module class source error: {0}")]
    ClassSourceError(String),
}

pub struct ModuleRegistry {
    provider: JsonRpcClient<HttpTransport>,
}

impl ModuleRegistry {
    pub fn new(url: Url) -> Self {
        let provider = JsonRpcClient::new(HttpTransport::new(url));
        Self { provider }
    }

    pub async fn get_extended_module_from_class_source_string(
        &self,
        class_hash: Option<String>,
        local_class_path: Option<PathBuf>,
        module_inputs: Vec<String>,
    ) -> Result<ExtendedModule, ModuleRegistryError> {
        let class_hash =
            class_hash.map(|class_hash| FieldElement::from_hex_be(&class_hash).unwrap());
        let module_inputs = module_inputs
            .into_iter()
            .map(|input| FieldElement::from_hex_be(&input).unwrap())
            .collect();
        self.get_extended_module_from_class_source(class_hash, local_class_path, module_inputs)
            .await
    }

    pub async fn get_extended_module_from_class_source(
        &self,
        class_hash: Option<FieldElement>,
        local_class_path: Option<PathBuf>,
        module_inputs: Vec<FieldElement>,
    ) -> Result<ExtendedModule, ModuleRegistryError> {
        if class_hash.is_some() && local_class_path.is_some() {
            return Err(ModuleRegistryError::ClassSourceError(
                "Only one of class_hash or local_class_path must be provided".to_string(),
            ));
        }

        let casm = if let Some(ref local_class_path) = local_class_path {
            self.get_module_class_from_local_path(local_class_path)
                .await?
        } else if let Some(class_hash) = class_hash {
            self.get_module_class(class_hash).await?
        } else {
            return Err(ModuleRegistryError::ClassSourceError(
                "One of class_hash or local_class_path must be provided".to_string(),
            ));
        };

        let class_hash = casm.compiled_class_hash();
        let converted_hash = FieldElement::from_bytes_be(&class_hash.to_be_bytes()).unwrap();
        info!("Program Hash: {:?}", converted_hash);

        let module = Module {
            class_hash: converted_hash,
            inputs: module_inputs,
            local_class_path,
        };

        Ok(ExtendedModule {
            task: module,
            module_class: casm,
        })
    }

    async fn get_module_class_from_local_path(
        &self,
        local_class_path: &PathBuf,
    ) -> Result<CasmContractClass, ModuleRegistryError> {
        let casm: CasmContractClass =
            serde_json::from_str(&std::fs::read_to_string(local_class_path).map_err(|_| {
                ModuleRegistryError::ClassSourceError(
                    "Local class path is not a valid JSON file".to_string(),
                )
            })?)?;

        info!(
            "Contract class fetched successfully from local path: {:?}",
            local_class_path
        );
        Ok(casm)
    }

    async fn get_module_class(
        &self,
        class_hash: FieldElement,
    ) -> Result<CasmContractClass, ModuleRegistryError> {
        info!(
            "Fetching contract class from module registry... Contract Class Hash: {}",
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

        assert_eq!(casm_from_rpc, ACCOUNT_BALANCE_EXAMPLE_CONTRACT.clone());
    }

    #[tokio::test]
    async fn test_get_module_class_from_local_path() {
        let (module_registry, _) = init();
        let _ = module_registry
            .get_module_class_from_local_path(&PathBuf::from(
                "../contracts/account_balance_example.compiled_contract_class.json",
            ))
            .await
            .unwrap();
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

        let extended_modules = module_registry
            .get_extended_module_from_class_source(Some(class_hash), None, vec![])
            .await
            .unwrap();

        assert_eq!(extended_modules.task.class_hash, class_hash);
        assert_eq!(extended_modules.task.inputs, vec![]);
        assert_eq!(
            extended_modules.module_class,
            ACCOUNT_BALANCE_EXAMPLE_CONTRACT.clone()
        );
    }
}
