use hdp::{
    hdp_run::HdpRunConfig,
    preprocessor::module_registry::ModuleRegistry,
    primitives::task::{
        module::{ModuleInput, Visibility},
        TaskEnvelope,
    },
};

#[tokio::main]
async fn main() {
    //set RUST_LOG into debug
    std::env::set_var("RUST_LOG", "debug");

    let module_regisry = ModuleRegistry::new();
    let module = module_regisry
        .get_extended_module_from_class_source(
            None,
            Some("./private_module/target/dev/private_module_get_balance.compiled_contract_class.json".into()),
            vec![
                ModuleInput::new(Visibility::Private, "0x5222a4"),
                ModuleInput::new(Visibility::Public, "0x00000000000000000000000013cb6ae34a13a0977f4d7101ebc24b87bb23f0d5" )
            ],
        )
        .await
        .unwrap();
    let tasks = vec![TaskEnvelope::Module(module)];
    let pre_processor_output_file = "input.json";
    let output_file = "output.json";
    let cairo_pie_file = "pie.zip";

    let hdp_run_config = HdpRunConfig {
        dry_run_program_path: "../../build/contract_dry_run.json".into(),
        sound_run_program_path: "../../build/hdp.json".into(),
        program_input_file: pre_processor_output_file.into(),
        batch_proof_file: Some(output_file.into()),
        cairo_pie_file: Some(cairo_pie_file.into()),
        ..Default::default()
    };

    hdp::run(&hdp_run_config, tasks).await.unwrap();
}
