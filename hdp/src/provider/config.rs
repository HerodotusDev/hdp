use reqwest::Url;

use crate::primitives::ChainId;

/// EVM provider configuration
#[derive(Clone, Debug)]
pub struct ProviderConfig {
    /// provider url
    pub provider_url: Url,
    /// Chain id
    pub chain_id: ChainId,
    /// Max number of requests to send in parallel
    ///
    /// For default, it is set to 100
    /// For archive node, recommend to set it to 1000
    /// This will effect fetch speed of account, storage proofs
    pub max_requests: u64,
}

/// This is optimal max number of requests to send in parallel when using non-paid alchemy rpc url
#[cfg(feature = "test_utils")]
pub const TEST_MAX_REQUESTS: u64 = 100;
#[cfg(feature = "test_utils")]
use lazy_static::lazy_static;

#[cfg(feature = "test_utils")]
lazy_static! {
    static ref TEST_RPC_URL: String = std::env::var("PROVIDER_URL_ETHEREUM_SEPOLIA")
        .expect("Environment variable PROVIDER_URL_ETHEREUM_SEPOLIA not set");
}

#[cfg(feature = "test_utils")]
impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider_url: TEST_RPC_URL.parse().unwrap(),
            chain_id: ChainId::EthereumSepolia,
            max_requests: TEST_MAX_REQUESTS,
        }
    }
}
