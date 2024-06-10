//! Module is the unit of pre-processing.
//! It contains the hash and the input.
//! This is request interface for the preprocessor.

use serde::Serialize;
use starknet::core::types::FieldElement;

#[derive(Clone, Serialize)]
pub struct Module {
    /// Requested module hash identifier.
    /// This is contract address of the module.
    class_hash: FieldElement,
    /// The input of the module.
    input: Vec<FieldElement>,
}

pub enum ModuleTag {
    TEST,
}

impl Module {
    pub fn from_tag(tag: ModuleTag, input: Vec<FieldElement>) -> Self {
        let class_hash = match tag {
            ModuleTag::TEST => FieldElement::from_hex_be(
                "0x054af96825d987ca89cf320f7c5a8031017815d884cff1592e8ff6da309f3ca6",
            ),
        }
        .expect("Invalid module tag");
        Self { class_hash, input }
    }

    pub fn new(class_hash: FieldElement, input: Vec<FieldElement>) -> Self {
        Self { class_hash, input }
    }

    pub fn get_class_hash(&self) -> FieldElement {
        self.class_hash
    }

    pub fn get_module_input(&self) -> Vec<FieldElement> {
        self.input.clone()
    }
}
