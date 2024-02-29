use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::hex::{self, FromHex};
use anyhow::{bail, Ok, Result};
use common::datalake::Datalake;
use common::utils::{bytes_to_hex_string, last_byte_to_u8};
use common::{
    datalake::{block_sampled::BlockSampledDatalake, dynamic_layout::DynamicLayoutDatalake},
    task::ComputationalTask,
};

/// Decode a batch of tasks
pub fn tasks_decoder(serialized_tasks_batch: String) -> Result<Vec<ComputationalTask>> {
    let tasks_type: DynSolType = "bytes[]".parse()?;
    let bytes = Vec::from_hex(serialized_tasks_batch).expect("Invalid hex string");

    let serialized_tasks = tasks_type.abi_decode(&bytes)?;

    let mut decoded_tasks = Vec::new();

    if let Some(tasks) = serialized_tasks.as_array() {
        for task in tasks {
            let computational_task =
                ComputationalTask::deserialize_aggregate_fn(task.as_bytes().unwrap())?;
            decoded_tasks.push(computational_task);
        }
    }

    Ok(decoded_tasks)
}

/// Decode a single task
pub fn task_decoder(serialized_task: String) -> Result<ComputationalTask> {
    let computational_task =
        ComputationalTask::deserialize_aggregate_fn(serialized_task.as_bytes())?;
    Ok(computational_task)
}

/// Decode a batch of datalakes
pub fn datalakes_decoder(serialized_datalakes_batch: String) -> Result<Vec<Datalake>> {
    let datalakes_type: DynSolType = "bytes[]".parse()?;
    let bytes = Vec::from_hex(serialized_datalakes_batch).expect("Invalid hex string");
    let serialized_datalakes = datalakes_type.abi_decode(&bytes)?;

    let mut decoded_datalakes = Vec::new();

    if let Some(datalakes) = serialized_datalakes.as_array() {
        for datalake in datalakes {
            let datalake_code = datalake.as_bytes().unwrap().chunks(32).next().unwrap();
            let datalake_string = bytes_to_hex_string(datalake.as_bytes().unwrap());

            let decoded_datalake = match last_byte_to_u8(datalake_code) {
                0 => Datalake::BlockSampled(BlockSampledDatalake::deserialize(datalake_string)?),
                1 => Datalake::DynamicLayout(DynamicLayoutDatalake::deserialize(datalake_string)?),
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

/// Decode a single datalake
pub fn datalake_decoder(serialized_datalake: String) -> Result<Datalake> {
    let datalake_code = serialized_datalake.as_bytes().chunks(32).next().unwrap();
    let datalake_string = bytes_to_hex_string(serialized_datalake.as_bytes());

    let decoded_datalake = match last_byte_to_u8(datalake_code) {
        0 => Datalake::BlockSampled(BlockSampledDatalake::deserialize(datalake_string)?),
        1 => Datalake::DynamicLayout(DynamicLayoutDatalake::deserialize(datalake_string)?),
        _ => Datalake::Unknown,
    };

    if decoded_datalake == Datalake::Unknown {
        bail!("Unknown datalake type");
    }

    Ok(decoded_datalake)
}

/// Encode a batch of datalakes
pub fn datalakes_encoder(datalakes: Vec<Datalake>) -> Result<String> {
    let mut encoded_datalakes: Vec<DynSolValue> = Vec::new();

    for datalake in datalakes {
        let encoded_datalake = match datalake {
            Datalake::BlockSampled(block_sampled_datalake) => block_sampled_datalake.serialize()?,
            Datalake::DynamicLayout(dynamic_layout_datalake) => {
                dynamic_layout_datalake.serialize()?
            }
            Datalake::Unknown => bail!("Unknown datalake type"),
        };
        let bytes = Vec::from_hex(encoded_datalake).expect("Invalid hex string");
        encoded_datalakes.push(DynSolValue::Bytes(bytes));
    }

    let array_encoded_datalakes = DynSolValue::Array(encoded_datalakes);
    let encoded_datalakes = array_encoded_datalakes.abi_encode();
    let hex_string = hex::encode(encoded_datalakes);
    Ok(format!("0x{}", hex_string))
}

/// Encode batch of tasks
pub fn tasks_encoder(tasks: Vec<ComputationalTask>) -> Result<String> {
    let mut encoded_tasks: Vec<DynSolValue> = Vec::new();

    for task in tasks {
        let encoded_task = task.encode()?;
        let bytes = Vec::from_hex(encoded_task).expect("Invalid hex string");
        encoded_tasks.push(DynSolValue::Bytes(bytes));
    }

    let array_encoded_tasks = DynSolValue::Array(encoded_tasks);
    let encoded_tasks = array_encoded_tasks.abi_encode();
    let hex_string = hex::encode(encoded_tasks);
    Ok(format!("0x{}", hex_string))
}
