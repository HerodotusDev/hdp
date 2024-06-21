mod integration_test {
    use std::path::PathBuf;

    use hdp_core::{
        compiler::{module::ModuleCompilerConfig, CompilerConfig},
        pre_processor::PreProcessor,
        processor::Processor,
    };
    use hdp_primitives::datalake::{
        block_sampled::{BlockSampledCollection, BlockSampledDatalake, HeaderField},
        envelope::DatalakeEnvelope,
        task::{Computation, DatalakeCompute},
    };
    use hdp_provider::evm::AbstractProviderConfig;
    use starknet::providers::Url;

    // Non-paid personal alchemy endpoint
    const SEPOLIA_RPC_URL: &str =
        "https://eth-sepolia.g.alchemy.com/v2/xar76cftwEtqTBWdF4ZFy9n8FLHAETDv";
    const STARKNET_SEPOLIA_RPC: &str =
        "https://starknet-sepolia.g.alchemy.com/v2/lINonYKIlp4NH9ZI6wvqJ4HeZj7T4Wm6";
    const PREPROCESS_PROGRAM_PATH: &str = "../../build/compiled_cairo/hdp.json";
    const PIE_PATH: &str = "./cairo.pie";

    fn init_preprocessor() -> PreProcessor {
        let module_config = ModuleCompilerConfig {
            module_registry_rpc_url: Url::parse(STARKNET_SEPOLIA_RPC).unwrap(),
            program_path: PathBuf::from(PREPROCESS_PROGRAM_PATH),
        };

        let datalake_config = AbstractProviderConfig {
            rpc_url: SEPOLIA_RPC_URL,
            chain_id: 11155111,
            rpc_chunk_size: 40,
        };
        PreProcessor::new_with_config(CompilerConfig::new(datalake_config, module_config))
    }

    fn init_processor() -> Processor {
        let config = AbstractProviderConfig {
            rpc_url: SEPOLIA_RPC_URL,
            chain_id: 11155111,
            rpc_chunk_size: 40,
        };
        Processor::new(config, PathBuf::from(PREPROCESS_PROGRAM_PATH))
    }

    #[tokio::test]
    async fn test_integration_1() {
        let pre_processor = init_preprocessor();
        let processor = init_processor();
        let start_process = std::time::Instant::now();

        let tasks = vec![
            DatalakeCompute {
                compute: Computation::new("min", None),
                datalake: DatalakeEnvelope::BlockSampled(BlockSampledDatalake {
                    block_range_start: 10001,
                    block_range_end: 10005,
                    increment: 1,
                    sampled_property: BlockSampledCollection::Header(HeaderField::Number),
                }),
            },
            DatalakeCompute {
                compute: Computation::new("avg", None),
                datalake: DatalakeEnvelope::BlockSampled(BlockSampledDatalake {
                    block_range_start: 10003,
                    block_range_end: 10004,
                    increment: 1,
                    sampled_property: BlockSampledCollection::Header(HeaderField::Number),
                }),
            },
        ];

        let preprocessed_result = pre_processor.process(tasks).await.unwrap();
        let preprocessor_end_process = start_process.elapsed();
        println!("Preprocessed result: {:#?}", preprocessed_result);

        let start_process = std::time::Instant::now();
        let processed_result = processor
            .process(preprocessed_result, PIE_PATH.to_string())
            .await
            .unwrap();
        let processor_end_process = start_process.elapsed();
        println!("Processed result: {:#?}", processed_result);

        println!("Preprocess time: {:?}", preprocessor_end_process);
        println!("Process time: {:?}", processor_end_process);
    }
}
