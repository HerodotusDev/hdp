//! Module is the unit of pre-processing.
//! It contains the hash and the input.
//! This is request interface for the preprocessor.

use starknet::core::types::FieldElement;

#[derive(Clone)]
pub struct Module {
    /// Requested module hash identifier.
    /// This is contract address of the module.
    class_hash: FieldElement,
    /// The input of the module.
    input: Vec<FieldElement>,
}

impl Module {
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
