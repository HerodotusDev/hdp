use std::env;
use tokio::sync::OnceCell;

pub static CONFIG: OnceCell<Config> = OnceCell::const_new();

pub struct Config {
    pub rpc_url: String,
    pub datalakes: String,
    pub tasks: String,
}

impl Config {
    pub async fn init(
        cli_rpc_url: Option<String>,
        cli_datalakes: Option<String>,
        cli_tasks: Option<String>,
    ) -> &'static Self {
        let rpc_url = cli_rpc_url.unwrap_or_else(|| env::var("RPC_URL").unwrap());
        let datalakes = cli_datalakes.unwrap_or_else(|| env::var("DATALAKES").unwrap());
        let tasks = cli_tasks.unwrap_or_else(|| env::var("TASKS").unwrap());

        CONFIG
            .get_or_init(|| async {
                Config {
                    rpc_url,
                    datalakes,
                    tasks,
                }
            })
            .await
    }
}
