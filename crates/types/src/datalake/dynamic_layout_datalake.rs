use alloy_dyn_abi::DynSolType;
use alloy_primitives::hex::FromHex;
use anyhow::{bail, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicLayoutDatalake {
    pub block_number: usize,
    pub account_address: String,
    pub slot_index: usize,
    pub initial_key: usize,
    pub key_boundry: usize,
    pub increment: usize,
}

impl DynamicLayoutDatalake {
    pub fn new(
        block_number: usize,
        account_address: String,
        slot_index: usize,
        initial_key: usize,
        key_boundry: usize,
        increment: usize,
    ) -> Self {
        Self {
            block_number,
            account_address,
            slot_index,
            initial_key,
            key_boundry,
            increment,
        }
    }

    pub fn from_serialized(serialized: String) -> Result<Self> {
        let datalake_type: DynSolType =
            "(uint256,uint256,address,uint256,uint256,uint256,uint256)".parse()?;
        let bytes = Vec::from_hex(serialized).expect("Invalid hex string");
        let decoded = datalake_type.abi_decode(&bytes)?;

        let value = decoded.as_tuple().unwrap();
        let datalake_code = value[0].as_uint().unwrap().0.to_string().parse::<usize>()?;

        if datalake_code != 1 {
            bail!("Serialized datalake is not a dynamic layout datalake");
        }

        let block_number = value[1].as_uint().unwrap().0.to_string().parse::<usize>()?;
        let account_address = value[2].as_address().unwrap().to_string();
        let slot_index = value[3].as_uint().unwrap().0.to_string().parse::<usize>()?;
        let initial_key = value[4].as_uint().unwrap().0.to_string().parse::<usize>()?;
        let key_boundry = value[5].as_uint().unwrap().0.to_string().parse::<usize>()?;
        let increment = value[6].as_uint().unwrap().0.to_string().parse::<usize>()?;

        Ok(Self {
            block_number,
            account_address,
            slot_index,
            initial_key,
            key_boundry,
            increment,
        })
    }
}

impl Default for DynamicLayoutDatalake {
    fn default() -> Self {
        Self::new(0, "".to_string(), 0, 0, 0, 0)
    }
}
