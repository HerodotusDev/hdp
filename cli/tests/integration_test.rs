mod integration_test {
    use alloy::transports::http::reqwest::Url;
    use hdp_preprocessor::{
        compile::config::CompilerConfig, module_registry::ModuleRegistry, PreProcessor,
    };
    use hdp_primitives::{
        aggregate_fn::AggregationFunction,
        processed_types::cairo_format::AsCairoFormat,
        task::{
            datalake::{
                block_sampled::{BlockSampledCollection, BlockSampledDatalake, HeaderField},
                compute::Computation,
                envelope::DatalakeEnvelope,
                DatalakeCompute,
            },
            TaskEnvelope,
        },
    };
    use hdp_processor::Processor;

    use std::{fs, path::PathBuf};

    const STARKNET_SEPOLIA_RPC: &str =
        "https://starknet-sepolia.g.alchemy.com/v2/lINonYKIlp4NH9ZI6wvqJ4HeZj7T4Wm6";
    const DRY_RUN_PROGRAM_PATH: &str = "../build/compiled_cairo/contract_dry_run.json";
    const PREPROCESS_PROGRAM_PATH: &str = "../build/compiled_cairo/hdp.json";
    const PIE_PATH: &str = "./cairo.pie";

    fn init_preprocessor() -> PreProcessor {
        let compile_config = CompilerConfig::default()
            .with_dry_run_program_path(PathBuf::from(DRY_RUN_PROGRAM_PATH));

        PreProcessor::new_with_config(compile_config)
    }

    fn init_processor() -> Processor {
        Processor::new(PathBuf::from(PREPROCESS_PROGRAM_PATH))
    }

    #[ignore = "ignore for now"]
    #[tokio::test]
    async fn test_integration_1() {
        let pre_processor = init_preprocessor();
        let processor = init_processor();
        let start_process = std::time::Instant::now();

        let tasks = vec![
            TaskEnvelope::DatalakeCompute(DatalakeCompute {
                compute: Computation::new(AggregationFunction::MIN, None),
                datalake: DatalakeEnvelope::BlockSampled(BlockSampledDatalake {
                    block_range_start: 10001,
                    block_range_end: 10005,
                    increment: 1,
                    sampled_property: BlockSampledCollection::Header(HeaderField::Number),
                }),
            }),
            TaskEnvelope::DatalakeCompute(DatalakeCompute {
                compute: Computation::new(AggregationFunction::AVG, None),
                datalake: DatalakeEnvelope::BlockSampled(BlockSampledDatalake {
                    block_range_start: 10003,
                    block_range_end: 10004,
                    increment: 1,
                    sampled_property: BlockSampledCollection::Header(HeaderField::Number),
                }),
            }),
        ];

        let preprocessed_result = pre_processor.process(tasks).await.unwrap();
        let preprocessor_end_process = start_process.elapsed();
        println!("Preprocessed result: {:#?}", preprocessed_result);

        // write
        fs::write(
            "preprocessed_result.json",
            serde_json::to_string_pretty(&preprocessed_result.as_cairo_format()).unwrap(),
        )
        .expect("Unable to write file");

        let start_process = std::time::Instant::now();
        let processed_result = processor
            .process(preprocessed_result, PathBuf::from(PIE_PATH))
            .await
            .unwrap();
        let processor_end_process = start_process.elapsed();
        println!("Processed result: {:#?}", processed_result);

        println!("Preprocess time: {:?}", preprocessor_end_process);
        println!("Process time: {:?}", processor_end_process);
    }

    #[ignore = "ignore for now"]
    #[tokio::test]
    async fn test_integration_2() {
        let pre_processor = init_preprocessor();
        let processor = init_processor();
        let start_process = std::time::Instant::now();

        let tasks = vec![
            TaskEnvelope::DatalakeCompute(DatalakeCompute {
                compute: Computation::new(AggregationFunction::MIN, None),
                datalake: DatalakeEnvelope::BlockSampled(BlockSampledDatalake {
                    block_range_start: 10001,
                    block_range_end: 10005,
                    increment: 1,
                    sampled_property: BlockSampledCollection::Header(HeaderField::Number),
                }),
            }),
            // TaskEnvelope::Module(Module::from_tag(
            //     ModuleTag::AccountBalanceExample,
            //     vec![felt!("1"), felt!("0")],
            // )),
        ];

        let preprocessed_result = pre_processor.process(tasks).await.unwrap();
        let preprocessor_end_process = start_process.elapsed();
        println!("Preprocessed result: {:#?}", preprocessed_result);

        // write
        fs::write(
            "preprocessed_result2.json",
            serde_json::to_string_pretty(&preprocessed_result.as_cairo_format()).unwrap(),
        )
        .expect("Unable to write file");

        let start_process = std::time::Instant::now();
        let processed_result = processor
            .process(preprocessed_result, PathBuf::from(PIE_PATH))
            .await
            .unwrap();
        let processor_end_process = start_process.elapsed();
        println!("Processed result: {:#?}", processed_result);

        println!("Preprocess time: {:?}", preprocessor_end_process);
        println!("Process time: {:?}", processor_end_process);
    }

    #[ignore = "ignore for now"]
    #[tokio::test]
    async fn test_integration_3() {
        let pre_processor = init_preprocessor();
        let processor = init_processor();
        let start_process = std::time::Instant::now();

        let url = Url::parse(STARKNET_SEPOLIA_RPC).unwrap();
        let module_regisry = ModuleRegistry::new(url);
        let module = module_regisry
            .get_extended_module_from_class_source_string(
                Some(
                    "0x02aacf92216d1ae71fbdaf3f41865c08f32317b37be18d8c136d442e94cdd823"
                        .to_string(),
                ),
                None,
                vec![
                    "0x4F21E5".to_string(),
                    "0x4F21E8".to_string(),
                    "0x13cb6ae34a13a0977f4d7101ebc24b87bb23f0d5".to_string(),
                ],
            )
            .await
            .unwrap();

        let tasks = vec![TaskEnvelope::Module(module)];

        let preprocessed_result = pre_processor.process(tasks).await.unwrap();
        let preprocessor_end_process = start_process.elapsed();
        println!("Preprocessed result: {:#?}", preprocessed_result);

        // write
        fs::write(
            "preprocessed_result3.json",
            serde_json::to_string_pretty(&preprocessed_result.as_cairo_format()).unwrap(),
        )
        .expect("Unable to write file");

        let start_process = std::time::Instant::now();
        let processed_result = processor
            .process(preprocessed_result, PathBuf::from(PIE_PATH))
            .await
            .unwrap();
        let processor_end_process = start_process.elapsed();
        println!("Processed result: {:#?}", processed_result);

        println!("Preprocess time: {:?}", preprocessor_end_process);
        println!("Process time: {:?}", processor_end_process);
    }
}
