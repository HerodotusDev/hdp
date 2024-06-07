/*
  configuration for the pre-processor
*/
pub struct PreProcessorConfig {
    module_hash: String,
    module_bytes: Vec<u8>,
}

impl PreProcessorConfig {
    pub fn new(module_hash: String) -> Self {
        // TODO: Load module bytes from the module hash
        let module_bytes = vec![];
        Self {
            module_hash,
            module_bytes,
        }
    }

    pub fn module_hash(&self) -> &str {
        &self.module_hash
    }

    pub fn module_bytes(&self) -> &[u8] {
        &self.module_bytes
    }
}
