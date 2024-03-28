//![`TransactionsDatalake`] is a struct that represents a transactions datalake.
//!
//! It can represent a transactions datalake for a specific address as sender.
//!
//! example 1: from_base_nonce is 0, to_base_nonce is 3 increment is 1
//! target nonce [0, 1, 2, 3]
//! - transaction 1 (nonce 0 -> 1)
//! - transaction 2 (nonce 1 -> 2)
//! - transaction 3 (nonce 2 -> 3)
//! - transaction 4 (nonce 3 -> 4)
//!
//! example 2: from_base_nonce is 0, to_base_nonce is 3 increment is 2
//! target nonce [0, 2]
//! - transaction 1 (nonce 0 -> 1)
//! - transaction 2 (nonce 2 -> 3)
//!
//! example 3: from_base_nonce is 0, to_base_nonce is 3 increment is 3
//! target nonce [0, 3]
//! - transaction 1 (nonce 0 -> 1)
//! - transaction 2 (nonce 3 -> 4)
//!
//! example 4: from_base_nonce is 0, to_base_nonce is 5 increment is 2
//! target nonce [0, 2, 4]
//! - transaction 1 (nonce 0 -> 1)
//! - transaction 2 (nonce 2 -> 3)
//! - transaction 3 (nonce 4 -> 5)

use std::str::FromStr;

use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{hex::FromHex, keccak256, Address};
use anyhow::{bail, Result};

use crate::{
    datalake::{datalake_type::DatalakeType, Datalake},
    utils::bytes_to_hex_string,
};

use super::TransactionsCollection;

#[derive(Debug, Clone, PartialEq)]
pub struct TransactionsDatalake {
    pub address: Address,
    // start of nonce range the first transaction base on
    pub from_base_nonce: u64,
    // end of nonce range the last transaction base on
    pub to_base_nonce: u64,
    // ex. "tx.to" , "tx.gas_price" or "tx_receipt.success", "tx_receipt.cumulative_gas_used"
    pub sampled_property: TransactionsCollection,
    // increment of nonce range
    pub increment: u64,
}

impl TransactionsDatalake {
    pub fn new(
        address: String,
        from_base_nonce: u64,
        to_base_nonce: u64,
        sampled_property: String,
        increment: u64,
    ) -> Result<Self> {
        Ok(Self {
            address: Address::from_hex(address)?,
            from_base_nonce,
            to_base_nonce,
            sampled_property: TransactionsCollection::from_str(&sampled_property)?,
            increment,
        })
    }
}

impl Datalake for TransactionsDatalake {
    /// Get the datalake code for transactions datalake
    fn get_datalake_type(&self) -> DatalakeType {
        DatalakeType::Transactions
    }

    /// Encode the [`TransactionsDatalake`] into a hex string
    fn encode(&self) -> Result<String> {
        let datalake_code: DynSolValue = self.get_datalake_type().to_u8().into();
        let address: DynSolValue = DynSolValue::Address(self.address);
        let from_base_nonce: DynSolValue = self.from_base_nonce.into();
        let to_base_nonce: DynSolValue = self.to_base_nonce.into();
        let sampled_property: DynSolValue = self.sampled_property.serialize()?.to_vec().into();
        let increment: DynSolValue = self.increment.into();

        let tuple_value = DynSolValue::Tuple(vec![
            datalake_code,
            address,
            from_base_nonce,
            to_base_nonce,
            increment,
            sampled_property,
        ]);

        match tuple_value.abi_encode_sequence() {
            Some(encoded_datalake) => Ok(bytes_to_hex_string(&encoded_datalake)),
            None => bail!("Encoding failed"),
        }
    }

    /// Get the commitment hash of the [`TransactionsDatalake`]
    fn commit(&self) -> String {
        let encoded_datalake = self.encode().expect("Encoding failed");
        let bytes = Vec::from_hex(encoded_datalake).expect("Invalid hex string");
        let hash = keccak256(bytes);
        format!("0x{:x}", hash)
    }

    /// Decode the encoded transactions datalake hex string into a [`TransactionsDatalake`]
    fn decode(encoded: &str) -> Result<Self> {
        let abi_type: DynSolType = "(uint256,address,uint256,uint256,uint256,bytes)".parse()?;
        let bytes = Vec::from_hex(encoded).expect("Invalid hex string");
        let decoded = abi_type.abi_decode_sequence(&bytes)?;

        let value = decoded.as_tuple().unwrap();
        let datalake_code = value[0].as_uint().unwrap().0.to_string().parse::<u8>()?;

        if DatalakeType::from_index(datalake_code)? != DatalakeType::Transactions {
            bail!("Encoded datalake is not a transactions datalake");
        }
        let address = value[1].as_address().unwrap();
        let from_base_nonce = value[2].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let to_base_nonce = value[3].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let sampled_property =
            TransactionsCollection::deserialize(value[5].as_bytes().unwrap().try_into().unwrap())?;
        let increment = value[4].as_uint().unwrap().0.to_string().parse::<u64>()?;

        Ok(Self {
            address,
            from_base_nonce,
            to_base_nonce,
            sampled_property,
            increment,
        })
    }
}
