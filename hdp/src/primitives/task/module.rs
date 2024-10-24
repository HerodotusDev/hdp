//! Module is the unit of pre-processing.
//! It contains the hash and the input.
//! This is request interface for the preprocessor.

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::Felt;
use std::{path::PathBuf, str::FromStr};

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    /// Note that this program_hash is pure cairo program hash
    #[serde_as(as = "UfeHex")]
    pub program_hash: Felt,
    pub inputs: Vec<ModuleInput>,
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
            value: Felt::from_hex(value).unwrap(),
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
}
