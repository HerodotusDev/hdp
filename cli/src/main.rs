use clap::Parser;
use decoder::args_decoder::{datalake_decoder, tasks_decoder};
use evaluator::evaluator;

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

    if tasks.len() != datalakes.len() {
        panic!("Tasks and datalakes must have the same length");
    }

    println!("tasks: {:?}\n", tasks);
    println!("datalakes: {:?}\n", datalakes);

    let res = evaluator(tasks, Some(datalakes)).unwrap();
    println!("res: {:?}", res.result);
    println!("rpc_url: {:?}", args.rpc_url);
}
