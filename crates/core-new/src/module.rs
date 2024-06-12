//! Module is the unit of pre-processing.
//! It contains the hash and the input.
//! This is request interface for the preprocessor.

use serde::Serialize;
use starknet_crypto::FieldElement;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Module {
    pub class_hash: FieldElement,
    pub inputs: Vec<FieldElement>,
}

pub enum ModuleTag {
    TEST,
}

impl Module {
    pub fn from_tag(tag: ModuleTag, inputs: Vec<FieldElement>) -> Self {
        let class_hash = match tag {
            ModuleTag::TEST => FieldElement::from_hex_be(
                "0x054af96825d987ca89cf320f7c5a8031017815d884cff1592e8ff6da309f3ca6",
            ),
        }
        .expect("Invalid module tag");
        Self { class_hash, inputs }
    }

    pub fn new(class_hash: FieldElement, inputs: Vec<FieldElement>) -> Self {
        Self { class_hash, inputs }
    }

    pub fn get_class_hash(&self) -> FieldElement {
        self.class_hash
    }

    pub fn get_module_inputs(&self) -> Vec<FieldElement> {
        self.inputs.clone()
    }
}
