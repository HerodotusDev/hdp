use reqwest::Url;

/// EVM provider configuration
#[derive(Clone, Debug)]
pub struct ProviderConfig {
    /// RPC url
    pub rpc_url: Url,
    /// Chain id
    pub chain_id: u64,
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
const TEST_CHAIN_ID: u64 = 11155111;
#[cfg(feature = "test_utils")]
const TEST_RPC_URL: &str = "https://eth-sepolia.g.alchemy.com/v2/xar76cftwEtqTBWdF4ZFy9n8FLHAETDv";

#[cfg(feature = "test_utils")]
impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            rpc_url: TEST_RPC_URL.parse().unwrap(),
            chain_id: TEST_CHAIN_ID,
            max_requests: TEST_MAX_REQUESTS,
        }
    }
}
