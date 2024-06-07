pub struct ProcessorInput {
    /// Detail sierra code of the module.
    /// This will be loaded to bootloader.
    module_bytes: Vec<u8>,
    /// The input of the module.
    /// Dynamic input from user when calling the module.
    module_input: Vec<u8>,
    /// Fetched proofs per each fetch point.
    proofs: Vec<String>,
}

impl ProcessorInput {
    pub fn new(module_bytes: Vec<u8>, module_input: Vec<u8>, proofs: Vec<String>) -> Self {
        Self {
            module_bytes,
            module_input,
            proofs,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        todo!("Convert ProcessorInput to json")
    }
}
