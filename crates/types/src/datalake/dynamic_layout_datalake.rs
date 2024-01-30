use alloy_dyn_abi::DynSolType;
use anyhow::Result;

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

    pub fn from_serialized(serialized: &[u8]) -> Result<Self> {
        let datalake_type: DynSolType =
            "(uint256,address,uint256,uint256,uint256,uint256)".parse()?;

        let decoded = datalake_type.abi_decode(serialized)?;

        let value = decoded.as_tuple().unwrap();
        let block_number = value[0].as_uint().unwrap().0.to_string().parse::<usize>()?;
        let account_address = value[1].as_address().unwrap().to_string();
        let slot_index = value[2].as_uint().unwrap().0.to_string().parse::<usize>()?;
        let initial_key = value[3].as_uint().unwrap().0.to_string().parse::<usize>()?;
        let key_boundry = value[4].as_uint().unwrap().0.to_string().parse::<usize>()?;
        let increment = value[5].as_uint().unwrap().0.to_string().parse::<usize>()?;

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
