use evm::provider::EvmProvider;

use super::{config::ProviderConfig, ProofProvider};

pub mod error;
pub mod evm;
pub mod starknet;

pub fn new_provider_from_config(config: &ProviderConfig) -> Box<dyn ProofProvider> {
    match config.chain_id {
        1 | 11155111 => Box::new(EvmProvider::new(config)),
        // TODO: change chain_id to string
        _ => panic!("not supported chain id"),
    }
}
