use hdp::{
    hdp_run::{self, HdpRunConfig},
    preprocessor::module_registry::ModuleRegistry,
    primitives::task::{module::Module, TaskEnvelope},
};

#[tokio::main]
async fn main() {
    //set RUST_LOG into debug
    std::env::set_var("RUST_LOG", "debug");
    // 1. initiate test env, spawn anvil deploy contract on devnet
    let request = r#"
    {
    "programHash": "0x64041a339b1edd10de83cf031cfa938645450f971d2527c90d4c2ce68d7d412",
    "inputs": [
        {
        "visibility": "private",
        "value": "0x5222a4"
        },
        {
        "visibility": "public",
        "value": "0x00000000000000000000000013cb6ae34a13a0977f4d7101ebc24b87bb23f0d5"
        }
    ] 
     }  
     "#;
    let requested_module: Module = serde_json::from_str(request).unwrap();
    let module_regisry = ModuleRegistry::new();
    let module = module_regisry
        .get_extended_module_from_class_source(
            Some(requested_module.program_hash),
            None,
            requested_module.inputs,
        )
        .await
        .unwrap();

    let tasks = vec![TaskEnvelope::Module(module)];

    let hdp_run_config = HdpRunConfig {
        dry_run_program_path: "./build/contract_dry_run.json".into(),
        sound_run_program_path: "./build/hdp.json".into(),
        ..Default::default()
    };

    let pre_processor_output_file = "input.json";
    let output_file = "output.json";
    let cairo_pie_file = "pie.zip";

    hdp_run::hdp_run(
        &hdp_run_config,
        tasks,
        Some(pre_processor_output_file.into()),
        Some(output_file.into()),
        Some(cairo_pie_file.into()),
    )
    .await
    .unwrap();
    // // 3. prover -> verify ( sharp warpper )
    // let sharp_wrapper = SharpWrapper::new();
    // sharp_wrapper.prove(cairo_pie, Verifier::Solidity).await?;

    // // pull the job status until it get finalized

    // // 4. authenticate
    // let hdp_contract = HdpContract::new();
    // // send transaction
    // hdp_contract.authenticate(authentication_output).await?;
}
