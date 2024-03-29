use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{hex::FromHex, keccak256, U256};
use anyhow::{bail, Result};

use crate::compiler::test::test_closer;
use hdp_primitives::utils::bytes_to_hex_string;

use super::{
    base::{DatalakeBase, Derivable},
    Datalake,
};

// TODO: DynamicLayoutDatalake is incomplete
#[derive(Debug, Clone, PartialEq)]
pub struct DynamicLayoutDatalake {
    pub block_number: u64,
    pub account_address: String,
    pub slot_index: u64,
    pub initial_key: u64,
    pub key_boundry: u64,
    pub increment: u64,
}

impl ToString for DynamicLayoutDatalake {
    fn to_string(&self) -> String {
        let encoded_datalake = self.serialize().unwrap();
        let bytes = Vec::from_hex(encoded_datalake).expect("Invalid hex string");
        let hash = keccak256(bytes);
        format!("0x{:x}", hash)
    }
}

impl DynamicLayoutDatalake {
    pub fn new(
        block_number: u64,
        account_address: String,
        slot_index: u64,
        initial_key: u64,
        key_boundry: u64,
        increment: u64,
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

    pub fn serialize(&self) -> Result<String> {
        let blocknumber = DynSolValue::Uint(U256::from(self.block_number), 256);
        let account_address = DynSolValue::Address(self.account_address.parse().unwrap());
        let slot_index = DynSolValue::Uint(U256::from(self.slot_index), 256);
        let initial_key = DynSolValue::Uint(U256::from(self.initial_key), 256);
        let key_boundry = DynSolValue::Uint(U256::from(self.key_boundry), 256);
        let increment = DynSolValue::Uint(U256::from(self.increment), 256);
        let datalake_code = DynSolValue::Uint(U256::from(1), 256);

        let tuple_value = DynSolValue::Tuple(vec![
            datalake_code,
            blocknumber,
            account_address,
            slot_index,
            initial_key,
            key_boundry,
            increment,
        ]);

        let encoded_datalake = tuple_value.abi_encode();
        Ok(bytes_to_hex_string(&encoded_datalake))
    }

    pub fn deserialize(serialized: String) -> Result<Self> {
        let datalake_type: DynSolType =
            "(uint256,uint256,address,uint256,uint256,uint256,uint256)".parse()?;
        let bytes = Vec::from_hex(serialized).expect("Invalid hex string");
        let decoded = datalake_type.abi_decode(&bytes)?;

        let value = decoded.as_tuple().unwrap();
        let datalake_code = value[0].as_uint().unwrap().0.to_string().parse::<u64>()?;

        if datalake_code != 1 {
            bail!("Serialized datalake is not a dynamic layout datalake");
        }

        let block_number = value[1].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let account_address = value[2].as_address().unwrap().to_string();
        let slot_index = value[3].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let initial_key = value[4].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let key_boundry = value[5].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let increment = value[6].as_uint().unwrap().0.to_string().parse::<u64>()?;

        Ok(Self {
            block_number,
            account_address,
            slot_index,
            initial_key,
            key_boundry,
            increment,
        })
    }

    pub async fn compile(&self) -> Result<Vec<String>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_layout_datalake_serialized() {
        let dynamic_layout_datalake = "0x000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000009eb0f60000000000000000000000007b2f05ce9ae365c3dbf30657e2dc6449989e83d60000000000000000000000000000000000000000000000000000000000000005000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000001";
        let decoded_datalake =
            DynamicLayoutDatalake::deserialize(dynamic_layout_datalake.to_string()).unwrap();
        let dynamic_layout_datalake = DynamicLayoutDatalake::new(
            10399990,
            "0x7b2f05cE9aE365c3DBF30657e2DC6449989e83D6".to_string(),
            5,
            0,
            3,
            1,
        );
        assert_eq!(decoded_datalake, dynamic_layout_datalake);
        assert_eq!(
            decoded_datalake.serialize().unwrap(),
            dynamic_layout_datalake.serialize().unwrap()
        );
    }
}
