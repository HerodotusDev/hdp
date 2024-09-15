//! Module registry is a service that provides the ability to fetch modules from the StarkNet network.
//! It fetch contract class from the StarkNet network and compile it to the casm.

use cairo_lang_starknet_classes::casm_contract_class::{
    CasmContractClass, StarknetSierraCompilationError,
};

use crate::{
    constant::HERODOTUS_PROGRAM_REGISTRY_URL,
    primitives::task::{
        module::{Module, ModuleInput},
        ExtendedModule,
    },
};
use reqwest::Client;
use starknet_crypto::FieldElement;
use std::{path::PathBuf, str::FromStr};
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

    #[error("Type conversion error: {0}")]
    TypeConversionError(String),
}

pub struct ModuleRegistry {
    client: Client,
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleRegistry {
    pub fn new() -> Self {
        let client = Client::new();
        Self { client }
    }

    pub async fn get_extended_module_from_class_source_string(
        &self,
        program_hash: Option<String>,
        local_class_path: Option<PathBuf>,
        module_inputs: Vec<String>,
    ) -> Result<ExtendedModule, ModuleRegistryError> {
        let program_hash = program_hash.map(|program_hash| {
            FieldElement::from_hex_be(&program_hash)
                .expect("program hash cannot be converted to FieldElement")
        });
        let module_inputs: Result<Vec<ModuleInput>, _> = module_inputs
            .into_iter()
            .map(|input| ModuleInput::from_str(&input))
            .collect();

        let module_inputs =
            module_inputs.map_err(|e| ModuleRegistryError::TypeConversionError(e.to_string()))?;

        self.get_extended_module_from_class_source(program_hash, local_class_path, module_inputs)
            .await
    }

    pub async fn get_extended_module_from_class_source(
        &self,
        program_hash: Option<FieldElement>,
        local_class_path: Option<PathBuf>,
        module_inputs: Vec<ModuleInput>,
    ) -> Result<ExtendedModule, ModuleRegistryError> {
        if program_hash.is_some() && local_class_path.is_some() {
            return Err(ModuleRegistryError::ClassSourceError(
                "Only one of program_hash or local_class_path must be provided".to_string(),
            ));
        }

        let casm = if let Some(ref local_class_path) = local_class_path {
            self.get_module_class_from_local_path(local_class_path)
                .await?
        } else if let Some(program_hash) = program_hash {
            self.get_module_class_from_program_hash(program_hash)
                .await?
        } else {
            return Err(ModuleRegistryError::ClassSourceError(
                "One of program_hash or local_class_path must be provided".to_string(),
            ));
        };

        let program_hash = casm.compiled_class_hash();
        let converted_hash = FieldElement::from_bytes_be(&program_hash.to_bytes_be())
            .expect("program hash cannot be converted to FieldElement");
        info!("program Hash: {:#?}", converted_hash);

        let module = Module {
            program_hash: converted_hash,
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
            "contract class fetched successfully from local path: {:?}",
            local_class_path
        );
        Ok(casm)
    }

    async fn get_module_class_from_program_hash(
        &self,
        program_hash: FieldElement,
    ) -> Result<CasmContractClass, ModuleRegistryError> {
        let program_hash_hex = format!("{:#x}", program_hash);

        info!(
            "fetching contract class from module registry... program_hash: {}",
            program_hash_hex
        );

        let api_url = format!("{}={}", HERODOTUS_PROGRAM_REGISTRY_URL, program_hash_hex);

        let response = self
            .client
            .get(&api_url)
            .header("User-Agent", "request")
            .send()
            .await
            .expect("response is failed");

        // Check if the response status is successful
        if response.status().is_success() {
            let response_text = response.text().await.expect("cannot get response");
            let casm: CasmContractClass = serde_json::from_str(&response_text)?;
            info!(
                "contract class fetched successfully from program_hash: {:?}",
                program_hash
            );
            Ok(casm)
        } else {
            Err(ModuleRegistryError::ClassSourceError(
                "failed to fetch contract class".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn init() -> (ModuleRegistry, FieldElement) {
        let module_registry = ModuleRegistry::new();
        // This is test contract class hash
        let program_hash = FieldElement::from_hex_be(
            "0x64041a339b1edd10de83cf031cfa938645450f971d2527c90d4c2ce68d7d412",
        )
        .unwrap();

        (module_registry, program_hash)
    }

    #[tokio::test]
    async fn test_get_module() {
        let (module_registry, program_hash) = init();
        let _casm_from_rpc = module_registry
            .get_module_class_from_program_hash(program_hash)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_multiple_module_classes() {
        let (module_registry, program_hash) = init();
        println!("{}", program_hash);

        let extended_modules = module_registry
            .get_extended_module_from_class_source(Some(program_hash), None, vec![])
            .await
            .unwrap();

        assert_eq!(
            extended_modules.task.program_hash,
            FieldElement::from_hex_be(
                "0x64041a339b1edd10de83cf031cfa938645450f971d2527c90d4c2ce68d7d412"
            )
            .unwrap()
        );
        assert_eq!(extended_modules.task.inputs, vec![]);
    }
}
