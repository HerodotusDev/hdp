mod integration_test {
    use std::path::PathBuf;

    use core_new::{
        pre_processor::{PreProcessor, PreProcessorConfig},
        processor::Processor,
    };
    use hdp_primitives::{
        datalake::{
            block_sampled::{BlockSampledCollection, BlockSampledDatalake, HeaderField},
            envelope::DatalakeEnvelope,
            task::{Computation, DatalakeCompute},
        },
        task::TaskEnvelope,
    };
    use hdp_provider::evm::AbstractProviderConfig;
    use starknet::providers::Url;

    // Non-paid personal alchemy endpoint
    const SEPOLIA_RPC_URL: &str =
        "https://eth-sepolia.g.alchemy.com/v2/xar76cftwEtqTBWdF4ZFy9n8FLHAETDv";
    const STARKNET_SEPOLIA_RPC: &str =
        "https://starknet-sepolia.g.alchemy.com/v2/lINonYKIlp4NH9ZI6wvqJ4HeZj7T4Wm6";
    const PREPROCESS_PROGRAM_PATH: &str = "../../build/compiled_cairo/hdp.json";

    fn init_preprocessor() -> PreProcessor {
        let config = PreProcessorConfig {
            module_registry_rpc_url: Url::parse(STARKNET_SEPOLIA_RPC).unwrap(),
            program_path: PathBuf::from(PREPROCESS_PROGRAM_PATH),
        };
        PreProcessor::new_with_config(config)
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
    async fn test_integration() {
        let pre_processor = init_preprocessor();
        let processor = init_processor();
        let start_process = std::time::Instant::now();

        let tasks = vec![
            TaskEnvelope::DatalakeCompute(DatalakeCompute {
                compute: Computation::new("min", None),
                datalake: DatalakeEnvelope::BlockSampled(BlockSampledDatalake {
                    block_range_start: 10001,
                    block_range_end: 10002,
                    increment: 1,
                    sampled_property: BlockSampledCollection::Header(HeaderField::Number),
                }),
            }),
            TaskEnvelope::DatalakeCompute(DatalakeCompute {
                compute: Computation::new("min", None),
                datalake: DatalakeEnvelope::BlockSampled(BlockSampledDatalake {
                    block_range_start: 10003,
                    block_range_end: 10004,
                    increment: 1,
                    sampled_property: BlockSampledCollection::Header(HeaderField::Number),
                }),
            }),
        ];

        let preprocessed_result = pre_processor.process(tasks).await.unwrap();
        let end_process = start_process.elapsed();
        println!("Preprocess time: {:?}", end_process);

        let start_process = std::time::Instant::now();
        let processed_result = processor.process(preprocessed_result).await.unwrap();
        println!("Processed result: {:?}", processed_result);

        let end_process = start_process.elapsed();
        println!("Process time: {:?}", end_process);
    }
}
