//!  Preprocessor is reponsible for identifying the required values.
//!  This will be most abstract layer of the preprocessor.

use std::collections::HashSet;
use std::path::PathBuf;

use crate::compiler::module::ModuleCompilerConfig;
use crate::compiler::Compiler;

use anyhow::{Ok, Result};
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use hdp_primitives::{datalake::task::DatalakeCompute, module::Module, task::TaskEnvelope};
use hdp_provider::key::FetchKeyEnvelope;

use starknet::providers::Url;
use tracing::info;

pub struct PreProcessor {
    /// compiler
    compiler: Compiler,
}

/// [`ExtendedTask`] is a structure that contains the task commitment, aggregate values set, compute and module class
/// This structure is used to provide the task to the processor
pub enum ExtendedTask {
    DatalakeCompute(ExtendedDatalake),
    Module(ExtendedModule),
}

pub struct ExtendedDatalake {
    pub task: DatalakeCompute,
    pub fetch_keys_set: Vec<FetchKeyEnvelope>,
}

#[derive(Clone, Debug)]
pub struct ExtendedModule {
    pub task: Module,
    pub module_class: CasmContractClass,
}

pub struct PreProcessorConfig {
    // rpc url to fetch the module class from starknet
    pub module_registry_rpc_url: Url,
    // pre-run program path
    pub program_path: PathBuf,
}

pub struct PreProcessResult {
    /// Fetch points are the values that are required to run the module
    pub fetch_keys: HashSet<FetchKeyEnvelope>,
    /// Tasks that are extended with relevant information for processor
    pub tasks: Vec<ExtendedTask>,
}

impl PreProcessor {
    pub fn new_with_config(config: PreProcessorConfig) -> Self {
        let rpc_url = config.module_registry_rpc_url;
        let program_path = config.program_path;
        let compiler = Compiler::new(ModuleCompilerConfig {
            module_registry_rpc_url: rpc_url,
            program_path: program_path.clone(),
        });
        Self { compiler }
    }

    /// User request is pass as input of this function,
    /// First it will generate input structure for preprocessor that need to pass to runner
    /// Then it will run the preprocessor and return the result, fetch points
    /// Fetch points are the values that are required to run the module
    pub async fn process(&self, tasks: Vec<TaskEnvelope>) -> Result<PreProcessResult> {
        let (fetch_keys, extended_tasks) = self.compiler.compile(tasks).await?;
        info!("Preprocessor completed successfully");
        Ok(PreProcessResult {
            fetch_keys,
            tasks: extended_tasks,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hdp_primitives::datalake::block_sampled::{
        BlockSampledCollection, BlockSampledDatalake, HeaderField,
    };
    use hdp_primitives::datalake::envelope::DatalakeEnvelope;
    use hdp_primitives::datalake::task::Computation;
    use hdp_primitives::module::{Module, ModuleTag};
    use starknet::macros::felt;
    use starknet::providers::Url;
    use std::path::PathBuf;

    const STARKNET_SEPOLIA_RPC: &str =
        "https://starknet-sepolia.g.alchemy.com/v2/lINonYKIlp4NH9ZI6wvqJ4HeZj7T4Wm6";
    const PREPROCESS_PROGRAM_PATH: &str = "../build/compiled_cairo/hdp.json";

    #[tokio::test]
    async fn test_process_only_datalake() {
        let start_process = std::time::Instant::now();
        let config = PreProcessorConfig {
            module_registry_rpc_url: Url::parse(STARKNET_SEPOLIA_RPC).unwrap(),
            program_path: PathBuf::from("../build/compiled_cairo/hdp.json"),
        };
        let pre_processor = PreProcessor::new_with_config(config);

        let tasks = vec![
            TaskEnvelope::DatalakeCompute(DatalakeCompute {
                compute: Computation::new("min", None),
                datalake: DatalakeEnvelope::BlockSampled(BlockSampledDatalake {
                    block_range_start: 1000,
                    block_range_end: 10000,
                    increment: 1,
                    sampled_property: BlockSampledCollection::Header(HeaderField::Number),
                }),
            }),
            TaskEnvelope::DatalakeCompute(DatalakeCompute {
                compute: Computation::new("min", None),
                datalake: DatalakeEnvelope::BlockSampled(BlockSampledDatalake {
                    block_range_start: 1000,
                    block_range_end: 10000,
                    increment: 1,
                    sampled_property: BlockSampledCollection::Header(HeaderField::Number),
                }),
            }),
        ];

        let result = pre_processor.process(tasks).await.unwrap();

        let end_process = start_process.elapsed();
        println!("Process time: {:?}", end_process);
        assert_eq!(result.fetch_keys.len(), 9000);
        assert_eq!(result.tasks.len(), 2);
        assert!(matches!(&result.tasks[0], ExtendedTask::DatalakeCompute(_)));
    }

    #[tokio::test]
    async fn test_process_only_module() {
        let start_process = std::time::Instant::now();
        let config = PreProcessorConfig {
            module_registry_rpc_url: Url::parse(STARKNET_SEPOLIA_RPC).unwrap(),
            program_path: PathBuf::from(PREPROCESS_PROGRAM_PATH),
        };
        let pre_processor = PreProcessor::new_with_config(config);

        let module = Module::from_tag(ModuleTag::TEST, vec![felt!("1"), felt!("2")]);
        let tasks = vec![TaskEnvelope::Module(module)];

        let result = pre_processor.process(tasks).await.unwrap();
        let end_process = start_process.elapsed();
        println!("Process time: {:?}", end_process);
        assert_eq!(result.fetch_keys.len(), 0);
        assert_eq!(result.tasks.len(), 1);
        assert!(matches!(&result.tasks[0], ExtendedTask::Module(_)));
    }
}
