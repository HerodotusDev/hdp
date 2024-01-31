use alloy_dyn_abi::DynSolType;
use alloy_primitives::hex::FromHex;
use anyhow::{bail, Ok, Result};
use common::utils::{bytes_to_hex_string, last_byte_to_u8};
use types::{
    datalake::{block_sampled::BlockSampledDatalake, dynamic_layout::DynamicLayoutDatalake},
    task::ComputationalTask,
    Datalake,
};

pub fn tasks_decoder(serialized_tasks_batch: String) -> Result<Vec<ComputationalTask>> {
    let tasks_type: DynSolType = "bytes[]".parse()?;
    let bytes = Vec::from_hex(serialized_tasks_batch).expect("Invalid hex string");
    let serialized_tasks = tasks_type.abi_decode(&bytes)?;
    let mut decoded_tasks = Vec::new();

    if let Some(tasks) = serialized_tasks.as_array() {
        for task in tasks {
            let computational_task = ComputationalTask::from_serialized(task.as_bytes().unwrap())?;
            decoded_tasks.push(computational_task);
        }
    }

    Ok(decoded_tasks)
}

pub fn datalake_decoder(serialized_datalakes_batch: String) -> Result<Vec<Datalake>> {
    let datalakes_type: DynSolType = "bytes[]".parse()?;
    let bytes = Vec::from_hex(serialized_datalakes_batch).expect("Invalid hex string");
    let serialized_datalakes = datalakes_type.abi_decode(&bytes)?;

    let mut decoded_datalakes = Vec::new();

    if let Some(datalakes) = serialized_datalakes.as_array() {
        for datalake in datalakes {
            let datalake_code = datalake.as_bytes().unwrap().chunks(32).next().unwrap();
            let datalake_string = bytes_to_hex_string(datalake.as_bytes().unwrap());

            let decoded_datalake = match last_byte_to_u8(datalake_code) {
                0 => {
                    Datalake::BlockSampled(BlockSampledDatalake::from_serialized(datalake_string)?)
                }
                1 => Datalake::DynamicLayout(DynamicLayoutDatalake::from_serialized(
                    datalake_string,
                )?),
                _ => Datalake::Unknown,
            };

            if decoded_datalake == Datalake::Unknown {
                bail!("Unknown datalake type");
            }

            decoded_datalakes.push(decoded_datalake);
        }
    }

    Ok(decoded_datalakes)
}
