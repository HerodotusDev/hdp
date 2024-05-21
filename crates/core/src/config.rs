use std::env;
use tokio::sync::OnceCell;

pub static CONFIG: OnceCell<Config> = OnceCell::const_new();

pub struct Config {
    pub chain_id: u64,
    pub rpc_url: String,
    pub rpc_account_chunk_size: u64,
    pub rpc_storage_chunk_size: u64,
    pub datalakes: String,
    pub tasks: String,
}

impl Config {
    pub async fn init(
        cli_rpc_url: Option<String>,
        cli_datalakes: Option<String>,
        cli_tasks: Option<String>,
        cli_chain_id: Option<u64>,
    ) -> &'static Self {
        let chain_id =
            cli_chain_id.unwrap_or_else(|| env::var("CHAIN_ID").unwrap().parse().unwrap());
        let rpc_url = cli_rpc_url.unwrap_or_else(|| env::var("RPC_URL").unwrap());
        let rpc_account_chunk_size = env::var("RPC_ACCOUNT_CHUNK_SIZE")
            .unwrap_or_else(|_| "40".to_string())
            .parse()
            .unwrap();
        let rpc_storage_chunk_size = env::var("RPC_STORAGE_CHUNK_SIZE")
            .unwrap_or_else(|_| "40".to_string())
            .parse()
            .unwrap();
        let datalakes = cli_datalakes.unwrap_or_else(|| env::var("DATALAKES").unwrap());
        let tasks = cli_tasks.unwrap_or_else(|| env::var("TASKS").unwrap());

        CONFIG
            .get_or_init(|| async {
                Config {
                    chain_id,
                    rpc_url,
                    rpc_account_chunk_size,
                    rpc_storage_chunk_size,
                    datalakes,
                    tasks,
                }
            })
            .await
    }
}
