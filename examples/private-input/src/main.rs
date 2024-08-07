use std::str::FromStr;

use alloy::primitives::Address;

#[tokio::main]
async fn main() {
    //set RUST_LOG into debug
    std::env::set_var("RUST_LOG", "debug");
    // 1. initiate test env, spawn anvil deploy contract on devnet

    // // 3. prover -> verify ( sharp warpper )
    // let sharp_wrapper = SharpWrapper::new();
    // sharp_wrapper.prove(cairo_pie, Verifier::Solidity).await?;

    // // pull the job status until it get finalized

    // // 4. authenticate
    // let hdp_contract = HdpContract::new();
    // // send transaction
    // hdp_contract.authenticate(authentication_output).await?;
}
