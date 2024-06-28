use alloy::primitives::Bytes;
use hdp_primitives::{
    datalake::{
        block_sampled::BlockSampledDatalake, compute::Computation, envelope::DatalakeEnvelope,
        transactions::TransactionsInBlockDatalake, DatalakeCompute,
    },
    solidity_types::{
        datalake_compute::BatchedDatalakeCompute,
        traits::{BatchedDatalakeComputeCodecs, DatalakeComputeCodecs},
    },
};
use tracing::info;

use crate::{
    command::{DataLakeCommands, HDPCliCommands},
    common::{handle_run, init_cli},
    interactive,
};

pub async fn run() -> anyhow::Result<()> {
    let start_run = std::time::Instant::now();
    let cli = init_cli()?;
    match cli.command {
        HDPCliCommands::Start => {
            interactive::run_interactive().await?;
        }
        HDPCliCommands::Encode {
            allow_run,
            rpc_url,
            chain_id,
            output_file,
            cairo_input,
            pie_file,
            aggregate_fn_id,
            aggregate_fn_ctx,
            command,
        } => {
            let datalake = match command {
                DataLakeCommands::BlockSampled {
                    block_range_start,
                    block_range_end,
                    sampled_property,
                    increment,
                } => {
                    let block_sampled_datalake = BlockSampledDatalake::new(
                        block_range_start,
                        block_range_end,
                        sampled_property,
                        increment,
                    );
                    DatalakeEnvelope::BlockSampled(block_sampled_datalake)
                }
                DataLakeCommands::TransactionsInBlock {
                    target_block,
                    sampled_property,
                    start_index,
                    end_index,
                    increment,
                    included_types,
                } => {
                    let transactions_datalake = TransactionsInBlockDatalake::new(
                        target_block,
                        sampled_property,
                        start_index,
                        end_index,
                        increment,
                        included_types,
                    );
                    DatalakeEnvelope::Transactions(transactions_datalake)
                }
            };
            let target_datalake_compute = DatalakeCompute::new(
                datalake,
                Computation::new(aggregate_fn_id, aggregate_fn_ctx),
            );
            let (encoded_datalakes, encoded_computes) = vec![target_datalake_compute].encode()?;

            // if allow_run is true, then run the evaluator
            if allow_run {
                handle_run(
                    Some(Bytes::from(encoded_computes)),
                    Some(Bytes::from(encoded_datalakes)),
                    rpc_url,
                    chain_id,
                    output_file,
                    cairo_input,
                    pie_file,
                )
                .await?
            }
        }
        HDPCliCommands::Decode { tasks, datalakes } => {
            let decoded_tasks = BatchedDatalakeCompute::decode(&datalakes, &tasks)?;
            info!("Decoded tasks: {:#?}", decoded_tasks);
        }
        HDPCliCommands::DecodeOne { task, datalake } => {
            let decoded_task = DatalakeCompute::decode(&datalake, &task)?;
            info!("Decoded task: {:#?}", decoded_task);
        }
        HDPCliCommands::Run {
            tasks,
            datalakes,
            rpc_url,
            chain_id,
            output_file,
            cairo_input,
            pie_file,
        } => {
            handle_run(
                tasks,
                datalakes,
                rpc_url,
                chain_id,
                output_file,
                cairo_input,
                pie_file,
            )
            .await?
        }
    }
    let duration_run = start_run.elapsed();
    info!("HDP Cli Finished in: {:?}", duration_run);
    Ok(())
}
