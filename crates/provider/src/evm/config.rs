use hdp_primitives::config::AllChainConfigs;
use reqwest::Url;

/// This is optimal max number of requests to send in parallel when using non-paid alchemy rpc url
pub const DEFAULT_MAX_REQUESTS: u32 = 100;
const DEFAULT_CHAIN_ID: u64 = 11155111;
const DEFAULT_RPC_URL: &str =
    "https://eth-sepolia.g.alchemy.com/v2/xar76cftwEtqTBWdF4ZFy9n8FLHAETDv";

/// EVM provider configuration
#[derive(Clone, Debug)]
pub struct EvmProviderConfig {
    /// RPC url
    pub rpc_url: Url,
    /// Chain id
    pub chain_id: u64,
    /// Max number of requests to send in parallel
    ///
    /// For default, it is set to 100
    /// For archive node, recommend to set it to 1000
    /// This will effect fetch speed of account, storage proofs
    pub max_requests: u32,
}

impl Default for EvmProviderConfig {
    fn default() -> Self {
        Self {
            rpc_url: DEFAULT_RPC_URL.parse().unwrap(),
            chain_id: DEFAULT_CHAIN_ID,
            max_requests: DEFAULT_MAX_REQUESTS,
        }
    }
}

impl EvmProviderConfig {
    pub fn from_chains_config(chains_config: &AllChainConfigs, chain_id: u64) -> Self {
        let chain_config = chains_config.get(&chain_id).unwrap();
        Self {
            rpc_url: chain_config.rpc_url.parse().unwrap(),
            chain_id,
            max_requests: chain_config.rpc_chunk_size,
        }
    }
}
