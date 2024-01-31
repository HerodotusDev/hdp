use clap::Parser;
use decoder::args_decoder::{datalake_decoder, tasks_decoder};
use types::{datalake::base::Derivable, Datalake};

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
    let mut tasks = tasks_decoder(args.tasks).unwrap();
    let mut datalakes = datalake_decoder(args.datalakes).unwrap();

    if tasks.len() != datalakes.len() {
        panic!("Tasks and datalakes must have the same length");
    }

    for (datalake_idx, datalake) in datalakes.iter_mut().enumerate() {
        let task = &mut tasks[datalake_idx];

        task.datalake = match datalake {
            Datalake::BlockSampled(block_datalake) => Some(block_datalake.derive()),
            Datalake::DynamicLayout(dynamic_layout_datalake) => {
                Some(dynamic_layout_datalake.derive())
            }
            _ => None,
        };

        task.datalake.as_mut().unwrap().compile();
    }
    println!("tasks: {:?}", tasks);
    println!("datalakes: {:?}", datalakes);
    println!("rpc_url: {:?}", args.rpc_url);
}
