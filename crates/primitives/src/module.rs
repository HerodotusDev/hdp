//! Module is the unit of pre-processing.
//! It contains the hash and the input.
//! This is request interface for the preprocessor.

use alloy_primitives::{keccak256, Keccak256};
use serde::Serialize;
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

#[serde_as]
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Module {
    #[serde_as(as = "UfeHex")]
    pub class_hash: FieldElement,
    #[serde_as(as = "Vec<UfeHex>")]
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

    pub fn commit(&self) -> String {
        // commit = keccak256(class_hash, keccak256(inputs))
        let input_bytes: Vec<u8> = self.inputs.iter().flat_map(|x| x.to_bytes_be()).collect();
        let commit_input = keccak256(input_bytes);

        let mut hasher = Keccak256::new();
        hasher.update(self.class_hash.to_bytes_be());
        hasher.update(commit_input);

        let commit = hasher.clone().finalize();
        format!("0x{:x}", commit)
    }
}
