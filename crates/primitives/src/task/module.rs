//! Module is the unit of pre-processing.
//! It contains the hash and the input.
//! This is request interface for the preprocessor.

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::{serde::unsigned_field_element::UfeHex, types::FromStrError};
use starknet_crypto::FieldElement;
use std::path::PathBuf;

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    /// Note that this program_hash is pure cairo contract's program hash
    #[serde_as(as = "UfeHex")]
    pub program_hash: FieldElement,
    #[serde_as(as = "Vec<UfeHex>")]
    pub inputs: Vec<FieldElement>,
    pub local_class_path: Option<PathBuf>,
}

pub enum ModuleTag {
    AccountBalanceExample,
}

impl Module {
    pub fn from_tag(tag: ModuleTag, inputs: Vec<FieldElement>) -> Self {
        let program_hash = match tag {
            ModuleTag::AccountBalanceExample => FieldElement::from_hex_be(
                "0x034d4ff54bc5c6cfee6719bfaa94ffa374071e8d656b74823681a955e9033dd9",
            ),
        }
        .expect("Invalid module tag");
        Self {
            program_hash,
            inputs,
            local_class_path: None,
        }
    }

    pub fn new_from_string(
        class_hash: String,
        inputs: Vec<String>,
        local_class_path: Option<PathBuf>,
    ) -> Result<Self, FromStrError> {
        let program_hash = FieldElement::from_hex_be(&class_hash)?;
        let inputs = inputs
            .iter()
            .map(|x| FieldElement::from_hex_be(x))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            program_hash,
            inputs,
            local_class_path,
        })
    }

    pub fn new(
        program_hash: FieldElement,
        inputs: Vec<FieldElement>,
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

    pub fn get_module_inputs(&self) -> Vec<FieldElement> {
        self.inputs.clone()
    }
}
