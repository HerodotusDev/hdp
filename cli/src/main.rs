use clap::Parser;
use decoder::args_decoder::{datalake_decoder, tasks_decoder};

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short = 't', long)]
    #[arg(value_name = "TASKS")]
    #[arg(help = "The tasks to process")]
    tasks: String,

    #[arg(short = 'd', long)]
    #[arg(value_name = "DATA_LAKES")]
    #[arg(help = "The data lakes to use")]
    datalakes: String,

    #[arg(short = 'r', long)]
    #[arg(value_name = "RPC_URL")]
    #[arg(help = "The RPC URL to use")]
    rpc_url: Option<String>,
}

fn main() {
    let args = Cli::parse();
    let tasks = tasks_decoder(args.tasks).unwrap();
    let datalakes = datalake_decoder(args.datalakes).unwrap();
    println!("tasks: {:?}", tasks);
    println!("datalakes: {:?}", datalakes);
    println!("rpc_url: {:?}", args.rpc_url);
}
