//! Module registry is a service that provides the ability to fetch modules from the StarkNet network.
//! It fetch contract class from the StarkNet network and compile it to the casm.

use cairo_lang_starknet_classes::casm_contract_class::{
    CasmContractClass, StarknetSierraCompilationError,
};

use hdp_primitives::task::{module::Module, ExtendedModule};
use reqwest::Client;
use serde::Deserialize;
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
    client: Client,
}

#[derive(Deserialize)]
struct GitHubFileResponse {
    download_url: String,
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
        program_hash: Option<FieldElement>,
        local_class_path: Option<PathBuf>,
        module_inputs: Vec<FieldElement>,
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
                "One of class_hash or local_class_path must be provided".to_string(),
            ));
        };

        let program_hash = casm.compiled_class_hash();
        let converted_hash = FieldElement::from_bytes_be(&program_hash.to_be_bytes()).unwrap();
        info!("Program Hash: {:#?}", converted_hash);

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
            "Contract class fetched successfully from local path: {:?}",
            local_class_path
        );
        Ok(casm)
    }

    async fn get_module_class_from_program_hash(
        &self,
        program_hash: FieldElement,
    ) -> Result<CasmContractClass, ModuleRegistryError> {
        info!(
            "Fetching contract class from module registry... program_hash: {:#?}",
            program_hash.to_string()
        );

        let program_hash_key = program_hash.to_string();
        let branch = "v2-fix";
        let api_url = format!(
            "https://api.github.com/repos/HerodotusDev/hdp/contents/crates/contracts/{}.json?ref={}",
            program_hash_key, branch
        );

        let response_text = self
            .client
            .get(&api_url)
            .header("User-Agent", "request")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        // Print the raw response text for debugging
        println!("API Response: {}", response_text);

        // Try to deserialize the response into GitHubFileResponse
        let response: Result<GitHubFileResponse, serde_json::Error> =
            serde_json::from_str(&response_text);
        let response = match response {
            Ok(resp) => resp,
            Err(err) => {
                eprintln!("Failed to deserialize GitHubFileResponse: {}", err);
                return Err(ModuleRegistryError::ClassSourceError("fail".to_string()));
            }
        };
        let download_response = self
            .client
            .get(&response.download_url)
            .send()
            .await
            .unwrap();

        let file_content = download_response.text().await.unwrap();
        let casm: CasmContractClass = serde_json::from_str(&file_content)?;

        info!(
            "Contract class fetched successfully fromprogram_hashh: {:?}",
            program_hash
        );
        Ok(casm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hdp_primitives::constant::NEW_EXAMPLE_CONTRACT;

    fn init() -> (ModuleRegistry, FieldElement) {
        let module_registry = ModuleRegistry::new();
        // This is test contract class hash
        let class_hash = FieldElement::from_hex_be(
            "0x00ababb33ae5911fd14e6b9f2853b6271f553b9ec7835298134f4bb020100971",
        )
        .unwrap();

        (module_registry, class_hash)
    }

    #[tokio::test]
    async fn test_get_module() {
        let (module_registry, program_hash) = init();
        let casm_from_rpc = module_registry
            .get_module_class_from_program_hash(program_hash)
            .await
            .unwrap();

        assert_eq!(casm_from_rpc, NEW_EXAMPLE_CONTRACT.clone());
    }

    #[tokio::test]
    async fn test_get_multiple_module_classes() {
        let (module_registry, class_hash) = init();
        println!("{}", class_hash);

        let extended_modules = module_registry
            .get_extended_module_from_class_source(Some(class_hash), None, vec![])
            .await
            .unwrap();

        assert_eq!(
            extended_modules.task.program_hash,
            FieldElement::from_hex_be(
                "0xaf1333b8346c1ac941efe380f3122a71c1f7cbad19301543712e74f765bfca"
            )
            .unwrap()
        );
        assert_eq!(extended_modules.task.inputs, vec![]);
        assert_eq!(extended_modules.module_class, NEW_EXAMPLE_CONTRACT.clone());
    }
}
