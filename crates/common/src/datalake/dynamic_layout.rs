use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{hex::FromHex, keccak256, U256};
use anyhow::{bail, Result};

use crate::compiler::test::test_closer;

use super::{
    base::{DataPoint, DatalakeBase, Derivable},
    Datalake,
};

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicLayoutDatalake {
    pub block_number: usize,
    pub account_address: String,
    pub slot_index: usize,
    pub initial_key: usize,
    pub key_boundry: usize,
    pub increment: usize,
}

impl ToString for DynamicLayoutDatalake {
    fn to_string(&self) -> String {
        let blocknumber = DynSolValue::Uint(U256::from(self.block_number), 256);
        let account_address = DynSolValue::Address(self.account_address.parse().unwrap());
        let slot_index = DynSolValue::Uint(U256::from(self.slot_index), 256);
        let initial_key = DynSolValue::Uint(U256::from(self.initial_key), 256);
        let key_boundry = DynSolValue::Uint(U256::from(self.key_boundry), 256);
        let increment = DynSolValue::Uint(U256::from(self.increment), 256);
        let tuple_value = DynSolValue::Tuple(vec![
            blocknumber,
            account_address,
            slot_index,
            initial_key,
            key_boundry,
            increment,
        ]);
        let encoded_datalake = tuple_value.abi_encode();
        let hash = keccak256(encoded_datalake);
        format!("0x{:x}", hash)
    }
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

    pub async fn compile(&self) -> Result<Vec<DataPoint>> {
        test_closer().await?;
        Ok(vec![])
    }
}

impl Default for DynamicLayoutDatalake {
    fn default() -> Self {
        Self::new(0, "".to_string(), 0, 0, 0, 0)
    }
}

impl Derivable for DynamicLayoutDatalake {
    fn derive(&self) -> DatalakeBase {
        DatalakeBase::new(
            self.to_string().as_str(),
            Datalake::DynamicLayout(self.clone()),
        )
    }
}
