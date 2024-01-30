use alloy_dyn_abi::DynSolType;
use alloy_primitives::hex::FromHex;
use anyhow::{Ok, Result};
use types::task::ComputationalTask;

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
