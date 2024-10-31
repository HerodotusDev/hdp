//! Module is the unit of pre-processing.
//! It contains the hash and the input.
//! This is request interface for the preprocessor.

use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Error as SerdeError;
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_types_core::felt::Felt;
use std::str::FromStr;
use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModuleError {
    #[error("Failed to convert program hash to FieldElement: {0}")]
    InvalidProgramHash(String),
    #[error("Failed to read local class file: {0}")]
    FileReadError(#[from] io::Error),
    #[error("Failed to parse CasmContractClass from local class file: {0}")]
    ParseError(#[from] SerdeError),
    #[error("Either programHash or localClassPath must be provided")]
    MissingProgramHash,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    /// Note that this program_hash is pure cairo program hash.
    /// `program_hash` is required, either provided or computed.
    #[serde_as(as = "UfeHex")]
    pub program_hash: Felt,
    pub inputs: Vec<ModuleInput>,

    /// `local_class_path` is only stored if module passed through locally.
    #[serde(skip)]
    pub local_class_path: Option<PathBuf>,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleInput {
    pub visibility: Visibility,
    #[serde_as(as = "UfeHex")]
    pub value: Felt,
}

impl ModuleInput {
    pub fn new(visibility: Visibility, value: &str) -> Self {
        Self {
            visibility,
            value: Felt::from_hex(value).expect("invalid hex string value to convert Felt"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Public,
    Private,
}

impl FromStr for ModuleInput {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 2 {
            return Err("Invalid input format");
        }

        let visibility = match parts[0] {
            "public" => Visibility::Public,
            "private" => Visibility::Private,
            _ => return Err("Unknown visibility"),
        };

        Ok(ModuleInput::new(visibility, parts[1]))
    }
}

impl From<String> for ModuleInput {
    fn from(s: String) -> Self {
        s.parse()
            .unwrap_or_else(|_| ModuleInput::new(Visibility::Private, &s))
    }
}

impl Module {
    pub fn new(
        program_hash: Felt,
        inputs: Vec<ModuleInput>,
        local_class_path: Option<PathBuf>,
    ) -> Self {
        Self {
            program_hash,
            inputs,
            local_class_path,
        }
    }

    pub fn new_from_str(
        program_hash: Option<String>,
        local_class_path: Option<PathBuf>,
        module_inputs: Vec<String>,
    ) -> Result<Self, ModuleError> {
        // Convert `program_hash` if provided, otherwise defer to `local_class_path`
        let program_hash = program_hash
            .map(|hash| {
                Felt::from_hex(&hash).map_err(|_| ModuleError::InvalidProgramHash(hash.clone()))
            })
            .transpose()?;

        // Parse module inputs, collecting any errors
        let module_inputs = module_inputs
            .into_iter()
            .map(|input| {
                ModuleInput::from_str(&input)
                    .map_err(|_| ModuleError::InvalidProgramHash(input.clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Compute `program_hash` if it was not provided, using `local_class_path`
        let program_hash = match program_hash {
            Some(hash) => hash,
            None => {
                if let Some(class_path) = &local_class_path {
                    let file_content = std::fs::read_to_string(class_path)?;
                    let casm: CasmContractClass = serde_json::from_str(&file_content)?;
                    casm.compiled_class_hash()
                } else {
                    return Err(ModuleError::MissingProgramHash);
                }
            }
        };

        Ok(Self {
            program_hash,
            inputs: module_inputs,
            local_class_path,
        })
    }

    pub fn get_program_hash(&self) -> Felt {
        self.program_hash
    }

    pub fn get_module_inputs(&self) -> Vec<ModuleInput> {
        self.inputs.clone()
    }

    /// Collect all the public inputs
    pub fn get_public_inputs(&self) -> Vec<Felt> {
        self.inputs
            .iter()
            .filter(|x| x.visibility == Visibility::Public)
            .map(|x| x.value)
            .collect()
    }
}

impl<'de> Deserialize<'de> for Module {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[serde_as]
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ModuleHelper {
            #[serde_as(as = "Option<UfeHex>")]
            program_hash: Option<Felt>,
            inputs: Vec<ModuleInput>,
            local_class_path: Option<PathBuf>,
        }

        let helper = ModuleHelper::deserialize(deserializer)?;

        let program_hash = match helper.program_hash {
            Some(hash) => hash,
            None => {
                if let Some(class_path) = &helper.local_class_path {
                    // Attempt to read and compute program_hash from local_class_path
                    let casm: CasmContractClass = serde_json::from_str(
                        &std::fs::read_to_string(class_path).map_err(|e| {
                            serde::de::Error::custom(format!(
                                "Failed to read local_class_path: {e}"
                            ))
                        })?,
                    )
                    .map_err(|e| {
                        serde::de::Error::custom(format!(
                            "Failed to parse CasmContractClass from local_class_path: {e}"
                        ))
                    })?;
                    casm.compiled_class_hash()
                } else {
                    return Err(serde::de::Error::missing_field(
                        "programHash or localClassPath",
                    ));
                }
            }
        };

        Ok(Module {
            program_hash,
            inputs: helper.inputs,
            local_class_path: helper.local_class_path,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_input() {
        let module_input_str = "public.0x123";
        let module = ModuleInput::from_str(module_input_str).unwrap();
        assert_eq!(
            module,
            ModuleInput {
                value: Felt::from_hex("0x123").unwrap(),
                visibility: Visibility::Public
            }
        );

        let module_input_str = "private.0x1";
        let module = ModuleInput::from_str(module_input_str).unwrap();
        assert_eq!(
            module,
            ModuleInput {
                value: Felt::from_hex("0x1").unwrap(),
                visibility: Visibility::Private
            }
        );
    }

    #[test]
    fn test_local_module_deserde() {
        let json_data = r#"
        {
            "type": "Module",
            "localClassPath": "../fixtures/primitives/local_class.json",
            "inputs": [
                {
                "visibility": "private",
                "value": "0x5222a4"
                },
                {
                "visibility": "public",
                "value": "0x00000000000000000000000013cb6ae34a13a0977f4d7101ebc24b87bb23f0d5"
                }
            ]
        }"#;

        let parsed: Module = serde_json::from_str(json_data).unwrap();

        let expected_program_hash =
            Felt::from_hex("0x4062c355f0b70ab4ed6ce821ad04cbb3a6fa6b3d5bcc1a89bf49cc7ed5884e1")
                .unwrap();
        let expected_inputs = vec![
            ModuleInput {
                visibility: Visibility::Private,
                value: Felt::from_hex("0x5222a4").unwrap(),
            },
            ModuleInput {
                visibility: Visibility::Public,
                value: Felt::from_hex("0x13cb6ae34a13a0977f4d7101ebc24b87bb23f0d5").unwrap(),
            },
        ];
        let expected_local_class_path =
            Some(PathBuf::from("../fixtures/primitives/local_class.json"));

        // Assertions
        assert_eq!(parsed.program_hash, expected_program_hash);
        assert_eq!(parsed.inputs, expected_inputs);
        assert_eq!(parsed.local_class_path, expected_local_class_path);
    }

    #[test]
    fn test_remote_module_deserde() {
        let json_data = r#"
        {
            "type": "Module",
            "programHash": "0x4062c355f0b70ab4ed6ce821ad04cbb3a6fa6b3d5bcc1a89bf49cc7ed5884e1",
            "inputs": [
                {
                "visibility": "private",
                "value": "0x5222a4"
                },
                {
                "visibility": "public",
                "value": "0x00000000000000000000000013cb6ae34a13a0977f4d7101ebc24b87bb23f0d5"
                }
            ]
        }"#;

        let parsed: Module = serde_json::from_str(json_data).unwrap();

        let expected_program_hash =
            Felt::from_hex("0x4062c355f0b70ab4ed6ce821ad04cbb3a6fa6b3d5bcc1a89bf49cc7ed5884e1")
                .unwrap();
        let expected_inputs = vec![
            ModuleInput {
                visibility: Visibility::Private,
                value: Felt::from_hex("0x5222a4").unwrap(),
            },
            ModuleInput {
                visibility: Visibility::Public,
                value: Felt::from_hex("0x13cb6ae34a13a0977f4d7101ebc24b87bb23f0d5").unwrap(),
            },
        ];

        // Assertions
        assert_eq!(parsed.program_hash, expected_program_hash);
        assert_eq!(parsed.inputs, expected_inputs);
        assert_eq!(parsed.local_class_path, None);
    }
}
