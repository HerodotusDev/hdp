use crate::primitives::processed_types::block_proofs::ProcessedBlockProofs;
use std::future::Future;
use std::pin::Pin;

use super::config::ProviderConfig;
use super::error::ProviderError;
use super::evm::provider::EvmProvider;
use super::key::CategorizedFetchKeys;
use super::types::FetchedDatalake;

pub type FetchProofsResult = Result<FetchedDatalake, ProviderError>;
pub type FetchProofsFromKeysResult = Result<ProcessedBlockProofs, ProviderError>;

pub type AsyncResult<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait ProofProvider: Send + Sync {
    fn fetch_proofs<'a>(
        &'a self,
        datalake: &'a crate::primitives::task::datalake::DatalakeCompute,
    ) -> AsyncResult<FetchProofsResult>;

    fn fetch_proofs_from_keys(
        &self,
        keys: CategorizedFetchKeys,
    ) -> AsyncResult<FetchProofsFromKeysResult>;
}

pub fn new_provider_from_config(config: &ProviderConfig) -> Box<dyn ProofProvider> {
    match config.chain_id {
        1 | 11155111 => Box::new(EvmProvider::new(config)),
        // TODO: change chain_id to string
        _ => panic!("not supported chain id"),
    }
}
