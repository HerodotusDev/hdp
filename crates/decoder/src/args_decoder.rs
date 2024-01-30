use alloy_dyn_abi::DynSolType;
use alloy_primitives::hex::FromHex;
use anyhow::{Ok, Result};
use types::{block_datalake::BlockDatalake, task::ComputationalTask};

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

pub fn datalake_decoder(serialized_datalakes_batch: String) -> Result<Vec<BlockDatalake>> {
    let datalakes_type: DynSolType = "bytes[]".parse()?;
    let bytes = Vec::from_hex(serialized_datalakes_batch).expect("Invalid hex string");
    let serialized_datalakes = datalakes_type.abi_decode(&bytes)?;
    let mut decoded_datalakes = Vec::new();

    if let Some(datalakes) = serialized_datalakes.as_array() {
        for datalake in datalakes {
            println!("datalake dageawgwe: {:?}", datalake);
            let decoded_datalake = BlockDatalake::from_serialized(datalake.as_bytes().unwrap())?;
            println!("decoded_datalake: {:?}", decoded_datalake);
            decoded_datalakes.push(decoded_datalake);
        }
    }

    Ok(decoded_datalakes)
}
