use alloy::{primitives::ChainId, transports::http::reqwest::Url};
use hdp_provider::evm::config::EvmProviderConfig;

use std::env;
use tokio::sync::OnceCell;

pub static MODULE_CONFIG: OnceCell<ModuleConfig> = OnceCell::const_new();

/// Configuration for the CLI
#[derive(Debug)]
pub struct ModuleConfig {
    pub evm_provider: EvmProviderConfig,
    pub module_registry_rpc_url: Url,
}

impl ModuleConfig {
    pub async fn init(
        cli_rpc_url: Option<Url>,
        cli_chain_id: Option<ChainId>,
        cli_module_registry_rpc_url: Option<Url>,
    ) -> &'static Self {
        let chain_id = cli_chain_id.unwrap_or_else(|| {
            env::var("CHAIN_ID")
                .expect("CHAIN_ID must be set")
                .parse()
                .expect("CHAIN_ID must be a number")
        });
        let rpc_url = cli_rpc_url.unwrap_or_else(|| {
            env::var("RPC_URL")
                .expect("RPC_URL must be set")
                .parse()
                .expect("RPC_URL must be a valid URL")
        });
        let rpc_chunk_size = env::var("RPC_CHUNK_SIZE")
            .unwrap_or_else(|_| "40".to_string())
            .parse()
            .expect("RPC_CHUNK_SIZE must be a number");
        let module_registry_rpc_url = cli_module_registry_rpc_url.unwrap_or_else(|| {
            env::var("MODULE_REGISTRY_RPC_URL")
                .expect("MODULE_REGISTRY_RPC_URL must be set")
                .parse()
                .expect("MODULE_REGISTRY_RPC_URL must be a valid URL")
        });

        MODULE_CONFIG
            .get_or_init(|| async {
                ModuleConfig {
                    evm_provider: EvmProviderConfig {
                        rpc_url,
                        chain_id,
                        max_requests: rpc_chunk_size,
                    },
                    module_registry_rpc_url,
                }
            })
            .await
    }
}
