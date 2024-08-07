use std::str::FromStr;

use alloy::primitives::Address;

#[tokio::main]
async fn main() {
    //set RUST_LOG into debug
    std::env::set_var("RUST_LOG", "debug");
    // 1. initiate test env, spawn anvil deploy contract on devnet
    //     let module = module_regisry
    //     .get_extended_module_from_class_source_string(
    //         Some(
    //             "0x02aacf92216d1ae71fbdaf3f41865c08f32317b37be18d8c136d442e94cdd823"
    //                 .to_string(),
    //         ),
    //         None,
    //         vec![
    //             "0x4F21E5".to_string(),
    //             "0x4F21E8".to_string(),
    //             "0x13cb6ae34a13a0977f4d7101ebc24b87bb23f0d5".to_string(),
    //         ],
    //     )
    //     .await
    //     .unwrap();

    // let tasks = vec![TaskEnvelope::Module(module)];
    // // 3. prover -> verify ( sharp warpper )
    // let sharp_wrapper = SharpWrapper::new();
    // sharp_wrapper.prove(cairo_pie, Verifier::Solidity).await?;

    // // pull the job status until it get finalized

    // // 4. authenticate
    // let hdp_contract = HdpContract::new();
    // // send transaction
    // hdp_contract.authenticate(authentication_output).await?;
}
