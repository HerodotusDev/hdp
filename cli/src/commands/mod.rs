use crate::commands::run::RunArgs;
use clap::{command, Parser, Subcommand};
use run_datalake::RunDatalakeArgs;
use run_module::RunModuleArgs;

pub mod run;
pub mod run_datalake;
pub mod run_module;

#[derive(Debug, Parser)]
#[command(name = "hdp")]
#[command(version, about, long_about = None)]
pub struct HDPCli {
    #[command(subcommand)]
    pub command: HDPCliCommands,
}

#[derive(Debug, Subcommand)]
pub enum HDPCliCommands {
    /// New to the HDP CLI? Start here!
    #[command(name = "start")]
    Start,
    /// Run single datalake compute
    #[command(name = "run-datalake", arg_required_else_help = true)]
    RunDatalake(RunDatalakeArgs),

    /// Run single module with either program hash or local class path
    #[command(name = "run-module", arg_required_else_help = true)]
    RunModule(RunModuleArgs),
    /// Run batch of tasks base on request json file
    #[command(name = "run", arg_required_else_help = true)]
    Run(RunArgs),
}
