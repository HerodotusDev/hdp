//! Module is the unit of pre-processing.
//! It contains the hash and the input.
//! This is request interface for the preprocessor.

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;
use std::{path::PathBuf, str::FromStr};

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    /// Note that this program_hash is pure cairo program hash
    #[serde_as(as = "UfeHex")]
    pub program_hash: FieldElement,
    pub inputs: Vec<ModuleInput>,
    pub local_class_path: Option<PathBuf>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Public,
    Private,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleInput {
    pub visibility: Visibility,
    #[serde_as(as = "UfeHex")]
    pub value: FieldElement,
}

impl ModuleInput {
    pub fn new(visibility: Visibility, value: &str) -> Self {
        Self {
            visibility,
            value: FieldElement::from_hex_be(value).unwrap(),
        }
    }
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
        program_hash: FieldElement,
        inputs: Vec<ModuleInput>,
        local_class_path: Option<PathBuf>,
    ) -> Self {
        Self {
            program_hash,
            inputs,
            local_class_path,
        }
    }

    pub fn get_program_hash(&self) -> FieldElement {
        self.program_hash
    }

    pub fn get_module_inputs(&self) -> Vec<ModuleInput> {
        self.inputs.clone()
    }

    /// Collect all the public inputs
    pub fn get_public_inputs(&self) -> Vec<FieldElement> {
        self.inputs
            .iter()
            .filter(|x| x.visibility == Visibility::Public)
            .map(|x| x.value)
            .collect()
    }
}
