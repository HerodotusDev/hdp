use starknet_crypto::FieldElement;

/*
    Module is the unit of pre-processing.
    It contains the hash and the input.
    This is request interface for the preprocessor.
*/

#[derive(Clone)]
pub struct Module {
    /// Requested module hash identifier.
    hash: FieldElement,
    /// The input of the module.
    input: Vec<FieldElement>,
}

impl Module {
    pub fn new(hash: FieldElement, input: Vec<FieldElement>) -> Self {
        Self { hash, input }
    }

    pub fn get_module_hash(&self) -> FieldElement {
        self.hash
    }

    pub fn get_module_input(&self) -> Vec<FieldElement> {
        self.input.clone()
    }
}
