pub struct PreProcessorInput {
    /// Detail sierra code of the module.
    /// This will be loaded to bootloader.
    module_bytes: Vec<u8>,
    /// The input of the module.
    /// Dynamic input from user when calling the module.
    module_input: Vec<u8>,
}

impl PreProcessorInput {
    pub fn new(module_hash: String, module_bytes: Vec<u8>, module_input: Vec<u8>) -> Self {
        Self {
            module_bytes,
            module_input,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        todo!("Convert PreProcessorInput to json")
    }
}
