use r#type::PreProcessorInput;

pub mod r#type;
/*
    InputGenerator is responsible for generating the input file for the runner.
    All runner requires input.json file to run target cairo program.
    This struct is responsible for generating the input struct, which able to converted to json.
*/
pub struct InputGenerator {}

impl InputGenerator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn pre_run_input(&self) -> PreProcessorInput {
        todo!("Generate input for pre-runner")
    }
}
