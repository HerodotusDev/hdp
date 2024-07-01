use alloy::{
    primitives::{Bytes, ChainId},
    transports::http::reqwest::Url,
};

use std::env;
use tokio::sync::OnceCell;

pub static CONFIG: OnceCell<Config> = OnceCell::const_new();

#[derive(Debug)]
pub struct Config {
    pub chain_id: ChainId,
    pub rpc_url: Url,
    pub rpc_chunk_size: u64,
    pub datalakes: Bytes,
    pub tasks: Bytes,
}

impl Config {
    pub async fn init(
        cli_rpc_url: Option<Url>,
        cli_datalakes: Option<Bytes>,
        cli_tasks: Option<Bytes>,
        cli_chain_id: Option<ChainId>,
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
        let datalakes = cli_datalakes.unwrap_or_else(|| {
            env::var("DATALAKES")
                .expect("DATALAKES must be set")
                .parse()
                .expect("DATALAKES must be a valid hex string")
        });
        let tasks = cli_tasks.unwrap_or_else(|| {
            env::var("TASKS")
                .expect("TASKS must be set")
                .parse()
                .expect("TASKS must be a valid hex string")
        });

        CONFIG
            .get_or_init(|| async {
                Config {
                    chain_id,
                    rpc_url,
                    rpc_chunk_size,
                    datalakes,
                    tasks,
                }
            })
            .await
    }
}
