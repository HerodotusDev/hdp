//![`TransactionsInBlockDatalake`] is a struct that represents a transactions datalake.
//!
//! It can represent all transactions in a block with a specific increment.
//!
//! Example: `TransactionsInBlockDatalake { target_block: 100, sampled_property: "tx.to", increment: 1 }`
//! represents all transactions in block 100 with a `tx.to` property sampled with an increment of 1.

use std::num::ParseIntError;
use std::str::FromStr;

use alloy::consensus::TxType;
use alloy::primitives::U256;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::TransactionsCollection;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionsInBlockDatalake {
    pub chain_id: u64,
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
        chain_id: u64,
        target_block: u64,
        sampled_property: TransactionsCollection,
        start_index: u64,
        end_index: u64,
        increment: u64,
        included_types: IncludedTypes,
    ) -> Self {
        Self {
            chain_id,
            target_block,
            sampled_property,
            start_index,
            end_index,
            increment,
            included_types,
        }
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl FromStr for IncludedTypes {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let included_types: Vec<u8> = s
            .split(',')
            .map(|x| x.parse())
            .collect::<Result<Vec<_>, _>>()?;

        if included_types.len() != 4 {
            panic!("Included types must be 4 bytes long");
        }

        Ok(IncludedTypes::from(&included_types))
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
