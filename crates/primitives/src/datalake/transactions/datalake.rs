//![`TransactionsInBlockDatalake`] is a struct that represents a transactions datalake.
//!
//! It can represent all transactions in a block with a specific increment.
//!
//! Example: `TransactionsInBlockDatalake { target_block: 100, sampled_property: "tx.to", increment: 1 }`
//! represents all transactions in block 100 with a `tx.to` property sampled with an increment of 1.

use std::str::FromStr;

use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{hex::FromHex, keccak256};
use anyhow::{bail, Result};

use crate::{
    datalake::{datalake_type::DatalakeType, Datalake, DatalakeCollection},
    utils::bytes_to_hex_string,
};

use super::TransactionsCollection;

#[derive(Debug, Clone, PartialEq)]
pub struct TransactionsInBlockDatalake {
    // target block number
    pub target_block: u64,
    // ex. "tx.to" , "tx.gas_price" or "tx_receipt.success", "tx_receipt.cumulative_gas_used"
    pub sampled_property: TransactionsCollection,
    // increment of transactions
    pub increment: u64,
}

impl TransactionsInBlockDatalake {
    pub fn new(target_block: u64, sampled_property: String, increment: u64) -> Result<Self> {
        Ok(Self {
            target_block,
            sampled_property: TransactionsCollection::from_str(&sampled_property)?,
            increment,
        })
    }
}

impl Datalake for TransactionsInBlockDatalake {
    /// Get the datalake code for transactions datalake
    fn get_datalake_type(&self) -> DatalakeType {
        DatalakeType::TransactionsInBlock
    }

    /// Encode the [`TransactionsInBlockDatalake`] into a hex string
    fn encode(&self) -> Result<String> {
        let datalake_code: DynSolValue = self.get_datalake_type().to_u8().into();
        let target_block: DynSolValue = self.target_block.into();
        let sampled_property: DynSolValue = self.sampled_property.serialize()?.into();
        let increment: DynSolValue = self.increment.into();

        let tuple_value = DynSolValue::Tuple(vec![
            datalake_code,
            target_block,
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
        let abi_type: DynSolType = "(uint256,uint256,uint256,bytes)".parse()?;
        let bytes = Vec::from_hex(encoded).expect("Invalid hex string");
        let decoded = abi_type.abi_decode_sequence(&bytes)?;

        let value = decoded.as_tuple().unwrap();
        let datalake_code = value[0].as_uint().unwrap().0.to_string().parse::<u8>()?;

        if DatalakeType::from_index(datalake_code)? != DatalakeType::TransactionsInBlock {
            bail!("Encoded datalake is not a transactions datalake");
        }

        let target_block = value[1].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let sampled_property = TransactionsCollection::deserialize(value[3].as_bytes().unwrap())?;
        let increment = value[2].as_uint().unwrap().0.to_string().parse::<u64>()?;

        Ok(Self {
            target_block,
            sampled_property,
            increment,
        })
    }
}
