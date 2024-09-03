use crate::provider::{config::ProviderConfig, indexer::Indexer};

use super::rpc::RpcProvider;

pub struct StarknetProvider {
    /// Account and storage trie provider
    pub(crate) rpc_provider: RpcProvider,
    /// Header provider
    pub(crate) header_provider: Indexer,
}

#[cfg(feature = "test_utils")]
impl Default for StarknetProvider {
    fn default() -> Self {
        Self::new(&ProviderConfig::default())
    }
}

impl StarknetProvider {
    pub fn new(config: &ProviderConfig) -> Self {
        let rpc_provider = RpcProvider::new(config.rpc_url.to_owned(), config.max_requests);
        let indexer = Indexer::new(config.chain_id);
        Self {
            rpc_provider,
            header_provider: indexer,
        }
    }
}
