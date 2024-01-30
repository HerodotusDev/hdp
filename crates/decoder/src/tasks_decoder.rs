use alloy_dyn_abi::DynSolType;
use anyhow::{Ok, Result};

pub fn decode(tasks: String) -> Result<()> {
    let tasks_type: DynSolType = "bytes[]".parse().unwrap();
    let decoded = tasks_type.abi_decode(tasks.as_bytes())?;

    Ok(())
}
