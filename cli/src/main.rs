use clap::Parser;
use common::config::Config;
use decoder::args_decoder::{datalake_decoder, tasks_decoder};
use evaluator::evaluator;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short = 't', long)]
    #[arg(value_name = "TASKS")]
    #[arg(help = "The tasks to process")]
    tasks: Option<String>,

    #[arg(short = 'd', long)]
    #[arg(value_name = "DATA_LAKES")]
    #[arg(help = "The data lakes to use")]
    datalakes: Option<String>,

    #[arg(short = 'r', long)]
    #[arg(value_name = "RPC_URL")]
    #[arg(help = "The RPC URL to use")]
    rpc_url: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    dotenv::dotenv().ok();
    let config = Config::init(args.rpc_url, args.datalakes, args.tasks).await;
    // let abstract_fetcher = AbstractFetcher::new(config.rpc_url.clone());
    let tasks = tasks_decoder(config.tasks.clone()).unwrap();
    let datalakes = datalake_decoder(config.datalakes.clone()).unwrap();

    if tasks.len() != datalakes.len() {
        panic!("Tasks and datalakes must have the same length");
    }

    println!("tasks: {:?}\n", tasks);
    println!("datalakes: {:?}\n", datalakes);

    let res = evaluator(tasks, Some(datalakes)).await.unwrap();
    println!("res: {:?}", res.result);
    println!("rpc_url: {:?}", config.rpc_url);
}
