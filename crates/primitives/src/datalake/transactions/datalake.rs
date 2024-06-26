//![`TransactionsInBlockDatalake`] is a struct that represents a transactions datalake.
//!
//! It can represent all transactions in a block with a specific increment.
//!
//! Example: `TransactionsInBlockDatalake { target_block: 100, sampled_property: "tx.to", increment: 1 }`
//! represents all transactions in block 100 with a `tx.to` property sampled with an increment of 1.

use std::str::FromStr;

use alloy::consensus::TxType;
use alloy::dyn_abi::{DynSolType, DynSolValue};
use alloy::primitives::B256;
use alloy::primitives::{keccak256, U256};
use anyhow::{bail, Result};

use crate::datalake::{datalake_type::DatalakeType, Datalake, DatalakeCollection};

use super::TransactionsCollection;

#[derive(Debug, Clone, PartialEq)]
pub struct TransactionsInBlockDatalake {
    // target block number
    pub target_block: u64,
    // start index of transactions range ( default 0 )
    pub start_index: u64,
    // end index of transactions range, not included in the range ( default last )
    pub end_index: u64,
    // increment of transactions
    pub increment: u64,
    // filter out the specific type of Txs
    pub included_types: IncludedTypes,
    // ex. "tx.to" , "tx.gas_price" or "tx_receipt.success", "tx_receipt.cumulative_gas_used"
    pub sampled_property: TransactionsCollection,
}

impl TransactionsInBlockDatalake {
    pub fn new(
        target_block: u64,
        sampled_property: String,
        start_index: u64,
        end_index: u64,
        increment: u64,
        included_types: &[u8],
    ) -> Result<Self> {
        Ok(Self {
            target_block,
            sampled_property: TransactionsCollection::from_str(&sampled_property)?,
            start_index,
            end_index,
            increment,
            included_types: IncludedTypes::from(included_types),
        })
    }
}

impl Datalake for TransactionsInBlockDatalake {
    /// Get the datalake code for transactions datalake
    fn get_datalake_type(&self) -> DatalakeType {
        DatalakeType::TransactionsInBlock
    }

    /// Encode the [`TransactionsInBlockDatalake`] into a hex string
    fn encode(&self) -> Result<Vec<u8>> {
        let datalake_code: DynSolValue = self.get_datalake_type().to_u8().into();
        let target_block: DynSolValue = self.target_block.into();
        let sampled_property: DynSolValue = self.sampled_property.serialize()?.into();
        let start_index: DynSolValue = self.start_index.into();
        let end_index: DynSolValue = self.end_index.into();
        let increment: DynSolValue = self.increment.into();
        let included_types: DynSolValue = self.included_types.to_uint256().into();

        let tuple_value = DynSolValue::Tuple(vec![
            datalake_code,
            target_block,
            start_index,
            end_index,
            increment,
            included_types,
            sampled_property,
        ]);

        match tuple_value.abi_encode_sequence() {
            Some(encoded_datalake) => Ok(encoded_datalake),
            None => bail!("Encoding failed"),
        }
    }

    /// Get the commitment hash of the [`TransactionsDatalake`]
    fn commit(&self) -> B256 {
        let encoded_datalake = self.encode().expect("Encoding failed");
        keccak256(encoded_datalake)
    }

    /// Decode the encoded transactions datalake hex string into a [`TransactionsDatalake`]
    fn decode(encoded: &[u8]) -> Result<Self> {
        let abi_type: DynSolType =
            "(uint256, uint256, uint256, uint256, uint256, uint256, bytes)".parse()?;
        let decoded = abi_type.abi_decode_sequence(encoded)?;

        let value = decoded.as_tuple().unwrap();
        let datalake_code = value[0].as_uint().unwrap().0.to_string().parse::<u8>()?;

        if DatalakeType::from_index(datalake_code)? != DatalakeType::TransactionsInBlock {
            bail!("Encoded datalake is not a transactions datalake");
        }

        let target_block = value[1].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let start_index = value[2].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let end_index = value[3].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let increment = value[4].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let included_types = IncludedTypes::from_uint256(value[5].as_uint().unwrap().0);
        let sampled_property = TransactionsCollection::deserialize(value[6].as_bytes().unwrap())?;

        Ok(Self {
            target_block,
            start_index,
            end_index,
            increment,
            included_types,
            sampled_property,
        })
    }
}

/// A struct to represent the included types in a transactions datalake
/// The included types are represented as a 4 byte array
/// Each byte represents a type of transaction to be included in the datalake
/// The bytes are represented as follows:
/// 0: Legacy
/// 1: EIP-2930
/// 2: EIP-1559
/// 3: EIP-4844
#[derive(Debug, Clone, PartialEq)]
pub struct IncludedTypes {
    inner: [u8; 4],
}

impl IncludedTypes {
    pub fn from(included_types: &[u8]) -> Self {
        if included_types.len() != 4 {
            panic!("Included types must be 4 bytes long");
        }
        if included_types.iter().any(|&x| x > 1) {
            panic!("Included types must be either 0 or 1");
        }
        if included_types.iter().all(|&x| x == 0) {
            panic!("At least one type must be included");
        }
        let mut inner = [0; 4];
        inner.copy_from_slice(included_types);
        Self { inner }
    }

    pub fn is_included(&self, target_type: TxType) -> bool {
        // check with the index of bytes is either 0 or 1
        self.inner[target_type as usize] != 0
    }

    pub fn to_uint256(&self) -> U256 {
        let mut bytes = [0; 32];
        bytes[28..32].copy_from_slice(&self.inner);
        U256::from_be_bytes(bytes)
    }

    pub fn from_uint256(value: U256) -> Self {
        let bytes: [u8; 32] = value.to_be_bytes();
        let mut inner = [0; 4];
        inner.copy_from_slice(&bytes[28..32]);
        Self { inner }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_included_types() {
        let included_types = IncludedTypes::from(&[1, 1, 1, 1]);
        assert!(included_types.is_included(TxType::Legacy));
        assert!(included_types.is_included(TxType::Eip2930));
        assert!(included_types.is_included(TxType::Eip1559));
        assert!(included_types.is_included(TxType::Eip4844));

        let uint256 = included_types.to_uint256();
        assert_eq!(uint256, U256::from(0x01010101));

        let included_types = IncludedTypes::from_uint256(uint256);
        assert!(included_types.is_included(TxType::Legacy));
        assert!(included_types.is_included(TxType::Eip2930));
        assert!(included_types.is_included(TxType::Eip1559));
        assert!(included_types.is_included(TxType::Eip4844));
    }

    #[test]
    fn test_included_types_partial() {
        let included_types = IncludedTypes::from(&[1, 0, 1, 0]);
        assert!(included_types.is_included(TxType::Legacy));
        assert!(!included_types.is_included(TxType::Eip2930));
        assert!(included_types.is_included(TxType::Eip1559));
        assert!(!included_types.is_included(TxType::Eip4844));

        let uint256 = included_types.to_uint256();
        assert_eq!(uint256, U256::from(0x01000100));

        let included_types = IncludedTypes::from_uint256(uint256);
        assert!(included_types.is_included(TxType::Legacy));
        assert!(!included_types.is_included(TxType::Eip2930));
        assert!(included_types.is_included(TxType::Eip1559));
        assert!(!included_types.is_included(TxType::Eip4844));
    }
}
